use axum::{
    extract::{State, Extension},
    http::StatusCode,
    response::{Json, Redirect, IntoResponse},
    routing::{get, post, put, delete},
    Router,
    middleware,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::Deserialize;
use crate::services::email::EmailService;
use bcrypt::{hash, verify, DEFAULT_COST};
use crate::common::requests::{RegisterRequest, LoginRequest};
use crate::common::responses::{ApiResponse, LoginResponse};
use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Validation};
use urlencoding;
use sqlx::Row;
use crate::models::user::User;
use crate::models::role::Role;
use crate::database::connection::DbPool;
use crate::database::models::CreateUser;
use crate::database::queries::UserQueries;
use crate::common::jwt::{generate_jwt, decode_jwt, Claims};

use crate::services::queue::PaymentQueue;
use crate::services::cache::OtpCache;
use chrono::Utc;

#[derive(Clone)]
pub struct AppStateData {
    pub db_pool: DbPool,
    pub payment_queue: PaymentQueue,
    pub otp_cache: OtpCache,
}

pub type AppState = Arc<Mutex<AppStateData>>;

use crate::routes::templates;
use crate::routes::submissions;
use crate::routes::submitters;
// use crate::routes::subscription;
use crate::routes::stripe_webhook;
use crate::common::jwt::auth_middleware;

pub fn create_router() -> Router<AppState> {
    println!("Creating router...");
    // Create API routes with /api prefix
    let auth_routes = Router::new()
        .route("/me", get(submitters::get_me))
        .route("/users", get(get_users_handler))
        .route("/admin/members", get(get_admin_team_members_handler))
        .route("/auth/users", post(invite_user_handler))
        .route("/auth/change-password", put(change_password_handler))
        .route("/auth/profile", put(update_user_profile_handler))
        .route("/submitters", get(submitters::get_submitters))
        .route("/submitters/:id", get(submitters::get_submitter))
        .route("/submitters/:id", put(submitters::update_submitter))
        .route("/submitters/:id", delete(submitters::delete_submitter))
        // .route("/subscription/status", get(subscription::get_subscription_status))
        // .route("/subscription/payment-link", get(subscription::get_payment_link))
        .merge(submissions::create_submission_router())
        .layer(middleware::from_fn(auth_middleware));

    let public_routes = Router::new()
        .route("/auth/register", post(register_handler))
        .route("/auth/login", post(login_handler))
        .route("/auth/activate", post(activate_user))
        .route("/auth/forgot-password", post(forgot_password_handler))
        .route("/auth/verify-reset-code", post(verify_reset_code_handler))
        .route("/auth/reset-password", post(reset_password_handler))
        .route("/stripe/webhook", post(stripe_webhook::stripe_webhook_handler))
        .merge(templates::create_template_router()); // Template router has its own public/auth separation

    let api_routes = public_routes.merge(auth_routes);
    println!("About to merge submitter router");
    println!("API routes created");

    // Combine API routes with other routes
    let final_router = Router::new()
        .nest("/api", api_routes)
        .route("/health", get(health_check))
        .route("/template_google_drive", get(template_google_drive_picker))
        .route("/auth/google_oauth2", get(google_oauth_init))
        .route("/auth/google_oauth2/callback", get(google_oauth_callback))
        .route("/public/submissions/:token", get(submitters::get_public_submitter).put(submitters::update_public_submitter))
        .route("/public/submissions/:token/fields", get(submitters::get_public_submitter_fields))
        .route("/public/submissions/:token/signatures", get(submitters::get_public_submitter_signatures))
        .route("/public/signatures/bulk/:token", post(submitters::submit_bulk_signatures))
        .route("/public/download/:token", get(submitters::download_signed_pdf));
    println!("Final router created");
    final_router
}

#[utoipa::path(
    post,
    path = "/api/auth/register",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully", body = ApiResponse<User>),
        (status = 400, description = "Registration failed", body = ApiResponse<User>)
    ),
    tag = "auth"
)]
pub async fn register_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> (StatusCode, Json<ApiResponse<User>>) {
    let pool = &state.lock().await.db_pool;

    // Check if user already exists
    if let Ok(Some(_)) = UserQueries::get_user_by_email(pool, &payload.email).await {
        return ApiResponse::bad_request("User already exists".to_string());
    }

    // Hash password using bcrypt
    let password_hash = match hash(&payload.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => return ApiResponse::internal_error("Failed to hash password".to_string()),
    };

    // Create user directly (active by default for self-registration)
    let create_user = CreateUser {
        name: payload.name.clone(),
        email: payload.email.clone(),
        password_hash,
        role: Role::Admin, // Default role for new users
        is_active: true, // Self-registered users are active immediately
        activation_token: None, // No activation needed for self-registration
    };

    match UserQueries::create_user(pool, create_user).await {
        Ok(db_user) => {
            let user: User = db_user.into();
            ApiResponse::created(user, "User registered successfully. You can now login.".to_string())
        }
        Err(e) => ApiResponse::internal_error(format!("Failed to create user: {}", e)),
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = ApiResponse<LoginResponse>),
        (status = 401, description = "Login failed", body = ApiResponse<LoginResponse>)
    ),
    tag = "auth"
)]
pub async fn login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginRequest>,
) -> (StatusCode, Json<ApiResponse<LoginResponse>>) {
    let pool = &state.lock().await.db_pool;

    // Get user from database
    match UserQueries::get_user_by_email(pool, &payload.email).await {
        Ok(Some(db_user)) => {
            // Verify password using bcrypt
            match verify(&payload.password, &db_user.password_hash) {
                Ok(true) => {
                    // Generate JWT token
                    let jwt_secret = std::env::var("JWT_SECRET")
                        .unwrap_or_else(|_| "your-secret-key".to_string());

                    match generate_jwt(db_user.id, &db_user.email, &db_user.role, &jwt_secret) {
                        Ok(token) => {
                            let user: User = db_user.into();
                            let login_response = LoginResponse { token, user };
                            ApiResponse::login_success(login_response, "Login successful".to_string())
                        }
                        Err(_) => ApiResponse::internal_error("Failed to generate token".to_string()),
                    }
                }
                Ok(false) => ApiResponse::unauthorized("Invalid credentials".to_string()),
                Err(_) => ApiResponse::internal_error("Password verification failed".to_string()),
            }
        }
        Ok(None) => ApiResponse::unauthorized("User not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Database error: {}", e)),
    }
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct ActivateUserRequest {
    pub email: Option<String>, // Email from URL parameter (overrides JWT token email)
    pub name: Option<String>, // Name from URL parameter (overrides JWT token name)
    pub token: String, // JWT token from invitation link (REQUIRED)
    pub password: String, // User sets their own password during activation
}

#[utoipa::path(
    post,
    path = "/api/auth/activate",
    tag = "auth",
    request_body = ActivateUserRequest,
    responses(
        (status = 200, description = "Account activated successfully using JWT token. Email and name can be overridden via request body"),
        (status = 400, description = "Invalid JWT token or user already exists"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn activate_user(
    State(state): State<AppState>,
    Json(payload): Json<ActivateUserRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let pool = &state.lock().await.db_pool;

    // JWT Token method only (secure and modern)
    use jsonwebtoken::{decode, DecodingKey, Validation};
    use serde::{Serialize, Deserialize};
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct InvitationClaims {
        pub invitation_id: i64,
        pub name: String,
        pub email: String,
        pub role: String,
        pub exp: usize,
    }
    
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    
    // Decode and verify JWT token
    let claims = match decode::<InvitationClaims>(
        &payload.token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default()
    ) {
        Ok(token_data) => token_data.claims,
        Err(e) => {
            eprintln!("JWT decode error: {}", e);
            return Ok(Json(serde_json::json!({
                "error": "Invalid or expired invitation token"
            })));
        }
    };
    
    // Get invitation from database using invitation_id
    let invitation = match sqlx::query_as::<_, crate::database::models::DbUserInvitation>(
        r#"
        SELECT * FROM user_invitations 
        WHERE id = $1 AND email = $2 AND is_used = FALSE AND expires_at > NOW()
        "#
    )
    .bind(claims.invitation_id)
    .bind(&claims.email)
    .fetch_optional(pool)
    .await
    {
        Ok(Some(inv)) => inv,
        Ok(None) => {
            return Ok(Json(serde_json::json!({
                "error": "Invalid or expired invitation"
            })));
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // Check if user already exists
    if let Ok(Some(_)) = UserQueries::get_user_by_email(pool, &invitation.email).await {
        return Ok(Json(serde_json::json!({
            "error": "User already exists"
        })));
    }

    // Hash password
    let password_hash = match hash(&payload.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Use name from URL parameter if provided, otherwise use invitation name
    let user_name = payload.name.clone().unwrap_or(invitation.name.clone());
    let user_email = payload.email.clone().unwrap_or(invitation.email.clone());

    // Create user in database with info from invitation or URL parameters
    let create_user = CreateUser {
        name: user_name,
        email: user_email.clone(),
        password_hash,
        role: invitation.role,
        is_active: true, // User is active after activation
        activation_token: None,
    };

    match UserQueries::create_user(pool, create_user).await {
        Ok(_) => {
            // Mark invitation as used
            if let Err(e) = sqlx::query("UPDATE user_invitations SET is_used = TRUE WHERE id = $1")
                .bind(invitation.id)
                .execute(pool)
                .await
            {
                eprintln!("Failed to mark invitation as used: {}", e);
            }

            Ok(Json(serde_json::json!({
                "message": "Account activated successfully. You can now login.",
                "email": user_email
            })))
        }
        Err(e) => {
            eprintln!("Failed to create user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Change password request struct
#[derive(Deserialize, utoipa::ToSchema)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

// Change password handler
#[utoipa::path(
    put,
    path = "/api/auth/change-password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully", body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Invalid current password or request", body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Unauthorized", body = ApiResponse<serde_json::Value>),
        (status = 500, description = "Internal server error", body = ApiResponse<serde_json::Value>)
    ),
    security(("bearer_auth" = [])),
    tag = "auth"
)]
pub async fn change_password_handler(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<ChangePasswordRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let pool = &state.lock().await.db_pool;

    // Get user from database
    match UserQueries::get_user_by_id(pool, user_id).await {
        Ok(Some(db_user)) => {
            // Verify current password
            match verify(&payload.current_password, &db_user.password_hash) {
                Ok(true) => {
                    // Hash new password
                    match hash(&payload.new_password, DEFAULT_COST) {
                        Ok(new_password_hash) => {
                            // Update password in database
                            match UserQueries::update_user_password(pool, user_id, new_password_hash).await {
                                Ok(_) => ApiResponse::success(
                                    serde_json::json!({
                                        "message": "Password changed successfully"
                                    }),
                                    "Password updated successfully".to_string()
                                ),
                                Err(e) => ApiResponse::internal_error(format!("Failed to update password: {}", e)),
                            }
                        }
                        Err(_) => ApiResponse::internal_error("Failed to hash new password".to_string()),
                    }
                }
                Ok(false) => ApiResponse::bad_request("Current password is incorrect".to_string()),
                Err(_) => ApiResponse::internal_error("Password verification failed".to_string()),
            }
        }
        Ok(None) => ApiResponse::unauthorized("User not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Database error: {}", e)),
    }
}

// Update user profile request struct
#[derive(Deserialize, utoipa::ToSchema)]
pub struct UpdateUserRequest {
    pub name: String,
    pub email: Option<String>,
    pub signature: Option<String>,
    pub initials: Option<String>,
}

// Update user profile handler
#[utoipa::path(
    put,
    path = "/api/auth/profile",
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = ApiResponse<User>),
        (status = 400, description = "Invalid request data", body = ApiResponse<User>),
        (status = 401, description = "Unauthorized", body = ApiResponse<User>),
        (status = 500, description = "Internal server error", body = ApiResponse<User>)
    ),
    security(("bearer_auth" = [])),
    tag = "auth"
)]
pub async fn update_user_profile_handler(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<UpdateUserRequest>,
) -> (StatusCode, Json<ApiResponse<User>>) {
    let pool = &state.lock().await.db_pool;

    // Validate name is not empty
    if payload.name.trim().is_empty() {
        return ApiResponse::bad_request("Name cannot be empty".to_string());
    }

    // Validate email if provided
    if let Some(ref email) = payload.email {
        if email.trim().is_empty() {
            return ApiResponse::bad_request("Email cannot be empty".to_string());
        }
        // Check if email is already in use by another user
        if let Ok(Some(existing_user)) = UserQueries::get_user_by_email(pool, email).await {
            if existing_user.id != user_id {
                return ApiResponse::bad_request("Email is already in use".to_string());
            }
        }
    }

    // Update user name in database
    if let Err(e) = UserQueries::update_user_name(pool, user_id, payload.name.clone()).await {
        return ApiResponse::internal_error(format!("Failed to update name: {}", e));
    }

    // Update user email if provided
    if let Some(email) = payload.email.clone() {
        if let Err(e) = UserQueries::update_user_email(pool, user_id, email).await {
            return ApiResponse::internal_error(format!("Failed to update email: {}", e));
        }
    }

    // Update user signature if provided
    if let Some(signature) = payload.signature.clone() {
        if let Err(e) = UserQueries::update_user_signature(pool, user_id, signature).await {
            return ApiResponse::internal_error(format!("Failed to update signature: {}", e));
        }
    }

    // Update user initials if provided
    if let Some(initials) = payload.initials.clone() {
        if let Err(e) = UserQueries::update_user_initials(pool, user_id, initials).await {
            return ApiResponse::internal_error(format!("Failed to update initials: {}", e));
        }
    }

    // Get updated user data
    match UserQueries::get_user_by_id(pool, user_id).await {
        Ok(Some(db_user)) => {
            let user: User = db_user.into();
            ApiResponse::success(user, "Profile updated successfully".to_string())
        }
        Ok(None) => ApiResponse::unauthorized("User not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to retrieve updated user: {}", e)),
    }
}

// Forgot password request struct
#[derive(Deserialize, utoipa::ToSchema)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

// Verify reset code request struct
#[derive(Deserialize, utoipa::ToSchema)]
pub struct VerifyResetCodeRequest {
    pub email: String,
    pub reset_code: String,
}

// Reset password request struct
#[derive(Deserialize, utoipa::ToSchema)]
pub struct ResetPasswordRequest {
    pub email: String,
    pub reset_code: String,
    pub new_password: String,
}

// Forgot password handler - sends OTP via email
#[utoipa::path(
    post,
    path = "/api/auth/forgot-password",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "OTP sent successfully", body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Invalid email or user not found", body = ApiResponse<serde_json::Value>),
        (status = 500, description = "Internal server error", body = ApiResponse<serde_json::Value>)
    ),
    tag = "auth"
)]
pub async fn forgot_password_handler(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let state_data = state.lock().await;

    // Check if user exists
    match UserQueries::get_user_by_email(&state_data.db_pool, &payload.email).await {
        Ok(Some(db_user)) => {
            // Generate 6-digit OTP
            use rand::Rng;
            let otp_code: u32 = rand::thread_rng().gen_range(100000..=999999);
            let otp_string = otp_code.to_string();

            // Store OTP in cache with 15 minutes TTL
            match state_data.otp_cache.store_otp(&payload.email, &otp_string, 900).await {
                Ok(_) => {
                    // Send email with OTP
                    let email_service = match EmailService::new() {
                        Ok(service) => service,
                        Err(e) => {
                            eprintln!("Failed to initialize email service: {:?}", e);
                            return (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse {
                                success: false,
                                status_code: 500,
                                message: "Internal server error".to_string(),
                                data: None,
                                error: Some("Email service unavailable".to_string()),
                            }));
                        }
                    };

                    match email_service.send_password_reset_code(&payload.email, &db_user.name, &otp_string).await {
                        Ok(_) => (StatusCode::OK, Json(ApiResponse {
                            success: true,
                            status_code: 200,
                            message: "OTP sent successfully".to_string(),
                            data: Some(serde_json::json!({
                                "message": "Password reset OTP sent to your email"
                            })),
                            error: None,
                        })),
                        Err(e) => {
                            eprintln!("Failed to send OTP email: {:?}", e);
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse {
                                success: false,
                                status_code: 500,
                                message: "Internal server error".to_string(),
                                data: None,
                                error: Some("Failed to send OTP email".to_string()),
                            }))
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to store OTP: {:?}", e);
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse {
                        success: false,
                        status_code: 500,
                        message: "Internal server error".to_string(),
                        data: None,
                        error: Some("Failed to generate OTP".to_string()),
                    }))
                }
            }
        }
        Ok(None) => (StatusCode::BAD_REQUEST, Json(ApiResponse {
            success: false,
            status_code: 400,
            message: "Bad request".to_string(),
            data: None,
            error: Some("User with this email not found".to_string()),
        })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse {
            success: false,
            status_code: 500,
            message: "Internal server error".to_string(),
            data: None,
            error: Some(format!("Database error: {}", e)),
        })),
    }
}

// Verify reset code handler
#[utoipa::path(
    post,
    path = "/api/auth/verify-reset-code",
    request_body = VerifyResetCodeRequest,
    responses(
        (status = 200, description = "OTP is valid", body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Invalid or expired OTP", body = ApiResponse<serde_json::Value>),
        (status = 500, description = "Internal server error", body = ApiResponse<serde_json::Value>)
    ),
    tag = "auth"
)]
pub async fn verify_reset_code_handler(
    State(state): State<AppState>,
    Json(payload): Json<VerifyResetCodeRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let state_data = state.lock().await;

    match state_data.otp_cache.verify_otp(&payload.email, &payload.reset_code).await {
        Ok(true) => (StatusCode::OK, Json(ApiResponse {
            success: true,
            status_code: 200,
            message: "Code verified successfully".to_string(),
            data: Some(serde_json::json!({
                "message": "OTP is valid"
            })),
            error: None,
        })),
        Ok(false) => (StatusCode::BAD_REQUEST, Json(ApiResponse {
            success: false,
            status_code: 400,
            message: "Bad request".to_string(),
            data: None,
            error: Some("Invalid or expired OTP".to_string()),
        })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse {
            success: false,
            status_code: 500,
            message: "Internal server error".to_string(),
            data: None,
            error: Some(format!("Verification error: {}", e)),
        })),
    }
}

// Reset password handler - verifies OTP and resets password in one step
#[utoipa::path(
    post,
    path = "/api/auth/reset-password",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successfully", body = ApiResponse<serde_json::Value>),
        (status = 400, description = "Invalid OTP or request", body = ApiResponse<serde_json::Value>),
        (status = 500, description = "Internal server error", body = ApiResponse<serde_json::Value>)
    ),
    tag = "auth"
)]
pub async fn reset_password_handler(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let state_data = state.lock().await;

    // Verify the OTP first
    match state_data.otp_cache.verify_otp(&payload.email, &payload.reset_code).await {
        Ok(true) => {
            // Get user by email
            match UserQueries::get_user_by_email(&state_data.db_pool, &payload.email).await {
                Ok(Some(db_user)) => {
                    // Hash new password
                    match hash(&payload.new_password, DEFAULT_COST) {
                        Ok(new_password_hash) => {
                            // Update password
                            match UserQueries::update_user_password(&state_data.db_pool, db_user.id, new_password_hash).await {
                                Ok(_) => (StatusCode::OK, Json(ApiResponse {
                                    success: true,
                                    status_code: 200,
                                    message: "Password reset successfully".to_string(),
                                    data: Some(serde_json::json!({
                                        "message": "Password reset successfully"
                                    })),
                                    error: None,
                                })),
                                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse {
                                    success: false,
                                    status_code: 500,
                                    message: "Internal server error".to_string(),
                                    data: None,
                                    error: Some(format!("Failed to update password: {}", e)),
                                })),
                            }
                        }
                        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse {
                            success: false,
                            status_code: 500,
                            message: "Internal server error".to_string(),
                            data: None,
                            error: Some("Failed to hash new password".to_string()),
                        })),
                    }
                }
                Ok(None) => (StatusCode::BAD_REQUEST, Json(ApiResponse {
                    success: false,
                    status_code: 400,
                    message: "Bad request".to_string(),
                    data: None,
                    error: Some("User not found".to_string()),
                })),
                Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse {
                    success: false,
                    status_code: 500,
                    message: "Internal server error".to_string(),
                    data: None,
                    error: Some(format!("Database error: {}", e)),
                })),
            }
        }
        Ok(false) => (StatusCode::BAD_REQUEST, Json(ApiResponse {
            success: false,
            status_code: 400,
            message: "Bad request".to_string(),
            data: None,
            error: Some("Invalid or expired OTP".to_string()),
        })),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiResponse {
            success: false,
            status_code: 500,
            message: "Internal server error".to_string(),
            data: None,
            error: Some(format!("Verification error: {}", e)),
        })),
    }
}

// Struct for invite user request (Admin only sends invitation, no password needed)
#[derive(Deserialize, utoipa::ToSchema)]
pub struct InviteUserRequest {
    pub name: String,
    pub email: String,
    pub role: Role,
}

// Invite user to team (Admin only - sends activation email, user data NOT created until activation)
#[utoipa::path(
    post,
    path = "/api/auth/users",
    request_body = InviteUserRequest,
    responses(
        (status = 200, description = "User invitation sent successfully"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "auth"
)]
pub async fn invite_user_handler(
    State(state): State<AppState>,
    axum::Extension(user_id): axum::Extension<i64>,
    Json(payload): Json<InviteUserRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let pool = &state.lock().await.db_pool;

    // Check if inviting user is admin
    match UserQueries::get_user_by_id(pool, user_id).await {
        Ok(Some(inviter)) => {
            let role_str = inviter.role.to_lowercase();
            if role_str != "admin" {
                return ApiResponse::unauthorized("Only admins can invite users".to_string());
            }
        }
        _ => return ApiResponse::unauthorized("Invalid user".to_string()),
    }

    // Check if email already exists (in users or pending invitations)
    if let Ok(Some(_)) = UserQueries::get_user_by_email(pool, &payload.email).await {
        return ApiResponse::bad_request("User with this email already exists".to_string());
    }

    // Check if invitation already exists
    match sqlx::query("SELECT id FROM user_invitations WHERE email = $1 AND is_used = FALSE")
        .bind(&payload.email)
        .fetch_optional(pool)
        .await
    {
        Ok(Some(_)) => return ApiResponse::bad_request("Invitation already sent to this email".to_string()),
        Ok(None) => {}, // No existing invitation, continue
        Err(e) => {
            eprintln!("Failed to check existing invitations: {}", e);
            return ApiResponse::internal_error("Database error".to_string());
        }
    }

    // Save invitation to database (NOT create user yet)
    let result = sqlx::query(
        r#"
        INSERT INTO user_invitations (email, name, role, invited_by_user_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#
    )
    .bind(&payload.email)
    .bind(&payload.name)
    .bind(&payload.role)
    .bind(user_id)
    .fetch_one(pool)
    .await;

    // Get the invitation ID from result
    let invitation_row = match result {
        Ok(row) => row,
        Err(e) => {
            eprintln!("Database error creating invitation: {}", e);
            return ApiResponse::internal_error("Failed to create invitation".to_string());
        }
    };
    
    let invitation_id: i64 = invitation_row.get("id");

    // Generate JWT token for invitation (expires in 24 hours)
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    
    // Create claims with invitation data
    use chrono::{Duration, Utc};
    use serde::{Serialize, Deserialize};
    
    #[derive(Debug, Serialize, Deserialize)]
    struct InvitationClaims {
        pub invitation_id: i64,
        pub name: String,
        pub email: String,
        pub role: String,
        pub exp: usize,
    }
    
    let claims = InvitationClaims {
        invitation_id,
        name: payload.name.clone(),
        email: payload.email.clone(),
        role: payload.role.to_string(),
        exp: (Utc::now() + Duration::hours(24)).timestamp() as usize,
    };
    
    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_secret.as_ref())) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to create JWT token: {}", e);
            return ApiResponse::internal_error("Failed to create invitation token".to_string());
        }
    };

    // Send invitation email with JWT token link
    let email_service = match EmailService::new() {
        Ok(service) => service,
        Err(e) => {
            eprintln!("Failed to initialize email service: {}", e);
            return ApiResponse::internal_error("Email service unavailable".to_string());
        }
    };

    // Activation link with JWT token, email and name in URL
    let activation_link = format!(
    "{}/activate?token={}&email={}&name={}",
    std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string()),
    token,
    urlencoding::encode(&payload.email),
    urlencoding::encode(&payload.name)
);

    // Send email with activation link (email service will generate proper template)
    if let Err(e) = email_service.send_user_activation_email(&payload.email, &payload.name, &activation_link).await {
        eprintln!("Failed to send invitation email: {}", e);
        // Don't fail - invitation is saved, admin can resend
    }

    ApiResponse::success(
        serde_json::json!({
            "message": "Invitation sent successfully",
            "email": payload.email,
            "name": payload.name,
            "role": payload.role,
            "invitation_id": invitation_id
        }),
        "User invitation sent. They will receive an email with activation link.".to_string()
    )
}

// Get all users (Admin only)
#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = 200, description = "List of users", body = Vec<User>),
        (status = 401, description = "Unauthorized")
    ),
    tag = "users"
)]
pub async fn get_users_handler(
    State(state): State<AppState>,
    axum::Extension(user_id): axum::Extension<i64>,
) -> (StatusCode, Json<ApiResponse<Vec<User>>>) {
    let pool = &state.lock().await.db_pool;

    // Check if requesting user is admin
    match UserQueries::get_user_by_id(pool, user_id).await {
        Ok(Some(requester)) => {
            let role_str = requester.role.to_lowercase();
            if role_str != "admin" {
                return ApiResponse::unauthorized("Only admins can view users list".to_string());
            }
        }
        _ => return ApiResponse::unauthorized("Invalid user".to_string()),
    }

    // Get all users
    match sqlx::query_as::<_, crate::database::models::DbUser>(
        "SELECT * FROM users ORDER BY created_at DESC"
    )
    .fetch_all(pool)
    .await
    {
        Ok(db_users) => {
            let users: Vec<User> = db_users.into_iter().map(|u| u.into()).collect();
            ApiResponse::success(users, "Users retrieved successfully".to_string())
        }
        Err(e) => ApiResponse::internal_error(format!("Database error: {}", e)),
    }
}

// Get team members invited by the current admin
#[utoipa::path(
    get,
    path = "/api/admin/members",
    responses(
        (status = 200, description = "List of team members", body = Vec<crate::models::user::TeamMember>),
        (status = 401, description = "Unauthorized")
    ),
    tag = "auth"
)]
pub async fn get_admin_team_members_handler(
    State(state): State<AppState>,
    axum::Extension(user_id): axum::Extension<i64>,
) -> (StatusCode, Json<ApiResponse<Vec<crate::models::user::TeamMember>>>) {
    let pool = &state.lock().await.db_pool;

    // Check if requesting user is admin
    match UserQueries::get_user_by_id(pool, user_id).await {
        Ok(Some(requester)) => {
            let role_str = requester.role.to_lowercase();
            if role_str != "admin" {
                return ApiResponse::unauthorized("Only admins can view team members".to_string());
            }
        }
        _ => return ApiResponse::unauthorized("Invalid user".to_string()),
    }

    // Get all invitations sent by this admin
    match sqlx::query_as::<_, crate::database::models::DbUserInvitation>(
        "SELECT * FROM user_invitations WHERE invited_by_user_id = $1 ORDER BY created_at DESC"
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    {
        Ok(invitations) => {
            let mut team_members = Vec::new();
            for inv in invitations {
                // Check if user exists (activated)
                let status = if let Ok(Some(_)) = UserQueries::get_user_by_email(pool, &inv.email).await {
                    "active"
                } else {
                    "pending"
                };
                team_members.push(crate::models::user::TeamMember {
                    id: None, // For now, not setting id
                    name: inv.name,
                    email: inv.email,
                    role: inv.role,
                    status: status.to_string(),
                    created_at: inv.created_at,
                });
            }
            ApiResponse::success(team_members, "Team members retrieved successfully".to_string())
        }
        Err(e) => ApiResponse::internal_error(format!("Database error: {}", e)),
    }
}

async fn health_check() -> &'static str {
    "OK"
}

use axum::response::Html;

async fn template_google_drive_picker() -> Html<String> {
    let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_else(|_| "YOUR_GOOGLE_CLIENT_ID".to_string());
    let developer_key = std::env::var("GOOGLE_DEVELOPER_KEY").unwrap_or_else(|_| "YOUR_GOOGLE_DEVELOPER_KEY".to_string());
    let html = format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>Google Drive Picker</title>
    <script type="text/javascript">
        let pickerApiLoaded = false;
        let oauthToken = null;

        function onApiLoad() {{
            window.gapi.load('auth2', onAuthApiLoad);
            window.gapi.load('picker', onPickerApiLoad);
        }}

        function onAuthApiLoad() {{
            window.gapi.auth2.init({{
                client_id: '{}'
            }});
        }}

        function onPickerApiLoad() {{
            pickerApiLoaded = true;
            fetch('/api/me', {{
                headers: {{
                    'Authorization': 'Bearer ' + localStorage.getItem('token')
                }}
            }}).then(response => response.json()).then(data => {{
                if (data.success && data.data.oauth_tokens) {{
                    const googleToken = data.data.oauth_tokens.find(t => t.provider === 'google');
                    if (googleToken) {{
                        oauthToken = googleToken.access_token;
                        createPicker();
                    }} else {{
                        requestOAuth();
                    }}
                }} else {{
                    requestOAuth();
                }}
            }}).catch(() => {{
                requestOAuth();
            }});
        }}

        function requestOAuth() {{
            window.parent.postMessage({{ type: 'google-drive-picker-request-oauth' }}, '*' );
        }}

        function createPicker() {{
            if (pickerApiLoaded && oauthToken) {{
                const picker = new google.picker.PickerBuilder()
                    .addView(google.picker.ViewId.DOCS)
                    .setOAuthToken(oauthToken)
                    .setDeveloperKey('{}')
                    .setCallback(pickerCallback)
                    .build();
                picker.setVisible(true);
            }}
        }}

        function pickerCallback(data) {{
            if (data.action === google.picker.Action.PICKED) {{
                window.parent.postMessage({{
                    type: 'google-drive-files-picked',
                    files: data.docs
                }}, '*' );
            }}
        }}

        window.addEventListener('load', function() {{
            window.parent.postMessage({{ type: 'google-drive-picker-loaded' }}, '*' );
        }});
    </script>
    <script src="https://apis.google.com/js/api.js?onload=onApiLoad"></script>
</head>
<body>
    <div id="picker-container"></div>
</body>
</html>
"#, client_id, developer_key);
    Html(html)
}

use axum::extract::Query;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct GoogleOAuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
}

async fn google_oauth_init(
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // Redirect to Google OAuth
    let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_else(|_| "YOUR_GOOGLE_CLIENT_ID".to_string());
    let redirect_uri = format!("{}/auth/google_oauth2/callback", std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()));

    let scope = "https://www.googleapis.com/auth/userinfo.email https://www.googleapis.com/auth/drive.file";
    let state = params.get("state").unwrap_or(&"".to_string()).clone();

    let auth_url = format!(
        "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&scope={}&response_type=code&access_type=offline&prompt=consent{}",
        client_id,
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(scope),
        if !state.is_empty() { format!("&state={}", urlencoding::encode(&state)) } else { "".to_string() }
    );

    Redirect::to(&auth_url)
}

async fn google_oauth_callback(
    Query(query): Query<GoogleOAuthCallbackQuery>,
    Extension(user_id): Extension<i64>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let pool = &state.lock().await.db_pool;

    if let Some(code) = query.code {
        // Exchange code for tokens
        let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_else(|_| "YOUR_GOOGLE_CLIENT_ID".to_string());
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_else(|_| "YOUR_GOOGLE_CLIENT_SECRET".to_string());
        let redirect_uri = format!("{}/auth/google_oauth2/callback", std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string()));

        let client = reqwest::Client::new();
        let token_response = match client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("client_id", client_id.as_str()),
                ("client_secret", client_secret.as_str()),
                ("code", &code),
                ("grant_type", "authorization_code"),
                ("redirect_uri", redirect_uri.as_str()),
            ])
            .send()
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("Failed to exchange code for tokens: {}", e);
                return Redirect::to("/dashboard?error=oauth_failed");
            }
        };

        if !token_response.status().is_success() {
            eprintln!("Token exchange failed: {}", token_response.status());
            return Redirect::to("/dashboard?error=oauth_failed");
        }

        let token_data: serde_json::Value = match token_response.json().await {
            Ok(data) => data,
            Err(e) => {
                eprintln!("Failed to parse token response: {}", e);
                return Redirect::to("/dashboard?error=oauth_failed");
            }
        };

        let access_token = token_data["access_token"].as_str().unwrap_or("").to_string();
        let refresh_token = token_data["refresh_token"].as_str().map(|s| s.to_string());
        let expires_in = token_data["expires_in"].as_u64().unwrap_or(3600);
        let expires_at = Some(Utc::now() + chrono::Duration::seconds(expires_in as i64));

        // Store tokens in database
        let create_token = super::super::database::models::CreateOAuthToken {
            user_id,
            provider: "google".to_string(),
            access_token,
            refresh_token,
            expires_at,
        };

        if let Err(e) = super::super::database::queries::OAuthTokenQueries::create_oauth_token(pool, create_token).await {
            eprintln!("Failed to store OAuth token: {}", e);
            return Redirect::to("/dashboard?error=token_storage_failed");
        }

        // Redirect back to dashboard or the original page
        let redirect_url = if let Some(state) = query.state {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&state) {
                if let Some(redir) = parsed["redir"].as_str() {
                    format!("{}?google_drive_connected=1", redir)
                } else {
                    "/dashboard?google_drive_connected=1".to_string()
                }
            } else {
                "/dashboard?google_drive_connected=1".to_string()
            }
        } else {
            "/dashboard?google_drive_connected=1".to_string()
        };

        Redirect::to(&redirect_url)
    } else {
        Redirect::to("/dashboard?error=oauth_no_code")
    }
}