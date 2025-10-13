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
use bcrypt::{hash, verify, DEFAULT_COST};

use crate::common::requests::{RegisterRequest, LoginRequest};
use crate::common::responses::{ApiResponse, LoginResponse};
use crate::models::user::User;
use crate::models::role::Role;
use crate::database::connection::DbPool;
use crate::database::models::CreateUser;
use crate::database::queries::UserQueries;
use crate::common::jwt::generate_jwt;

pub type AppState = Arc<Mutex<DbPool>>;

use crate::routes::templates;
use crate::routes::submissions;
use crate::routes::submitters;
use crate::common::jwt::auth_middleware;

pub fn create_router() -> Router<AppState> {
    println!("Creating router...");
    // Create API routes with /api prefix
    let auth_routes = Router::new()
        .route("/me", get(submitters::get_me))
        .route("/submitters", get(submitters::get_submitters))
        .route("/submitters/:id", get(submitters::get_submitter))
        .route("/submitters/:id", put(submitters::update_submitter))
        .route("/submitters/:id", delete(submitters::delete_submitter))
        .merge(submissions::create_submission_router())
        .layer(middleware::from_fn(auth_middleware));

    let public_routes = Router::new()
        .route("/auth/register", post(register_handler))
        .route("/auth/login", post(login_handler))
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
    let pool = &*state.lock().await;

    // Check if user already exists
    if let Ok(Some(_)) = UserQueries::get_user_by_email(pool, &payload.email).await {
        return ApiResponse::bad_request("User already exists".to_string());
    }

    // Hash password using bcrypt
    let password_hash = match hash(&payload.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => return ApiResponse::internal_error("Failed to hash password".to_string()),
    };

    // Create user in database
    let create_user = CreateUser {
        name: payload.name.clone(),
        email: payload.email.clone(),
        password_hash,
        role: Role::TeamMember, // Default role for new users
    };

    match UserQueries::create_user(pool, create_user).await {
        Ok(db_user) => {
            let user: User = db_user.into();
            ApiResponse::created(user, "User registered successfully".to_string())
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
    let pool = &*state.lock().await;

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

async fn health_check() -> &'static str {
    "OK"
}