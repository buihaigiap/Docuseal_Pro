use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::post,
    Router,
    Extension,
    middleware,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::common::token::generate_token;

use crate::common::responses::ApiResponse;
use crate::models::submission::{Submission, CreateSubmissionRequest};
use crate::models::submitter::Submitter;
use crate::database::connection::DbPool;
use crate::database::models::CreateSubmitter;
use crate::database::queries::{SubmitterQueries, TemplateQueries};
// use crate::routes::subscription::{can_user_submit, increment_usage_count};
use crate::routes::templates::convert_db_template_to_template;
use crate::common::jwt::auth_middleware;
use crate::common::authorization::require_admin_or_team_member;
use crate::services::email::EmailService;

pub type AppState = Arc<Mutex<DbPool>>;

#[utoipa::path(
    post,
    path = "/api/submissions",
    tag = "submissions",
    request_body = CreateSubmissionRequest,
    responses(
        (status = 201, description = "Submission created successfully", body = ApiResponse<Submission>),
        (status = 400, description = "Bad request", body = ApiResponse<Submission>),
        (status = 404, description = "Template not found", body = ApiResponse<Submission>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_submission(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<CreateSubmissionRequest>,
) -> (StatusCode, Json<ApiResponse<Submission>>) {

    let pool = &*state.lock().await;

    // TODO: 🔒 Check if user can submit (usage limit check) - disabled for now
    /*
    match can_user_submit(pool, user_id).await {
        Ok(false) => {
            return (
                StatusCode::FORBIDDEN,
                Json(ApiResponse {
                    success: false,
                    message: "Usage limit exceeded. Please upgrade to premium to continue submitting documents.".to_string(),
                    data: None,
                }),
            );
        },
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse {
                    success: false,
                    message: format!("Failed to check usage limits: {}", e),
                    data: None,
                }),
            );
        },
        Ok(true) => {
            // User can submit, continue
        }
    }
    */

    // Check if template exists
    match TemplateQueries::get_template_by_id(pool, payload.template_id).await {
        Ok(Some(db_template)) => {
            // In merged schema, we create submitters directly without a separate submission record
            let mut created_submitters = Vec::new();

            for submitter in &payload.submitters {
                let token = generate_token();
                let create_submitter = CreateSubmitter {
                    template_id: payload.template_id,
                    user_id: user_id,
                    name: submitter.name.clone(),
                    email: submitter.email.clone(),
                    status: "pending".to_string(),
                    token: token.clone(),
                };

                match SubmitterQueries::create_submitter(pool, create_submitter).await {
                    Ok(db_submitter) => {
                        let submitter_api = Submitter {
                            id: Some(db_submitter.id),
                            template_id: Some(db_submitter.template_id),
                            user_id: Some(db_submitter.user_id),
                            name: db_submitter.name,
                            email: db_submitter.email,
                            status: db_submitter.status,
                            signed_at: db_submitter.signed_at,
                            token: db_submitter.token,
                            bulk_signatures: db_submitter.bulk_signatures,
                            created_at: db_submitter.created_at,
                            updated_at: db_submitter.updated_at,
                        };
                        created_submitters.push(submitter_api);

                        // Send email to submitter
                        let template = convert_db_template_to_template(db_template.clone());
                        if let Ok(email_service) = EmailService::new() {
                            if let Err(e) = email_service.send_signature_request(
                                &submitter.email,
                                &submitter.name,
                                &template.name,
                                &token,
                            ).await {
                                eprintln!("Failed to send email to {}: {}", submitter.email, e);
                            }
                        }
                    }
                    Err(e) => {
                        return ApiResponse::internal_error(format!("Failed to create submitter: {}", e));
                    }
                }
            }

            // Create synthetic submission response
            let submission = Submission {
                id: payload.template_id, // Use template_id as submission id
                template_id: payload.template_id,
                user_id: user_id,
                status: "active".to_string(),
                documents: None,
                submitters: Some(created_submitters),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
                expires_at: payload.expires_at,
            };

            // TODO: 📊 Increment usage count for free users after successful submission - disabled for now
            /*
            if let Err(e) = increment_usage_count(pool, user_id).await {
                eprintln!("Warning: Failed to increment usage count for user {}: {}", user_id, e);
                // Don't fail the request, just log the warning
            }
            */

            ApiResponse::success(submission, "Submission created successfully".to_string())
        }
        Ok(None) => ApiResponse::not_found("Template not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Database error: {}", e)),
    }
}

pub fn create_submission_router() -> Router<AppState> {
    Router::new()
        .route("/submissions", post(create_submission))
        .layer(middleware::from_fn(auth_middleware))
}
