use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, put, post},
    Router,
    Extension,
    middleware,
};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::common::responses::ApiResponse;
use crate::models::submitter::{Submitter, UpdateSubmitterRequest};
use crate::database::connection::DbPool;
use crate::database::queries::SubmitterQueries;
use crate::common::jwt::auth_middleware;

pub type AppState = Arc<Mutex<DbPool>>;

#[utoipa::path(
    get,
    path = "/api/submitters",
    responses(
        (status = 200, description = "List all submitters", body = ApiResponse<Vec<Submitter>>),
        (status = 500, description = "Internal server error", body = ApiResponse<Vec<Submitter>>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_submitters(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> (StatusCode, Json<ApiResponse<Vec<Submitter>>>) {
    let pool = &*state.lock().await;

    // Get submissions for this user first
    match crate::database::queries::SubmissionQueries::get_submissions_by_user(pool, user_id).await {
        Ok(db_submissions) => {
            let mut all_submitters = Vec::new();
            
            // For each submission, get its submitters
            for db_sub in db_submissions {
                if let Ok(submitters) = SubmitterQueries::get_submitters_by_submission(pool, db_sub.id).await {
                    for db_submitter in submitters {
                        let submitter = Submitter {
                            id: Some(db_submitter.id),
                            submission_id: Some(db_submitter.submission_id),
                            name: db_submitter.name,
                            email: db_submitter.email,
                            status: db_submitter.status,
                            signed_at: db_submitter.signed_at,
                            token: db_submitter.token,
                            fields_data: db_submitter.fields_data,
                            created_at: db_submitter.created_at,
                            updated_at: db_submitter.updated_at,
                        };
                        all_submitters.push(submitter);
                    }
                }
            }
            
            ApiResponse::success(all_submitters, "Submitters retrieved successfully".to_string())
        }
        Err(e) => ApiResponse::internal_error(format!("Failed to get submitters: {}", e)),
    }
}

#[utoipa::path(
    get,
    path = "/api/submitters/{id}",
    params(
        ("id" = i64, Path, description = "Submitter ID")
    ),
    responses(
        (status = 200, description = "Submitter retrieved successfully", body = ApiResponse<Submitter>),
        (status = 404, description = "Submitter not found", body = ApiResponse<Submitter>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_submitter(
    State(state): State<AppState>,
    Path(submitter_id): Path<i64>,
) -> (StatusCode, Json<ApiResponse<Submitter>>) {
    let pool = &*state.lock().await;

    // This is a simplified version - in real app, check permissions
    // For now, assume public access with token
    ApiResponse::not_found("Submitter not found".to_string())
}

#[utoipa::path(
    put,
    path = "/api/submitters/{id}",
    params(
        ("id" = i64, Path, description = "Submitter ID")
    ),
    request_body = UpdateSubmitterRequest,
    responses(
        (status = 200, description = "Submitter updated successfully", body = ApiResponse<Submitter>),
        (status = 404, description = "Submitter not found", body = ApiResponse<Submitter>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_submitter(
    State(state): State<AppState>,
    Path(submitter_id): Path<i64>,
    Json(payload): Json<UpdateSubmitterRequest>,
) -> (StatusCode, Json<ApiResponse<Submitter>>) {
    let pool = &*state.lock().await;

    match SubmitterQueries::update_submitter(pool, submitter_id, payload.status.as_deref(), payload.fields_data.as_ref()).await {
        Ok(Some(db_submitter)) => {
            let submitter = Submitter {
                id: Some(db_submitter.id),
                submission_id: Some(db_submitter.submission_id),
                name: db_submitter.name,
                email: db_submitter.email,
                status: db_submitter.status,
                signed_at: db_submitter.signed_at,
                token: db_submitter.token,
                fields_data: db_submitter.fields_data,
                created_at: db_submitter.created_at,
                updated_at: db_submitter.updated_at,
            };
            ApiResponse::success(submitter, "Submitter updated successfully".to_string())
        }
        Ok(None) => ApiResponse::not_found("Submitter not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to update submitter: {}", e)),
    }
}

// Public endpoint for submitter to access submission
#[utoipa::path(
    get,
    path = "/public/submissions/{token}",
    params(
        ("token" = String, Path, description = "Submitter token")
    ),
    responses(
        (status = 200, description = "Submission data retrieved successfully", body = ApiResponse<serde_json::Value>),
        (status = 404, description = "Submission not found", body = ApiResponse<serde_json::Value>)
    )
)]
pub async fn get_public_submission(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let pool = &*state.lock().await;

    match SubmitterQueries::get_submitter_by_token(pool, &token).await {
        Ok(Some(db_submitter)) => {
            // Get submission details
            match crate::database::queries::SubmissionQueries::get_submission_by_id(pool, db_submitter.submission_id).await {
                Ok(Some(db_submission)) => {
                    // Get template details
                    match crate::database::queries::TemplateQueries::get_template_by_id(pool, db_submission.template_id).await {
                        Ok(Some(db_template)) => {
                            // Convert template to API model
                            let template = crate::routes::templates::convert_db_template_to_template(db_template);
                            
                            // Get all submitters for this submission
                            let submitters = match SubmitterQueries::get_submitters_by_submission(pool, db_submission.id).await {
                                Ok(submitters) => {
                                    submitters.into_iter().map(|db_sub| Submitter {
                                        id: Some(db_sub.id),
                                        submission_id: Some(db_sub.submission_id),
                                        name: db_sub.name,
                                        email: db_sub.email,
                                        status: db_sub.status,
                                        signed_at: db_sub.signed_at,
                                        token: db_sub.token,
                                        fields_data: db_sub.fields_data,
                                        created_at: db_sub.created_at,
                                        updated_at: db_sub.updated_at,
                                    }).collect::<Vec<_>>()
                                }
                                Err(_) => Vec::new(),
                            };

                            // Convert submission to API model
                            let submission = crate::models::submission::Submission {
                                id: db_submission.id,
                                template_id: db_submission.template_id,
                                user_id: db_submission.user_id,
                                status: db_submission.status,
                                documents: None, // TODO: implement documents
                                submitters: Some(submitters),
                                created_at: db_submission.created_at,
                                updated_at: db_submission.updated_at,
                                expires_at: db_submission.expires_at,
                            };

                            // Get signature positions for this submitter
                            let signature_positions = match crate::database::queries::SignatureQueries::get_signature_positions_by_submitter(pool, db_submitter.id).await {
                                Ok(positions) => positions.into_iter().map(|pos| crate::models::signature::SignaturePosition {
                                    id: Some(pos.id),
                                    submitter_id: pos.submitter_id,
                                    field_name: pos.field_name,
                                    page: pos.page,
                                    x: pos.x,
                                    y: pos.y,
                                    width: pos.width,
                                    height: pos.height,
                                    signature_value: pos.signature_value,
                                    signed_at: pos.signed_at,
                                    ip_address: pos.ip_address,
                                    user_agent: pos.user_agent,
                                    version: pos.version,
                                    is_active: pos.is_active,
                                    created_at: pos.created_at,
                                }).collect::<Vec<_>>(),
                                Err(_) => Vec::new(),
                            };

                            // Get signature data for this submitter
                            let signature_data = match crate::database::queries::SignatureQueries::get_signature_data_by_submitter(pool, db_submitter.id).await {
                                Ok(Some(data)) => Some(crate::models::signature::SignatureData {
                                    id: Some(data.id),
                                    submitter_id: data.submitter_id,
                                    signature_value: data.signature_value,
                                    signed_at: Some(data.signed_at),
                                    ip_address: data.ip_address,
                                    user_agent: data.user_agent,
                                }),
                                _ => None,
                            };

                            let data = serde_json::json!({
                                "submitter": Submitter {
                                    id: Some(db_submitter.id),
                                    submission_id: Some(db_submitter.submission_id),
                                    name: db_submitter.name.clone(),
                                    email: db_submitter.email.clone(),
                                    status: db_submitter.status.clone(),
                                    signed_at: db_submitter.signed_at,
                                    token: db_submitter.token.clone(),
                                    fields_data: db_submitter.fields_data.clone(),
                                    created_at: db_submitter.created_at,
                                    updated_at: db_submitter.updated_at,
                                },
                                "submission": submission,
                                "template": template,
                                "signature_positions": signature_positions,
                                "signature_data": signature_data,
                                "message": "Submission data for signing"
                            });
                            
                            ApiResponse::success(data, "Submission accessed successfully".to_string())
                        }
                        _ => ApiResponse::not_found("Template not found".to_string()),
                    }
                }
                _ => ApiResponse::not_found("Submission not found".to_string()),
            }
        }
        Ok(None) => ApiResponse::not_found("Invalid token".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to access submission: {}", e)),
    }
}

pub fn create_submitter_router() -> Router<AppState> {
    Router::new()
        .route("/submitters", get(get_submitters))
        .route("/submitters/:id", get(get_submitter))
        .route("/submitters/:id", put(update_submitter))
        .route("/signatures/positions", post(submit_signature_position))
        .route("/signatures", post(submit_signature))
        .layer(middleware::from_fn(auth_middleware))
}

#[utoipa::path(
    post,
    path = "/api/signatures/positions",
    request_body = crate::models::signature::CreateSignaturePosition,
    responses(
        (status = 201, description = "Signature position submitted successfully", body = ApiResponse<crate::models::signature::SignaturePosition>),
        (status = 400, description = "Bad request", body = ApiResponse<crate::models::signature::SignaturePosition>),
        (status = 404, description = "Submitter not found", body = ApiResponse<crate::models::signature::SignaturePosition>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn submit_signature_position(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<crate::models::signature::CreateSignaturePosition>,
) -> (StatusCode, Json<ApiResponse<crate::models::signature::SignaturePosition>>) {
    let pool = &*state.lock().await;

    // Verify submitter belongs to user
    match SubmitterQueries::get_submitter_by_id(pool, payload.submitter_id).await {
        Ok(Some(db_submitter)) => {
            // Check if submitter belongs to user's submission
            match crate::database::queries::SubmissionQueries::get_submission_by_id(pool, db_submitter.submission_id).await {
                Ok(Some(db_submission)) => {
                    if db_submission.user_id != user_id {
                        return ApiResponse::forbidden("Access denied".to_string());
                    }

                    // Create signature position
                    match crate::database::queries::SignatureQueries::create_signature_position(pool, payload).await {
                        Ok(db_position) => {
                            let position = crate::models::signature::SignaturePosition {
                                id: Some(db_position.id),
                                submitter_id: db_position.submitter_id,
                                field_name: db_position.field_name,
                                page: db_position.page,
                                x: db_position.x,
                                y: db_position.y,
                                width: db_position.width,
                                height: db_position.height,
                                signature_value: db_position.signature_value,
                                signed_at: db_position.signed_at,
                                ip_address: db_position.ip_address,
                                user_agent: db_position.user_agent,
                                version: db_position.version,
                                is_active: db_position.is_active,
                                created_at: db_position.created_at,
                            };
                            ApiResponse::success(position, "Signature position submitted successfully".to_string())
                        }
                        Err(e) => ApiResponse::internal_error(format!("Failed to submit signature position: {}", e)),
                    }
                }
                _ => ApiResponse::not_found("Submission not found".to_string()),
            }
        }
        _ => ApiResponse::not_found("Submitter not found".to_string()),
    }
}

#[utoipa::path(
    post,
    path = "/api/signatures",
    request_body = crate::models::signature::CreateSignatureData,
    responses(
        (status = 201, description = "Signature submitted successfully", body = ApiResponse<crate::models::signature::SignatureData>),
        (status = 400, description = "Bad request", body = ApiResponse<crate::models::signature::SignatureData>),
        (status = 404, description = "Submitter not found", body = ApiResponse<crate::models::signature::SignatureData>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn submit_signature(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<crate::models::signature::CreateSignatureData>,
) -> (StatusCode, Json<ApiResponse<crate::models::signature::SignatureData>>) {
    let pool = &*state.lock().await;

    // Verify submitter belongs to user
    match SubmitterQueries::get_submitter_by_id(pool, payload.submitter_id).await {
        Ok(Some(db_submitter)) => {
            // Check if submitter belongs to user's submission
            match crate::database::queries::SubmissionQueries::get_submission_by_id(pool, db_submitter.submission_id).await {
                Ok(Some(db_submission)) => {
                    if db_submission.user_id != user_id {
                        return ApiResponse::forbidden("Access denied".to_string());
                    }

                    // Create signature data
                    match crate::database::queries::SignatureQueries::create_signature_data(pool, payload).await {
                        Ok(db_signature) => {
                            let signature = crate::models::signature::SignatureData {
                                id: Some(db_signature.id),
                                submitter_id: db_signature.submitter_id,
                                signature_value: db_signature.signature_value,
                                signed_at: Some(db_signature.signed_at),
                                ip_address: db_signature.ip_address,
                                user_agent: db_signature.user_agent,
                            };
                            ApiResponse::success(signature, "Signature submitted successfully".to_string())
                        }
                        Err(e) => ApiResponse::internal_error(format!("Failed to submit signature: {}", e)),
                    }
                }
                _ => ApiResponse::not_found("Submission not found".to_string()),
            }
        }
        _ => ApiResponse::not_found("Submitter not found".to_string()),
    }
}

#[utoipa::path(
    post,
    path = "/public/signatures/positions/{token}",
    params(
        ("token" = String, Path, description = "Submitter token")
    ),
    request_body = crate::models::signature::CreateSignaturePosition,
    responses(
        (status = 201, description = "Signature position submitted successfully", body = ApiResponse<crate::models::signature::SignaturePosition>),
        (status = 400, description = "Bad request", body = ApiResponse<crate::models::signature::SignaturePosition>),
        (status = 404, description = "Submitter not found", body = ApiResponse<crate::models::signature::SignaturePosition>)
    )
)]
pub async fn submit_public_signature_position(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Json(payload): Json<crate::models::signature::PublicCreateSignaturePosition>,
) -> (StatusCode, Json<ApiResponse<crate::models::signature::SignaturePosition>>) {
    let pool = &*state.lock().await;

    // Get submitter by token
    match SubmitterQueries::get_submitter_by_token(pool, &token).await {
        Ok(Some(db_submitter)) => {
            // Create full payload
            let full_payload = crate::models::signature::CreateSignaturePosition {
                submitter_id: db_submitter.id,
                field_name: payload.field_name,
                page: payload.page,
                x: payload.x,
                y: payload.y,
                width: payload.width,
                height: payload.height,
                signature_value: payload.signature_value,
                ip_address: payload.ip_address,
                user_agent: payload.user_agent,
                version: Some(1),
            };

            // Create signature position
            match crate::database::queries::SignatureQueries::create_signature_position(pool, full_payload).await {
                Ok(db_position) => {
                    // Update submitter status to completed after successful signature
                    match SubmitterQueries::update_submitter(pool, db_submitter.id, Some("completed"), None).await {
                        Ok(_) => {
                            let position = crate::models::signature::SignaturePosition {
                                id: Some(db_position.id),
                                submitter_id: db_position.submitter_id,
                                field_name: db_position.field_name,
                                page: db_position.page,
                                x: db_position.x,
                                y: db_position.y,
                                width: db_position.width,
                                height: db_position.height,
                                signature_value: db_position.signature_value,
                                signed_at: db_position.signed_at,
                                ip_address: db_position.ip_address,
                                user_agent: db_position.user_agent,
                                version: db_position.version,
                                is_active: db_position.is_active,
                                created_at: db_position.created_at,
                            };
                            ApiResponse::success(position, "Signature submitted successfully - document completed".to_string())
                        }
                        Err(e) => {
                            // If status update fails, still return success for signature but log error
                            let position = crate::models::signature::SignaturePosition {
                                id: Some(db_position.id),
                                submitter_id: db_position.submitter_id,
                                field_name: db_position.field_name,
                                page: db_position.page,
                                x: db_position.x,
                                y: db_position.y,
                                width: db_position.width,
                                height: db_position.height,
                                signature_value: db_position.signature_value,
                                signed_at: db_position.signed_at,
                                ip_address: db_position.ip_address,
                                user_agent: db_position.user_agent,
                                version: db_position.version,
                                is_active: db_position.is_active,
                                created_at: db_position.created_at,
                            };
                            ApiResponse::success(position, "Signature position submitted successfully".to_string())
                        }
                    }
                }
                Err(e) => ApiResponse::internal_error(format!("Failed to submit signature position: {}", e)),
            }
        }
        Ok(None) => ApiResponse::not_found("Submitter not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Database error: {}", e)),
    }
}

#[utoipa::path(
    get,
    path = "/public/signatures/history/{token}",
    params(
        ("token" = String, Path, description = "Submitter token")
    ),
    responses(
        (status = 200, description = "Signature history retrieved successfully", body = ApiResponse<Vec<crate::models::signature::SignaturePosition>>),
        (status = 404, description = "Submitter not found", body = ApiResponse<Vec<crate::models::signature::SignaturePosition>>)
    )
)]
pub async fn get_signature_history(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> (StatusCode, Json<ApiResponse<Vec<crate::models::signature::SignaturePosition>>>) {
    let pool = &*state.lock().await;

    // Get submitter by token
    match SubmitterQueries::get_submitter_by_token(pool, &token).await {
        Ok(Some(db_submitter)) => {
            // Get signature positions for this submitter
            match crate::database::queries::SignatureQueries::get_signature_positions_by_submitter(pool, db_submitter.id).await {
                Ok(positions) => {
                    let signature_positions = positions.into_iter().map(|pos| crate::models::signature::SignaturePosition {
                        id: Some(pos.id),
                        submitter_id: pos.submitter_id,
                        field_name: pos.field_name,
                        page: pos.page,
                        x: pos.x,
                        y: pos.y,
                        width: pos.width,
                        height: pos.height,
                        signature_value: pos.signature_value,
                        signed_at: pos.signed_at,
                        ip_address: pos.ip_address,
                        user_agent: pos.user_agent,
                        version: pos.version,
                        is_active: pos.is_active,
                        created_at: pos.created_at,
                    }).collect::<Vec<_>>();
                    ApiResponse::success(signature_positions, "Signature history retrieved successfully".to_string())
                }
                Err(e) => ApiResponse::internal_error(format!("Failed to get signature history: {}", e)),
            }
        }
        Ok(None) => ApiResponse::not_found("Invalid token".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Database error: {}", e)),
    }
}

#[utoipa::path(
    put,
    path = "/public/submitters/{token}",
    params(
        ("token" = String, Path, description = "Submitter token")
    ),
    request_body = UpdateSubmitterRequest,
    responses(
        (status = 200, description = "Submitter updated successfully", body = ApiResponse<Submitter>),
        (status = 404, description = "Submitter not found", body = ApiResponse<Submitter>)
    )
)]
pub async fn update_public_submitter(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Json(payload): Json<UpdateSubmitterRequest>,
) -> (StatusCode, Json<ApiResponse<Submitter>>) {
    let pool = &*state.lock().await;

    // Get submitter by token
    match SubmitterQueries::get_submitter_by_token(pool, &token).await {
        Ok(Some(db_submitter)) => {
            // Update submitter
            match SubmitterQueries::update_submitter(pool, db_submitter.id, payload.status.as_deref(), payload.fields_data.as_ref()).await {
                Ok(Some(updated_submitter)) => {
                    let submitter = Submitter {
                        id: Some(updated_submitter.id),
                        submission_id: Some(updated_submitter.submission_id),
                        name: updated_submitter.name,
                        email: updated_submitter.email,
                        status: updated_submitter.status,
                        signed_at: updated_submitter.signed_at,
                        token: updated_submitter.token,
                        fields_data: updated_submitter.fields_data,
                        created_at: updated_submitter.created_at,
                        updated_at: updated_submitter.updated_at,
                    };
                    ApiResponse::success(submitter, "Submitter updated successfully".to_string())
                }
                Ok(None) => ApiResponse::not_found("Submitter not found".to_string()),
                Err(e) => ApiResponse::internal_error(format!("Failed to update submitter: {}", e)),
            }
        }
        Ok(None) => ApiResponse::not_found("Invalid token".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Database error: {}", e)),
    }
}