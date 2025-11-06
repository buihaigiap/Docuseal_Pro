use axum::{
    extract::State,
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
    path = "/api/settings/global",
    responses(
        (status = 200, description = "Global settings retrieved successfully", body = ApiResponse<DbGlobalSettings>),
        (status = 500, description = "Internal server error", body = ApiResponse<DbGlobalSettings>)
    ),
    tag = "settings"
)]
pub async fn get_global_settings(
    State(state): State<AppState>,
) -> (StatusCode, Json<ApiResponse<DbGlobalSettings>>) {
    let pool = &state.lock().await.db_pool;

    match GlobalSettingsQueries::get_global_settings(pool).await {
        Ok(Some(settings)) => ApiResponse::success(settings, "Global settings retrieved successfully".to_string()),
        Ok(None) => ApiResponse::internal_error("Global settings not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to retrieve global settings: {}", e)),
    }
}

#[utoipa::path(
    put,
    path = "/api/settings/global",
    request_body = UpdateGlobalSettingsRequest,
    responses(
        (status = 200, description = "Global settings updated successfully", body = ApiResponse<DbGlobalSettings>),
        (status = 400, description = "Bad request", body = ApiResponse<DbGlobalSettings>),
        (status = 500, description = "Internal server error", body = ApiResponse<DbGlobalSettings>)
    ),
    tag = "settings"
)]
pub async fn update_global_settings(
    State(state): State<AppState>,
    Json(payload): Json<UpdateGlobalSettings>,
) -> (StatusCode, Json<ApiResponse<DbGlobalSettings>>) {
    let pool = &state.lock().await.db_pool;

    match GlobalSettingsQueries::update_global_settings(pool, payload).await {
        Ok(settings) => ApiResponse::success(settings, "Global settings updated successfully".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to update global settings: {}", e)),
    }
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/global", get(get_global_settings).put(update_global_settings))
}