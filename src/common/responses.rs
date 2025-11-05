use axum::http::StatusCode;
use axum::response::Json;
use serde::Serialize;
use utoipa::ToSchema;

/// Generic API response structure for all endpoints
#[derive(Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub status_code: u16,
    pub message: String,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    /// 200 OK - Success response
    pub fn ok(data: T, message: String) -> (StatusCode, Json<Self>) {
        (
            StatusCode::OK,
            Json(Self {
                success: true,
                status_code: 200,
                message,
                data: Some(data),
                error: None,
            }),
        )
    }

    /// 201 Created - Resource created successfully
    pub fn created(data: T, message: String) -> (StatusCode, Json<Self>) {
        (
            StatusCode::CREATED,
            Json(Self {
                success: true,
                status_code: 201,
                message,
                data: Some(data),
                error: None,
            }),
        )
    }

    /// 400 Bad Request - Client error
    pub fn bad_request(error: String) -> (StatusCode, Json<ApiResponse<T>>) {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                success: false,
                status_code: 400,
                message: "Bad Request".to_string(),
                data: None,
                error: Some(error),
            }),
        )
    }

    /// 401 Unauthorized - Authentication required
    pub fn unauthorized(error: String) -> (StatusCode, Json<ApiResponse<T>>) {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse {
                success: false,
                status_code: 401,
                message: "Unauthorized".to_string(),
                data: None,
                error: Some(error),
            }),
        )
    }

    /// 403 Forbidden - Access denied
    pub fn forbidden(error: String) -> (StatusCode, Json<ApiResponse<T>>) {
        (
            StatusCode::FORBIDDEN,
            Json(ApiResponse {
                success: false,
                status_code: 403,
                message: "Forbidden".to_string(),
                data: None,
                error: Some(error),
            }),
        )
    }

    /// 404 Not Found - Resource not found
    pub fn not_found(error: String) -> (StatusCode, Json<ApiResponse<T>>) {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse {
                success: false,
                status_code: 404,
                message: "Not Found".to_string(),
                data: None,
                error: Some(error),
            }),
        )
    }

    /// 500 Internal Server Error - Server error
    pub fn internal_error(error: String) -> (StatusCode, Json<ApiResponse<T>>) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse {
                success: false,
                status_code: 500,
                message: "Internal Server Error".to_string(),
                data: None,
                error: Some(error),
            }),
        )
    }

    /// 200 OK - Success response (alias for ok)
    pub fn success(data: T, message: String) -> (StatusCode, Json<Self>) {
        Self::ok(data, message)
    }
}

/// Login response containing JWT token and user info
#[derive(Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub user: super::super::models::user::User,
}

/// 2FA required response
#[derive(Serialize, ToSchema)]
pub struct TwoFactorRequiredResponse {
    pub requires_2fa: bool,
    pub temp_token: String, // Temporary token for 2FA verification
    pub user_id: i64,
}

impl ApiResponse<LoginResponse> {
    /// 200 OK - Login success response
    pub fn login_success(login_data: LoginResponse, message: String) -> (StatusCode, Json<Self>) {
        (
            StatusCode::OK,
            Json(Self {
                success: true,
                status_code: 200,
                message,
                data: Some(login_data),
                error: None,
            }),
        )
    }
}

impl ApiResponse<TwoFactorRequiredResponse> {
    /// 200 OK - 2FA required response
    pub fn two_factor_required(tfa_data: TwoFactorRequiredResponse, message: String) -> (StatusCode, Json<Self>) {
        (
            StatusCode::OK,
            Json(Self {
                success: true,
                status_code: 200,
                message,
                data: Some(tfa_data),
                error: None,
            }),
        )
    }
}