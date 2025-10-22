use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
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

#[derive(Clone)]
pub struct AppStateData {
    pub db_pool: DbPool,
    pub payment_queue: PaymentQueue,
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
        .route("/auth/users", post(invite_user_handler))
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
        .route("/stripe/webhook", post(stripe_webhook::stripe_webhook_handler))
        .merge(templates::create_template_router()); // Template router has its own public/auth separation

    let api_routes = public_routes.merge(auth_routes);
    println!("About to merge submitter router");
    println!("API routes created");

    // Combine API routes with other routes
    let final_router = Router::new()
        .nest("/api", api_routes)
        .route("/health", get(health_check))
        .route("/public/submissions/:token", get(submitters::get_public_submitter).put(submitters::update_public_submitter))
        .route("/public/submissions/:token/fields", get(submitters::get_public_submitter_fields))
        .route("/public/submissions/:token/signatures", get(submitters::get_public_submitter_signatures))
        .route("/public/signatures/bulk/:token", post(submitters::submit_bulk_signatures));
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
        role: Role::Member, // Default role for new users
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
        "{}/auth/activate?token={}&email={}&name={}",
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

async fn health_check() -> &'static str {
    "OK"
}