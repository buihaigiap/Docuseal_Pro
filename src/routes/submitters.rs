use axum::{
    extract::{Path, State, Extension},
    http::StatusCode,
    response::Json,
    routing::{get, put, post},
    Router,
    middleware,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::common::responses::ApiResponse;
use crate::database::connection::DbPool;
use crate::database::queries::{SubmitterQueries, TemplateQueries, TemplateFieldQueries, SignatureQueries};
use crate::common::jwt::auth_middleware;
use crate::common::authorization::require_admin_or_team_member;
use crate::common::token::generate_token;
use chrono::Utc;

use crate::routes::web::AppState;

#[utoipa::path(
    get,
    path = "/api/submitters",
    responses(
        (status = 200, description = "Submitters retrieved successfully", body = ApiResponse<Vec<crate::models::submitter::Submitter>>),
        (status = 500, description = "Internal server error", body = ApiResponse<Vec<crate::models::submitter::Submitter>>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_submitters(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> (StatusCode, Json<ApiResponse<Vec<crate::models::submitter::Submitter>>>) {
    let pool = &*state.lock().await;

    // Get submitters directly for this user
    match SubmitterQueries::get_submitters_by_user(pool, user_id).await {
        Ok(db_submitters) => {
            let mut all_submitters = Vec::new();
            
            for db_submitter in db_submitters {
                let submitter = crate::models::submitter::Submitter {
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
                all_submitters.push(submitter);
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
        (status = 200, description = "Submitter retrieved successfully", body = ApiResponse<crate::models::submitter::Submitter>),
        (status = 404, description = "Submitter not found", body = ApiResponse<crate::models::submitter::Submitter>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_submitter(
    State(state): State<AppState>,
    Path(submitter_id): Path<i64>,
) -> (StatusCode, Json<ApiResponse<crate::models::submitter::Submitter>>) {
    let pool = &*state.lock().await;

    match SubmitterQueries::get_submitter_by_id(pool, submitter_id).await {
        Ok(Some(db_submitter)) => {
            let submitter = crate::models::submitter::Submitter {
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
            ApiResponse::success(submitter, "Submitter retrieved successfully".to_string())
        }
        Ok(None) => ApiResponse::not_found("Submitter not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to get submitter: {}", e)),
    }
}

#[utoipa::path(
    put,
    path = "/api/submitters/{id}",
    params(
        ("id" = i64, Path, description = "Submitter ID")
    ),
    request_body = crate::models::submitter::UpdateSubmitterRequest,
    responses(
        (status = 200, description = "Submitter updated successfully", body = ApiResponse<crate::models::submitter::Submitter>),
        (status = 404, description = "Submitter not found", body = ApiResponse<crate::models::submitter::Submitter>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_submitter(
    State(state): State<AppState>,
    Path(submitter_id): Path<i64>,
    Json(payload): Json<crate::models::submitter::UpdateSubmitterRequest>,
) -> (StatusCode, Json<ApiResponse<crate::models::submitter::Submitter>>) {
    let pool = &*state.lock().await;

    match SubmitterQueries::update_submitter(pool, submitter_id, payload.status.as_deref()).await {
        Ok(Some(db_submitter)) => {
            let submitter = crate::models::submitter::Submitter {
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
            ApiResponse::success(submitter, "Submitter updated successfully".to_string())
        }
        Ok(None) => ApiResponse::not_found("Submitter not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to update submitter: {}", e)),
    }
}


#[utoipa::path(
    get,
    path = "/public/submissions/{token}",
    params(
        ("token" = String, Path, description = "Submitter token")
    ),
    responses(
        (status = 200, description = "Submission retrieved successfully", body = ApiResponse<crate::models::submitter::PublicSubmissionResponse>),
        (status = 404, description = "Submitter not found", body = ApiResponse<crate::models::submitter::PublicSubmissionResponse>)
    )
)]
pub async fn get_public_submitter(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> (StatusCode, Json<ApiResponse<crate::models::submitter::PublicSubmissionResponse>>) {
    let pool = &*state.lock().await;

    match SubmitterQueries::get_submitter_by_token(pool, &token).await {
        Ok(Some(db_submitter)) => {
            let submitter = crate::models::submitter::Submitter {
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

            // Get the template
            let template_id = db_submitter.template_id;
            match crate::database::queries::TemplateQueries::get_template_by_id(pool, template_id).await {
                Ok(Some(db_template)) => {
                    match crate::routes::templates::convert_db_template_to_template_with_fields(db_template, pool).await {
                        Ok(template) => {
                            let response = crate::models::submitter::PublicSubmissionResponse {
                                template,
                                submitter,
                            };
                            ApiResponse::success(response, "Submission retrieved successfully".to_string())
                        }
                        Err(e) => ApiResponse::internal_error(format!("Failed to load template: {}", e)),
                    }
                }
                Ok(None) => ApiResponse::not_found("Template not found".to_string()),
                Err(e) => ApiResponse::internal_error(format!("Failed to get template: {}", e)),
            }
        }
        Ok(None) => ApiResponse::not_found("Submitter not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to get submitter: {}", e)),
    }
}

#[utoipa::path(
    put,
    path = "/public/submissions/{token}",
    params(
        ("token" = String, Path, description = "Submitter token")
    ),
    request_body = crate::models::submitter::PublicUpdateSubmitterRequest,
    responses(
        (status = 200, description = "Submitter updated successfully", body = ApiResponse<crate::models::submitter::Submitter>),
        (status = 404, description = "Submitter not found", body = ApiResponse<crate::models::submitter::Submitter>)
    )
)]
pub async fn update_public_submitter(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Json(payload): Json<crate::models::submitter::PublicUpdateSubmitterRequest>,
) -> (StatusCode, Json<ApiResponse<crate::models::submitter::Submitter>>) {
    let pool = &*state.lock().await;

    match SubmitterQueries::get_submitter_by_token(pool, &token).await {
        Ok(Some(db_submitter)) => {
            match SubmitterQueries::update_submitter(pool, db_submitter.id, None).await {
                Ok(Some(updated_submitter)) => {
                    let submitter = crate::models::submitter::Submitter {
                        id: Some(updated_submitter.id),
                        template_id: Some(updated_submitter.template_id),
                        user_id: Some(updated_submitter.user_id),
                        name: updated_submitter.name,
                        email: updated_submitter.email,
                        status: updated_submitter.status,
                        signed_at: updated_submitter.signed_at,
                        token: updated_submitter.token,
                        bulk_signatures: updated_submitter.bulk_signatures,
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
        (status = 200, description = "Bulk signatures submitted successfully", body = ApiResponse<crate::models::submitter::Submitter>),
        (status = 404, description = "Submitter not found", body = ApiResponse<crate::models::submitter::Submitter>)
    )
)]
pub async fn submit_bulk_signatures(
    State(state): State<AppState>,
    Path(token): Path<String>,
    Json(payload): Json<crate::models::signature::BulkSignatureRequest>,
) -> (StatusCode, Json<ApiResponse<crate::models::submitter::Submitter>>) {
    let pool = &*state.lock().await;

    match SubmitterQueries::get_submitter_by_token(pool, &token).await {
        Ok(Some(db_submitter)) => {
            
            // Validate that all field_ids belong to this template
            for signature_item in &payload.signatures {
                match TemplateFieldQueries::get_template_field_by_id(pool, signature_item.field_id).await {
                    Ok(Some(field)) => {
                        if field.template_id != db_submitter.template_id {
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
                let field_name = match TemplateFieldQueries::get_template_field_by_id(pool, field_id).await {
                    Ok(Some(field)) => field.name,
                    Ok(None) => format!("field_{}", field_id),
                    Err(_) => format!("field_{}", field_id),
                };
                signatures_array.push(serde_json::json!({
                    "field_id": field_id,
                    "field_name": field_name,
                    "signature_value": signature_item.signature_value
                }));
            }

            let bulk_signatures = serde_json::Value::Array(signatures_array);

            match SubmitterQueries::update_submitter_with_signatures(
                pool,
                db_submitter.id,
                &bulk_signatures,
                payload.ip_address.as_deref(),
                payload.user_agent.as_deref(),
            ).await {
                Ok(Some(updated_submitter)) => {
                    let submitter = crate::models::submitter::Submitter {
                        id: Some(updated_submitter.id),
                        template_id: Some(updated_submitter.template_id),
                        user_id: Some(updated_submitter.user_id),
                        name: updated_submitter.name,
                        email: updated_submitter.email,
                        status: updated_submitter.status,
                        signed_at: updated_submitter.signed_at,
                        token: updated_submitter.token,
                        bulk_signatures: updated_submitter.bulk_signatures,
                        created_at: updated_submitter.created_at,
                        updated_at: updated_submitter.updated_at,
                    };
                    ApiResponse::success(submitter, "Bulk signatures submitted successfully".to_string())
                }
                Ok(None) => ApiResponse::not_found("Submitter not found".to_string()),
                Err(e) => ApiResponse::internal_error(format!("Failed to save bulk signatures: {}", e)),
            }
        }
        _ => ApiResponse::not_found("Invalid token".to_string()),
    }
}

pub fn create_submitter_router() -> Router<AppState> {
    Router::new()
        .route("/submitters", get(get_submitters))
        .route("/submitters/:id", get(get_submitter))
        .route("/submitters/:id", put(update_submitter))
        .layer(middleware::from_fn(auth_middleware))
        .layer(middleware::from_fn(require_admin_or_team_member))
}
