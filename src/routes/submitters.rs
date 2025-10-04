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
use crate::database::queries::{SubmitterQueries, TemplateFieldQueries};
use crate::common::jwt::auth_middleware;
use crate::common::token::generate_token;

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

    let tokens_secret = std::env::var("TOKENS_SECRET").unwrap_or_else(|_| "default-tokens-secret-change-this".to_string());
    match SubmitterQueries::get_submitter_by_token(pool, &token).await {
        Ok(Some(db_submitter)) => {
            // Get submission details
            match crate::database::queries::SubmissionQueries::get_submission_by_id(pool, db_submitter.submission_id).await {
                Ok(Some(db_submission)) => {
                    // Get template details
                    match crate::database::queries::TemplateQueries::get_template_by_id(pool, db_submission.template_id).await {
                        Ok(Some(db_template)) => {
                            // Convert template to API model with fields
                            let template = match crate::routes::templates::convert_db_template_to_template_with_fields(db_template, pool).await {
                                Ok(t) => t,
                                Err(_) => return ApiResponse::internal_error("Failed to load template fields".to_string()),
                            };
                            
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

                            // DEPRECATED: Old signature positions format, now we use bulk_signatures
                            let mut signature_positions: Vec<crate::models::signature::SignaturePosition> = Vec::new();

                            // Also get bulk signatures from signature_positions table and convert them to signature positions format
                            if let Ok(bulk_signature_positions) = crate::database::queries::SignatureQueries::get_bulk_signature_positions_by_submitter(pool, db_submitter.id).await {
                                for bulk_pos in bulk_signature_positions {
                                    if let Some(bulk_signatures) = bulk_pos.bulk_signatures {
                                        if let Some(signatures_array) = bulk_signatures.as_array() {
                                            for signature_item in signatures_array {
                                                if let Some(obj) = signature_item.as_object() {
                                                    let field_id = obj.get("field_id").and_then(|v| v.as_i64());
                                                    let field_name = obj.get("field_name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                                                    let signature_value = obj.get("signature_value").and_then(|v| v.as_str());

                                                    if let Some(sig_val) = signature_value {
                                                        signature_positions.push(crate::models::signature::SignaturePosition {
                                                            id: Some(bulk_pos.id),
                                                            submitter_id: bulk_pos.submitter_id,
                                                            field_id,
                                                            field_name,
                                                            signature_value: Some(sig_val.to_string()),
                                                            signed_at: bulk_pos.signed_at,
                                                            ip_address: bulk_pos.ip_address.clone(),
                                                            user_agent: bulk_pos.user_agent.clone(),
                                                            version: 1,
                                                            is_active: true,
                                                            created_at: bulk_pos.created_at,
                                                        });
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

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
        .layer(middleware::from_fn(auth_middleware))
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

    let tokens_secret = std::env::var("TOKENS_SECRET").unwrap_or_else(|_| "default-tokens-secret-change-this".to_string());
    // DEPRECATED: Old signature history endpoint, now we use bulk_signatures
    // Return empty array for backward compatibility
    let signature_positions: Vec<crate::models::signature::SignaturePosition> = Vec::new();
    ApiResponse::success(signature_positions, "Signature history endpoint deprecated, use bulk signatures instead".to_string())
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

    let tokens_secret = std::env::var("TOKENS_SECRET").unwrap_or_else(|_| "default-tokens-secret-change-this".to_string());
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

#[utoipa::path(
    post,
    path = "/public/signatures/bulk/{token}",
    params(
        ("token" = String, Path, description = "Submitter token")
    ),
    request_body = crate::models::signature::BulkSignatureRequest,
    responses(
        (status = 201, description = "Bulk signatures submitted successfully", body = ApiResponse<crate::database::models::DbSignaturePosition>),
        (status = 400, description = "Bad request", body = ApiResponse<crate::database::models::DbSignaturePosition>),
        (status = 404, description = "Submitter not found", body = ApiResponse<crate::database::models::DbSignaturePosition>)
    )
)]
pub async fn submit_bulk_signatures(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Json(payload): Json<crate::models::signature::BulkSignatureRequest>,
) -> (StatusCode, Json<ApiResponse<crate::database::models::DbSignaturePosition>>) {
    let pool = &*state.lock().await;

    let tokens_secret = std::env::var("TOKENS_SECRET").unwrap_or_else(|_| "default-tokens-secret-change-this".to_string());

    // Get submitter by token
    match SubmitterQueries::get_submitter_by_token(pool, &token).await {
        Ok(Some(db_submitter)) => {
            // Get submission to validate template
            match crate::database::queries::SubmissionQueries::get_submission_by_id(pool, db_submitter.submission_id).await {
                Ok(Some(db_submission)) => {
                    // Validate that all field_ids belong to this template
                    for signature_item in &payload.signatures {
                        match TemplateFieldQueries::get_template_field_by_id(pool, signature_item.field_id).await {
                            Ok(Some(field)) => {
                                if field.template_id != db_submission.template_id {
                                    return ApiResponse::bad_request(format!("Field {} does not belong to this template", signature_item.field_id));
                                }
                            }
                            Ok(None) => return ApiResponse::bad_request(format!("Field {} not found", signature_item.field_id)),
                            Err(e) => return ApiResponse::internal_error(format!("Failed to validate field {}: {}", signature_item.field_id, e)),
                        }
                    }

                    // Create signatures array with field details
                    let mut signatures_array = Vec::new();
                    for signature_item in &payload.signatures {
                        let field_id = signature_item.field_id;

                        // Get field name from template_fields
                        let field_name = match TemplateFieldQueries::get_template_field_by_id(pool, field_id).await {
                            Ok(Some(field)) => field.name,
                            Ok(None) => format!("field_{}", field_id), // Fallback if field not found
                            Err(_) => format!("field_{}", field_id), // Fallback on error
                        };

                        let signature_obj = serde_json::json!({
                            "field_id": field_id,
                            "field_name": field_name,
                            "signature_value": signature_item.signature_value
                        });
                        signatures_array.push(signature_obj);
                    }

                    let signatures_json = serde_json::Value::Array(signatures_array);

                    // Create bulk signature record in signature_positions table
                    match crate::database::queries::SignatureQueries::create_bulk_signature_position(
                        pool,
                        db_submitter.id,
                        signatures_json,
                        payload.ip_address.as_deref(),
                        payload.user_agent.as_deref(),
                    ).await {
                        Ok(db_bulk_signature) => {
                            // Update submitter status to completed
                            match SubmitterQueries::update_submitter(pool, db_submitter.id, Some("completed"), None).await {
                                Ok(_) => {
                                    ApiResponse::success(db_bulk_signature, "Bulk signatures submitted successfully".to_string())
                                }
                                Err(e) => ApiResponse::internal_error(format!("Signatures saved but failed to update status: {}", e)),
                            }
                        }
                        Err(e) => ApiResponse::internal_error(format!("Failed to save bulk signatures: {}", e)),
                    }
                }
                Ok(None) => ApiResponse::not_found("Submission not found".to_string()),
                Err(e) => ApiResponse::internal_error(format!("Failed to get submission: {}", e)),
            }
        }
        Ok(None) => ApiResponse::not_found("Invalid token".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Database error: {}", e)),
    }
}