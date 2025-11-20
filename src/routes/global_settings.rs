use axum::{
    extract::{Multipart, State, Extension},
    http::StatusCode,
    response::Json,
    routing::{get, put, post},
    Router,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::common::responses::ApiResponse;
use crate::database::models::{DbGlobalSettings, UpdateGlobalSettings};
use crate::database::queries::GlobalSettingsQueries;
use crate::database::queries::UserQueries;
use crate::models::user::User;
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

#[utoipa::path(
    post,
    path = "/api/settings/upload-logo",
    responses(
        (status = 200, description = "Logo uploaded successfully", body = ApiResponse<String>),
        (status = 401, description = "Unauthorized", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>)
    ),
    tag = "settings"
)]
pub async fn upload_logo(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    mut multipart: Multipart,
) -> (StatusCode, Json<ApiResponse<String>>) {
    use tokio::io::AsyncWriteExt;

    let pool = &state.lock().await.db_pool;
    let user_id_i32 = user_id as i32;

    // Check user subscription status
    match UserQueries::get_user_by_id(pool, user_id).await {
        Ok(Some(db_user)) => {
            let user = User::from(db_user);
            if user.subscription_status != "premium" {
                return ApiResponse::bad_request("Logo upload is only available for premium users. Please upgrade your plan to use this feature.".to_string());
            }
        }
        Ok(None) => return ApiResponse::unauthorized("User not found".to_string()),
        Err(e) => return ApiResponse::internal_error(format!("Failed to verify user: {}", e)),
    }

    // Create uploads/logos directory if it doesn't exist
    let logos_dir = std::path::Path::new("uploads/logos");
    if !logos_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(logos_dir) {
            return ApiResponse::internal_error(format!("Failed to create logos directory: {}", e));
        }
    }

    let mut filename = None;
    let mut data = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        if name == "logo" {
            filename = field.file_name().map(|f| f.to_string());
            data = field.bytes().await.unwrap_or_default().to_vec();
        }
    }

    if data.is_empty() {
        return ApiResponse::bad_request("No logo file provided".to_string());
    }

    let filename = filename.unwrap_or_else(|| format!("logo_{}.png", user_id));
    let filepath = logos_dir.join(&filename);

    // Write file
    match tokio::fs::File::create(&filepath).await {
        Ok(mut file) => {
            if let Err(e) = file.write_all(&data).await {
                return ApiResponse::internal_error(format!("Failed to write logo file: {}", e));
            }
        }
        Err(e) => {
            return ApiResponse::internal_error(format!("Failed to create logo file: {}", e));
        }
    }

    // Update user settings with logo_url
    let logo_url = format!("/uploads/logos/{}", filename);
    let update_data = UpdateGlobalSettings {
        logo_url: Some(logo_url.clone()),
        ..Default::default()
    };

    match GlobalSettingsQueries::update_user_settings(pool, user_id as i32, update_data).await {
        Ok(_) => ApiResponse::success(logo_url, "Logo uploaded successfully".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to update user settings: {}", e)),
    }
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/settings/user", get(get_user_settings).put(update_user_settings))
        .route("/settings/upload-logo", post(upload_logo))
}