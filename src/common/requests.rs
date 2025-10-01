use serde::Deserialize;
use utoipa::ToSchema;

/// Register user request
#[derive(Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

/// Login user request
#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}