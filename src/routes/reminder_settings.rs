use axum::{
    extract::{State, Extension},
    http::StatusCode,
    response::Json,
    routing::{get, put},
    Router,
};
use crate::common::responses::ApiResponse;
use crate::database::queries::UserReminderSettingsQueries;
use crate::database::models::{UpdateUserReminderSettings, DbUserReminderSettings};
use crate::routes::web::AppState;
use crate::constants::{is_valid_reminder_duration, REMINDER_DURATIONS};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserReminderSettingsResponse {
    pub first_reminder_hours: Option<i32>,
    pub second_reminder_hours: Option<i32>,
    pub third_reminder_hours: Option<i32>,
    /// Reminders are enabled when all 3 hours are configured (non-NULL)
    pub enabled: bool,
}

impl From<DbUserReminderSettings> for UserReminderSettingsResponse {
    fn from(db: DbUserReminderSettings) -> Self {
        // Auto-enable if all 3 hours are set
        let enabled = db.first_reminder_hours.is_some() 
            && db.second_reminder_hours.is_some() 
            && db.third_reminder_hours.is_some();
        
        Self {
            first_reminder_hours: db.first_reminder_hours,
            second_reminder_hours: db.second_reminder_hours,
            third_reminder_hours: db.third_reminder_hours,
            enabled,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateReminderSettingsRequest {
    pub first_reminder_hours: Option<i32>,
    pub second_reminder_hours: Option<i32>,
    pub third_reminder_hours: Option<i32>,
}

/// Get current user's reminder settings
#[utoipa::path(
    get,
    path = "/api/reminder-settings",
    responses(
        (status = 200, description = "Reminder settings retrieved successfully", body = ApiResponse<UserReminderSettingsResponse>),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_reminder_settings(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> (StatusCode, Json<ApiResponse<UserReminderSettingsResponse>>) {
    let pool = &state.lock().await.db_pool;

    match UserReminderSettingsQueries::get_or_create_default(pool, user_id).await {
        Ok(settings) => {
            let response = UserReminderSettingsResponse::from(settings);
            ApiResponse::success(response, "Reminder settings retrieved successfully".to_string())
        }
        Err(e) => ApiResponse::internal_error(format!("Failed to get reminder settings: {}", e)),
    }
}

/// Update current user's reminder settings
/// Reminders are automatically enabled when all 3 hours are configured (non-NULL)
#[utoipa::path(
    put,
    path = "/api/reminder-settings",
    request_body = UpdateReminderSettingsRequest,
    responses(
        (status = 200, description = "Reminder settings updated successfully", body = ApiResponse<UserReminderSettingsResponse>),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_reminder_settings(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<UpdateReminderSettingsRequest>,
) -> (StatusCode, Json<ApiResponse<UserReminderSettingsResponse>>) {
    let pool = &state.lock().await.db_pool;

    // Build valid durations list for error message
    let valid_durations: Vec<String> = REMINDER_DURATIONS
        .iter()
        .map(|(hours, label)| format!("{} ({})", hours, label))
        .collect();
    let valid_options = valid_durations.join(", ");

    // Validation: hours must be one of the predefined durations
    if let Some(hours) = payload.first_reminder_hours {
        if !is_valid_reminder_duration(hours) {
            return ApiResponse::bad_request(
                format!("Invalid first reminder duration. Valid options: {}", valid_options)
            );
        }
    }
    if let Some(hours) = payload.second_reminder_hours {
        if !is_valid_reminder_duration(hours) {
            return ApiResponse::bad_request(
                format!("Invalid second reminder duration. Valid options: {}", valid_options)
            );
        }
    }
    if let Some(hours) = payload.third_reminder_hours {
        if !is_valid_reminder_duration(hours) {
            return ApiResponse::bad_request(
                format!("Invalid third reminder duration. Valid options: {}", valid_options)
            );
        }
    }

    // Ensure user has settings record first
    if let Err(e) = UserReminderSettingsQueries::get_or_create_default(pool, user_id).await {
        return ApiResponse::internal_error(format!("Failed to initialize reminder settings: {}", e));
    }

    let update_data = UpdateUserReminderSettings {
        first_reminder_hours: payload.first_reminder_hours,
        second_reminder_hours: payload.second_reminder_hours,
        third_reminder_hours: payload.third_reminder_hours,
    };

    match UserReminderSettingsQueries::update(pool, user_id, update_data).await {
        Ok(Some(settings)) => {
            let response = UserReminderSettingsResponse::from(settings);
            ApiResponse::success(response, "Reminder settings updated successfully".to_string())
        }
        Ok(None) => ApiResponse::not_found("Reminder settings not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to update reminder settings: {}", e)),
    }
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/reminder-settings", get(get_reminder_settings))
        .route("/reminder-settings", put(update_reminder_settings))
}
