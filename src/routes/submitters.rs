use axum::{
    extract::{Path, State, Extension},
    http::StatusCode,
    response::Json,
    routing::{get, put, delete},
    Router,
    middleware,
};
use crate::common::responses::ApiResponse;
use crate::database::queries::{SubmitterQueries, TemplateFieldQueries, UserQueries};
use crate::common::jwt::auth_middleware;
use crate::common::authorization::require_admin_or_team_member;

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
    get,
    path = "/api/me",
    responses(
        (status = 200, description = "Current user retrieved successfully", body = ApiResponse<crate::models::user::User>),
        (status = 404, description = "User not found", body = ApiResponse<crate::models::user::User>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_me(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> (StatusCode, Json<ApiResponse<crate::models::user::User>>) {
    let pool = &*state.lock().await;

    match UserQueries::get_user_by_id(pool, user_id).await {
        Ok(Some(db_user)) => {
            let user = crate::models::user::User::from(db_user);
            ApiResponse::success(user, "Current user retrieved successfully".to_string())
        }
        Ok(None) => ApiResponse::not_found("User not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to get user: {}", e)),
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
    delete,
    path = "/api/submitters/{id}",
    params(
        ("id" = i64, Path, description = "Submitter ID")
    ),
    responses(
        (status = 200, description = "Submitter deleted successfully", body = ApiResponse<String>),
        (status = 404, description = "Submitter not found", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>)
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_submitter(
    State(state): State<AppState>,
    Path(submitter_id): Path<i64>,
    Extension(user_id): Extension<i64>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    let pool = &*state.lock().await;

    // First, verify the submitter exists and belongs to this user
    match SubmitterQueries::get_submitter_by_id(pool, submitter_id).await {
        Ok(Some(db_submitter)) => {
            // Check if the submitter belongs to this user
            if db_submitter.user_id != user_id {
                return ApiResponse::unauthorized("You don't have permission to delete this submitter".to_string());
            }

            // Delete the submitter
            match SubmitterQueries::delete_submitter(pool, submitter_id).await {
                Ok(true) => {
                    ApiResponse::success(
                        format!("Submitter {} deleted successfully", submitter_id),
                        "Submitter deleted successfully".to_string()
                    )
                }
                Ok(false) => ApiResponse::not_found("Submitter not found".to_string()),
                Err(e) => ApiResponse::internal_error(format!("Failed to delete submitter: {}", e)),
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
    get,
    path = "/public/submissions/{token}",
    params(
        ("token" = String, Path, description = "Submitter token")
    ),
    responses(
        (status = 200, description = "Submitter retrieved successfully", body = ApiResponse<crate::models::submitter::Submitter>),
        (status = 404, description = "Submitter not found", body = ApiResponse<crate::models::submitter::Submitter>)
    )
)]
pub async fn get_public_submitter(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> (StatusCode, Json<ApiResponse<crate::models::submitter::Submitter>>) {
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
            ApiResponse::success(submitter, "Submitter retrieved successfully".to_string())
        }
        Ok(None) => ApiResponse::not_found("Submitter not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to get submitter: {}", e)),
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
                        // Check if submitter is allowed to sign this field based on partner
                        if let Some(ref partner) = field.partner {
                            let allowed = partner == &db_submitter.name || 
                                         partner == &db_submitter.email || 
                                         db_submitter.name.contains(&format!("({})", partner));
                            if !allowed {
                                return ApiResponse::bad_request(format!("Field {} is not assigned to this submitter", signature_item.field_id));
                            }
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

#[utoipa::path(
    get,
    path = "/public/submissions/{token}/fields",
    params(
        ("token" = String, Path, description = "Submitter token")
    ),
    responses(
        (status = 200, description = "Template fields retrieved successfully", body = ApiResponse<crate::models::submitter::PublicSubmitterFieldsResponse>),
        (status = 404, description = "Submitter not found", body = ApiResponse<crate::models::submitter::PublicSubmitterFieldsResponse>)
    )
)]
pub async fn get_public_submitter_fields(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> (StatusCode, Json<ApiResponse<crate::models::submitter::PublicSubmitterFieldsResponse>>) {
    let pool = &*state.lock().await;

    match SubmitterQueries::get_submitter_by_token(pool, &token).await {
        Ok(Some(db_submitter)) => {
            // Get the template
            let template_id = db_submitter.template_id;
            match crate::database::queries::TemplateQueries::get_template_by_id(pool, template_id).await {
                Ok(Some(db_template)) => {
                    match crate::routes::templates::convert_db_template_to_template_with_fields(db_template, pool).await {
                        Ok(template) => {
                            // Extract only required info
                            let document = template.documents.as_ref()
                                .and_then(|docs| docs.get(0).cloned());
                            let template_info = crate::models::submitter::PublicTemplateInfo {
                                id: template.id,
                                name: template.name.clone(),
                                slug: template.slug.clone(),
                                user_id: template.user_id,
                                document,
                            };
                            let all_fields = template.template_fields.clone().unwrap_or_default();
                            // Filter fields based on partner matching submitter's name or email
                            println!("DEBUG: Submitter name: {}, email: {}", db_submitter.name, db_submitter.email);
                            let fields: Vec<crate::models::template::TemplateField> = all_fields.into_iter()
                                .filter(|field| {
                                    if let Some(ref partner) = field.partner {
                                        let matches = partner == &db_submitter.name || partner == &db_submitter.email;
                                        println!("DEBUG: Field {} partner '{}' matches: {}", field.name, partner, matches);
                                        matches
                                    } else {
                                        println!("DEBUG: Field {} has no partner, allowing", field.name);
                                        true // Allow fields without partner for all submitters
                                    }
                                })
                                .collect();
                            println!("DEBUG: Filtered fields count: {}", fields.len());
                            let response = crate::models::submitter::PublicSubmitterFieldsResponse {
                                template_info,
                                template_fields: fields,
                            };
                            ApiResponse::success(response, "Template fields retrieved successfully".to_string())
                        }
                        Err(e) => ApiResponse::internal_error(format!("Failed to load template fields: {}", e)),
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
    get,
    path = "/public/submissions/{token}/signatures",
    params(
        ("token" = String, Path, description = "Submitter token")
    ),
    responses(
        (status = 200, description = "Signatures retrieved successfully", body = ApiResponse<crate::models::submitter::PublicSubmitterSignaturesResponse>),
        (status = 404, description = "Submitter not found", body = ApiResponse<crate::models::submitter::PublicSubmitterSignaturesResponse>)
    )
)]
pub async fn get_public_submitter_signatures(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> (StatusCode, Json<ApiResponse<crate::models::submitter::PublicSubmitterSignaturesResponse>>) {
    let pool = &*state.lock().await;

    match SubmitterQueries::get_submitter_by_token(pool, &token).await {
        Ok(Some(db_submitter)) => {
            // Get the template
            let template_id = db_submitter.template_id;
            match crate::database::queries::TemplateQueries::get_template_by_id(pool, template_id).await {
                Ok(Some(db_template)) => {
                    match crate::routes::templates::convert_db_template_to_template_with_fields(db_template, pool).await {
                        Ok(template) => {
                            // Extract template info
                            let document = template.documents.as_ref()
                                .and_then(|docs| docs.get(0).cloned());
                            let template_info = crate::models::submitter::PublicTemplateInfo {
                                id: template.id,
                                name: template.name.clone(),
                                slug: template.slug.clone(),
                                user_id: template.user_id,
                                document,
                            };
                            
                            let fields = crate::database::queries::TemplateFieldQueries::get_all_template_fields(pool, template_id).await
                                .unwrap_or_default();
                            
                            // Get all submitters for this template
                            match SubmitterQueries::get_submitters_by_template(pool, template_id).await {
                                Ok(all_submitters) => {
                                    // Group submitters by creation time proximity (within 1 minute)
                                    let mut time_groups: std::collections::HashMap<String, Vec<crate::database::models::DbSubmitter>> = std::collections::HashMap::new();

                                    for submitter in &all_submitters {
                                        // Group by minute timestamp (floor to nearest minute)
                                        let timestamp = submitter.created_at.timestamp();
                                        let minute_key = (timestamp / 60).to_string(); // Group by minute
                                        time_groups.entry(minute_key).or_insert_with(Vec::new).push(submitter.clone());
                                    }

                                    // Find the group that contains the current submitter
                                    let current_group = time_groups.into_iter()
                                        .find(|(_, group)| group.iter().any(|s| s.id == db_submitter.id))
                                        .map(|(_, group)| group)
                                        .unwrap_or_else(|| vec![db_submitter.clone()]);

                                    // Collect all bulk_signatures from submitters in the same group
                                    let mut all_signatures = Vec::new();
                                    
                                    for submitter in current_group {
                                        if let Some(signatures) = &submitter.bulk_signatures {
                                            if let Some(signatures_array) = signatures.as_array() {
                                                for sig in signatures_array {
                                                    let mut enriched_sig = sig.clone();
                                                    // Add submitter info to each signature
                                                    if let Some(obj) = enriched_sig.as_object_mut() {
                                                        obj.insert("submitter_name".to_string(), serde_json::Value::String(submitter.name.clone()));
                                                        obj.insert("submitter_email".to_string(), serde_json::Value::String(submitter.email.clone()));
                                                        obj.insert("submitter_id".to_string(), serde_json::Value::Number(submitter.id.into()));
                                                        obj.insert("signed_at".to_string(), serde_json::Value::String(submitter.signed_at.map(|dt| dt.to_rfc3339()).unwrap_or_default()));
                                                        
                                                        // Enrich with field information
                                                        if let Some(field_id) = sig.get("field_id").and_then(|v| v.as_i64()) {
                                                            if let Some(field) = fields.iter().find(|f| f.id == field_id) {
                                                                obj.insert("field_info".to_string(), serde_json::to_value(field).unwrap_or(serde_json::Value::Null));
                                                            }
                                                        }
                                                    }
                                                    all_signatures.push(enriched_sig);
                                                }
                                            }
                                        }
                                    }
                                    
                                    let bulk_signatures = if all_signatures.is_empty() {
                                        None
                                    } else {
                                        Some(serde_json::Value::Array(all_signatures))
                                    };
                                    
                                    let response = crate::models::submitter::PublicSubmitterSignaturesResponse {
                                        template_info,
                                        bulk_signatures,
                                    };
                                    ApiResponse::success(response, "All signatures retrieved successfully".to_string())
                                }
                                Err(e) => ApiResponse::internal_error(format!("Failed to get submitters: {}", e)),
                            }
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

pub fn create_submitter_router() -> Router<AppState> {
    println!("Creating submitter router...");
    Router::new()
        .route("/me", get(get_me))
        .route("/submitters", get(get_submitters))
        .route("/submitters/:id", get(get_submitter))
        .route("/submitters/:id", put(update_submitter))
        .route("/submitters/:id", delete(delete_submitter))
        .layer(middleware::from_fn(auth_middleware))
        .layer(middleware::from_fn(require_admin_or_team_member))
}
