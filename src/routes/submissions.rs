use axum::{
    extract::{Path, State, Request},
    http::{StatusCode, header},
    response::{Json, Response},
    routing::{get, post, put, delete},
    Router,
    body::Body,
    Extension,
    middleware,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json;
use uuid::Uuid;

use crate::common::responses::ApiResponse;
use crate::models::submission::{Submission, CreateSubmissionRequest, UpdateSubmissionRequest};
use crate::models::submitter::Submitter;
use crate::database::connection::DbPool;
use crate::database::models::{CreateSubmission, CreateSubmitter};
use crate::database::queries::{SubmissionQueries, SubmitterQueries, TemplateQueries};
use crate::routes::templates::convert_db_template_to_template;
use crate::common::jwt::auth_middleware;

pub type AppState = Arc<Mutex<DbPool>>;

#[utoipa::path(
    get,
    path = "/api/submissions",
    responses(
        (status = 200, description = "List all submissions", body = ApiResponse<Vec<Submission>>),
        (status = 500, description = "Internal server error", body = ApiResponse<Vec<Submission>>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_submissions(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> (StatusCode, Json<ApiResponse<Vec<Submission>>>) {
    let pool = &*state.lock().await;

    match SubmissionQueries::get_submissions_by_user(pool, user_id).await {
        Ok(db_submissions) => {
            let mut submissions = Vec::new();
            for db_sub in db_submissions {
                // Get template info
                if let Ok(Some(db_template)) = TemplateQueries::get_template_by_id(pool, db_sub.template_id).await {
                    let template = convert_db_template_to_template(db_template);
                    
                    // Convert to API model
                    let submission = Submission {
                        id: db_sub.id,
                        template_id: db_sub.template_id,
                        user_id: db_sub.user_id,
                        status: db_sub.status,
                        documents: None, // TODO: implement documents
                        submitters: db_sub.submitters.map(|s| serde_json::from_value(s).unwrap_or_default()),
                        created_at: db_sub.created_at,
                        updated_at: db_sub.updated_at,
                        expires_at: db_sub.expires_at,
                    };
                    submissions.push(submission);
                }
            }
            ApiResponse::success(submissions, "Submissions retrieved successfully".to_string())
        }
        Err(e) => ApiResponse::internal_error(format!("Failed to get submissions: {}", e)),
    }
}

#[utoipa::path(
    post,
    path = "/api/submissions",
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

    // Check if template exists
    match TemplateQueries::get_template_by_id(pool, payload.template_id).await {
        Ok(Some(_)) => {
            // Create submission
            let submitters_json = serde_json::to_value(&payload.submitters).unwrap_or(serde_json::Value::Null);
            
            let create_data = CreateSubmission {
                template_id: payload.template_id,
                user_id: user_id,
                status: "pending".to_string(),
                documents: None,
                submitters: Some(submitters_json),
                expires_at: payload.expires_at,
            };

            match SubmissionQueries::create_submission(pool, create_data).await {
                Ok(db_sub) => {
                    // Create submitters
                    for submitter in &payload.submitters {
                        let token = Uuid::new_v4().to_string();
                        let create_submitter = CreateSubmitter {
                            submission_id: db_sub.id,
                            name: submitter.name.clone(),
                            email: submitter.email.clone(),
                            status: "pending".to_string(),
                            token: token.clone(),
                            fields_data: None,
                        };
                        if let Err(e) = SubmitterQueries::create_submitter(pool, create_submitter).await {
                            eprintln!("Failed to create submitter: {}", e);
                        }
                    }

                    // Get the created submitters from database
                    let submitters = match SubmitterQueries::get_submitters_by_submission(pool, db_sub.id).await {
                        Ok(submitters) => submitters.into_iter().map(|s| crate::models::submitter::Submitter {
                            id: Some(s.id),
                            submission_id: Some(s.submission_id),
                            name: s.name,
                            email: s.email,
                            status: s.status,
                            signed_at: s.signed_at,
                            token: s.token,
                            fields_data: s.fields_data,
                            created_at: s.created_at,
                            updated_at: s.updated_at,
                        }).collect(),
                        Err(_) => Vec::new(),
                    };

                    let submission = Submission {
                        id: db_sub.id,
                        template_id: db_sub.template_id,
                        user_id: db_sub.user_id,
                        status: db_sub.status,
                        documents: None,
                        submitters: Some(submitters),
                        created_at: db_sub.created_at,
                        updated_at: db_sub.updated_at,
                        expires_at: db_sub.expires_at,
                    };
                    ApiResponse::success(submission, "Submission created successfully".to_string())
                }
                Err(e) => ApiResponse::internal_error(format!("Failed to create submission: {}", e)),
            }
        }
        _ => ApiResponse::not_found("Template not found".to_string()),
    }
}

#[utoipa::path(
    get,
    path = "/api/submissions/{id}",
    params(
        ("id" = i64, Path, description = "Submission ID")
    ),
    responses(
        (status = 200, description = "Submission retrieved successfully", body = ApiResponse<Submission>),
        (status = 404, description = "Submission not found", body = ApiResponse<Submission>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_submission(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(submission_id): Path<i64>,
) -> (StatusCode, Json<ApiResponse<Submission>>) {
    let pool = &*state.lock().await;

    match SubmissionQueries::get_submission(pool, submission_id, user_id).await {
        Ok(Some(db_sub)) => {
            // Get submitters from database
            let submitters = match SubmitterQueries::get_submitters_by_submission(pool, db_sub.id).await {
                Ok(submitters) => submitters.into_iter().map(|s| crate::models::submitter::Submitter {
                    id: Some(s.id),
                    submission_id: Some(s.submission_id),
                    name: s.name,
                    email: s.email,
                    status: s.status,
                    signed_at: s.signed_at,
                    token: s.token,
                    fields_data: s.fields_data,
                    created_at: s.created_at,
                    updated_at: s.updated_at,
                }).collect(),
                Err(_) => Vec::new(),
            };

            let submission = Submission {
                id: db_sub.id,
                template_id: db_sub.template_id,
                user_id: db_sub.user_id,
                status: db_sub.status,
                documents: None,
                submitters: Some(submitters),
                created_at: db_sub.created_at,
                updated_at: db_sub.updated_at,
                expires_at: db_sub.expires_at,
            };
            ApiResponse::success(submission, "Submission retrieved successfully".to_string())
        }
        Ok(None) => ApiResponse::not_found("Submission not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to get submission: {}", e)),
    }
}

#[utoipa::path(
    put,
    path = "/api/submissions/{id}",
    params(
        ("id" = i64, Path, description = "Submission ID")
    ),
    request_body = UpdateSubmissionRequest,
    responses(
        (status = 200, description = "Submission updated successfully", body = ApiResponse<Submission>),
        (status = 404, description = "Submission not found", body = ApiResponse<Submission>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_submission(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(submission_id): Path<i64>,
    Json(payload): Json<UpdateSubmissionRequest>,
) -> (StatusCode, Json<ApiResponse<Submission>>) {
    let pool = &*state.lock().await;

    let submitters_json = payload.submitters.as_ref().map(|s| serde_json::to_value(s).unwrap_or(serde_json::Value::Null));

    match SubmissionQueries::update_submission(pool, submission_id, user_id, payload.status.as_deref(), submitters_json.as_ref()).await {
        Ok(Some(db_sub)) => {
            let submission = Submission {
                id: db_sub.id,
                template_id: db_sub.template_id,
                user_id: db_sub.user_id,
                status: db_sub.status,
                documents: None,
                submitters: db_sub.submitters.map(|s| serde_json::from_value(s).unwrap_or_default()),
                created_at: db_sub.created_at,
                updated_at: db_sub.updated_at,
                expires_at: db_sub.expires_at,
            };
            ApiResponse::success(submission, "Submission updated successfully".to_string())
        }
        Ok(None) => ApiResponse::not_found("Submission not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to update submission: {}", e)),
    }
}

#[utoipa::path(
    delete,
    path = "/api/submissions/{id}",
    params(
        ("id" = i64, Path, description = "Submission ID")
    ),
    responses(
        (status = 200, description = "Submission deleted successfully", body = ApiResponse<String>),
        (status = 404, description = "Submission not found", body = ApiResponse<String>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_submission(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Path(submission_id): Path<i64>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    let pool = &*state.lock().await;

    match SubmissionQueries::delete_submission(pool, submission_id, user_id).await {
        Ok(true) => ApiResponse::success("Submission deleted successfully".to_string(), "Submission deleted successfully".to_string()),
        Ok(false) => ApiResponse::not_found("Submission not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to delete submission: {}", e)),
    }
}

pub fn create_submission_router() -> Router<AppState> {
    Router::new()
        .route("/submissions", get(get_submissions))
        .route("/submissions", post(create_submission))
        .route("/submissions/:id", get(get_submission))
        .route("/submissions/:id", put(update_submission))
        .route("/submissions/:id", delete(delete_submission))
        .layer(middleware::from_fn(auth_middleware))
}