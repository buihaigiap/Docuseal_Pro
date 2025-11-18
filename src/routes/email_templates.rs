use axum::{
    extract::{State, Extension},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use crate::common::responses::ApiResponse;
use crate::database::queries::EmailTemplateQueries;
use crate::database::models::{CreateEmailTemplate, UpdateEmailTemplate};
use crate::routes::web::AppState;
use crate::models::email_template::EmailTemplate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EmailTemplateResponse {
    pub id: i64,
    pub user_id: i64,
    pub template_type: String,
    pub subject: String,
    pub body: String,
    pub body_format: String,
    pub is_default: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<EmailTemplate> for EmailTemplateResponse {
    fn from(template: EmailTemplate) -> Self {
        EmailTemplateResponse {
            id: template.id,
            user_id: template.user_id,
            template_type: template.template_type,
            subject: template.subject,
            body: template.body,
            body_format: template.body_format,
            is_default: template.is_default,
            created_at: template.created_at,
            updated_at: template.updated_at,
        }
    }
}

/// Get all email templates for the current user
#[utoipa::path(
    get,
    path = "/api/email-templates",
    responses(
        (status = 200, description = "Email templates retrieved successfully", body = ApiResponse<Vec<EmailTemplateResponse>>),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_email_templates(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> (StatusCode, Json<ApiResponse<Vec<EmailTemplateResponse>>>) {
    let pool = &state.lock().await.db_pool;

    match EmailTemplateQueries::get_templates_by_user(pool, user_id).await {
        Ok(templates) => {
            let response: Vec<EmailTemplateResponse> = templates.into_iter()
                .map(|t| EmailTemplate::from(t).into())
                .collect();
            ApiResponse::success(response, "Email templates retrieved successfully".to_string())
        }
        Err(e) => ApiResponse::internal_error(format!("Failed to get email templates: {}", e)),
    }
}

/// Get a specific email template by ID
#[utoipa::path(
    get,
    path = "/api/email-templates/{id}",
    responses(
        (status = 200, description = "Email template retrieved successfully", body = ApiResponse<EmailTemplateResponse>),
        (status = 404, description = "Email template not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_email_template(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> (StatusCode, Json<ApiResponse<EmailTemplateResponse>>) {
    let pool = &state.lock().await.db_pool;

    match EmailTemplateQueries::get_template_by_id(pool, id, user_id).await {
        Ok(Some(template)) => {
            let response = EmailTemplateResponse::from(EmailTemplate::from(template));
            ApiResponse::success(response, "Email template retrieved successfully".to_string())
        }
        Ok(None) => ApiResponse::not_found("Email template not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to get email template: {}", e)),
    }
}

/// Create a new email template
#[utoipa::path(
    post,
    path = "/api/email-templates",
    request_body = CreateEmailTemplate,
    responses(
        (status = 201, description = "Email template created successfully", body = ApiResponse<EmailTemplateResponse>),
        (status = 400, description = "Invalid input"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_email_template(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<CreateEmailTemplate>,
) -> (StatusCode, Json<ApiResponse<EmailTemplateResponse>>) {
    let pool = &state.lock().await.db_pool;

    // Validate input
    if payload.subject.trim().is_empty() {
        return ApiResponse::bad_request("Subject is required".to_string());
    }

    match EmailTemplateQueries::create_template(pool, payload, user_id).await {
        Ok(template) => {
            let response = EmailTemplateResponse::from(EmailTemplate::from(template));
            ApiResponse::created(response, "Email template created successfully".to_string())
        }
        Err(e) => ApiResponse::internal_error(format!("Failed to create email template: {}", e)),
    }
}

/// Update an existing email template
#[utoipa::path(
    put,
    path = "/api/email-templates/{id}",
    request_body = UpdateEmailTemplate,
    responses(
        (status = 200, description = "Email template updated successfully", body = ApiResponse<EmailTemplateResponse>),
        (status = 404, description = "Email template not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_email_template(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    axum::extract::Path(id): axum::extract::Path<i64>,
    Json(payload): Json<UpdateEmailTemplate>,
) -> (StatusCode, Json<ApiResponse<EmailTemplateResponse>>) {
    let pool = &state.lock().await.db_pool;

    match EmailTemplateQueries::update_template(pool, id, user_id, payload).await {
        Ok(Some(template)) => {
            let response = EmailTemplateResponse::from(EmailTemplate::from(template));
            ApiResponse::success(response, "Email template updated successfully".to_string())
        }
        Ok(None) => ApiResponse::not_found("Email template not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to update email template: {}", e)),
    }
}

/// Delete an email template
#[utoipa::path(
    delete,
    path = "/api/email-templates/{id}",
    responses(
        (status = 200, description = "Email template deleted successfully", body = ApiResponse<String>),
        (status = 404, description = "Email template not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_email_template(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    axum::extract::Path(id): axum::extract::Path<i64>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    let pool = &state.lock().await.db_pool;

    match EmailTemplateQueries::delete_template(pool, id, user_id).await {
        Ok(true) => ApiResponse::success("Email template deleted successfully".to_string(), "Deleted".to_string()),
        Ok(false) => ApiResponse::not_found("Email template not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to delete email template: {}", e)),
    }
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/email-templates", get(get_email_templates))
        .route("/email-templates", post(create_email_template))
        .route("/email-templates/:id", get(get_email_template))
        .route("/email-templates/:id", put(update_email_template))
        .route("/email-templates/:id", delete(delete_email_template))
}