use axum::{
    extract::{Request, State, Extension},
    http::StatusCode,
    response::Json,
    routing::{get, put},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::common::responses::ApiResponse;
use crate::database::models::{DbGlobalSettings, UpdateGlobalSettings};
use crate::database::queries::GlobalSettingsQueries;
use crate::routes::web::AppState;

#[utoipa::path(
    get,
    path = "/api/settings/user",
    responses(
        (status = 200, description = "User settings retrieved successfully", body = ApiResponse<DbGlobalSettings>),
        (status = 401, description = "Unauthorized", body = ApiResponse<DbGlobalSettings>),
        (status = 500, description = "Internal server error", body = ApiResponse<DbGlobalSettings>)
    ),
    tag = "settings"
)]
pub async fn get_user_settings(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> (StatusCode, Json<ApiResponse<DbGlobalSettings>>) {
    let pool = &state.lock().await.db_pool;
    let user_id_i32 = user_id as i32;

        match GlobalSettingsQueries::get_user_settings(pool, user_id as i32).await {
        Ok(Some(settings)) => ApiResponse::success(settings, "User settings retrieved successfully".to_string()),
        Ok(None) => {
            // Create default user settings
            match GlobalSettingsQueries::create_user_settings(pool, user_id as i32).await {
                Ok(settings) => ApiResponse::success(settings, "User settings created and retrieved successfully".to_string()),
                Err(e) => ApiResponse::internal_error(format!("Failed to create user settings: {}", e)),
            }
        }
        Err(e) => ApiResponse::internal_error(format!("Failed to retrieve user settings: {}", e)),
    }
}

#[utoipa::path(
    put,
    path = "/api/settings/user",
    request_body = UpdateGlobalSettings,
    responses(
        (status = 200, description = "User settings updated successfully", body = ApiResponse<DbGlobalSettings>),
        (status = 401, description = "Unauthorized", body = ApiResponse<DbGlobalSettings>),
        (status = 500, description = "Internal server error", body = ApiResponse<DbGlobalSettings>)
    ),
    tag = "settings"
)]
pub async fn update_user_settings(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<UpdateGlobalSettings>,
) -> (StatusCode, Json<ApiResponse<DbGlobalSettings>>) {
    let pool = &state.lock().await.db_pool;
    let user_id_i32 = user_id as i32;
    println!("update_user_settings called with user_id: {}, payload: {:?}", user_id, payload);

    match GlobalSettingsQueries::update_user_settings(pool, user_id as i32, payload).await {
        Ok(settings) => {
            println!("update_user_settings success: {:?}", settings);
            ApiResponse::success(settings, "User settings updated successfully".to_string())
        }
        Err(e) => {
            println!("update_user_settings error: {}", e);
            ApiResponse::internal_error(format!("Failed to update user settings: {}", e))
        }
    }
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/settings/user", get(get_user_settings).put(update_user_settings))
}