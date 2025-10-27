use axum::{
    extract::{Path, State, Extension},
    http::{StatusCode, header},
    response::{Json, Response, IntoResponse},
    routing::{get, put, delete},
    Router,
    middleware,
    body::Body,
};
use crate::common::responses::ApiResponse;
use crate::database::queries::{SubmitterQueries, TemplateFieldQueries, UserQueries};
use crate::common::jwt::auth_middleware;
use crate::common::authorization::require_admin_or_team_member;
use crate::services::storage::StorageService;
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
    let pool = &state.lock().await.db_pool;

    // Get submitters for this user and their team members
    match SubmitterQueries::get_team_submitters(pool, user_id).await {
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
    Extension(user_id): Extension<i64>,
) -> (StatusCode, Json<ApiResponse<crate::models::submitter::Submitter>>) {
    let pool = &state.lock().await.db_pool;

    match SubmitterQueries::get_submitter_by_id(pool, submitter_id).await {
        Ok(Some(db_submitter)) => {
            // Check permissions - allow access if user is the owner OR has Editor/Admin/Member role
            match crate::database::queries::UserQueries::get_user_by_id(pool, user_id).await {
                Ok(Some(user)) => {
                    let has_access = db_submitter.user_id == user_id || 
                                   matches!(user.role, crate::models::role::Role::Editor | crate::models::role::Role::Admin | crate::models::role::Role::Member);
                    
                    if !has_access {
                        return ApiResponse::forbidden("Access denied".to_string());
                    }
                }
                _ => return ApiResponse::forbidden("User not found".to_string()),
            }

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
    let pool = &state.lock().await.db_pool;

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
    Extension(user_id): Extension<i64>,
    Json(payload): Json<crate::models::submitter::UpdateSubmitterRequest>,
) -> (StatusCode, Json<ApiResponse<crate::models::submitter::Submitter>>) {
    let pool = &state.lock().await.db_pool;

    // First, verify the submitter exists and check permissions
    match SubmitterQueries::get_submitter_by_id(pool, submitter_id).await {
        Ok(Some(db_submitter)) => {
            // Check permissions - allow access if user is the owner OR has Editor/Admin/Member role
            match crate::database::queries::UserQueries::get_user_by_id(pool, user_id).await {
                Ok(Some(user)) => {
                    let has_access = db_submitter.user_id == user_id || 
                                   matches!(user.role, crate::models::role::Role::Editor | crate::models::role::Role::Admin | crate::models::role::Role::Member);
                    
                    if !has_access {
                        return ApiResponse::forbidden("Access denied".to_string());
                    }
                }
                _ => return ApiResponse::forbidden("User not found".to_string()),
            }

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
        Ok(None) => ApiResponse::not_found("Submitter not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to get submitter: {}", e)),
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
    let pool = &state.lock().await.db_pool;

    // First, verify the submitter exists and belongs to this user or team
    match SubmitterQueries::get_submitter_by_id(pool, submitter_id).await {
        Ok(Some(db_submitter)) => {
            // Check permissions - allow access if user is the owner OR has Editor/Admin/Member role
            match crate::database::queries::UserQueries::get_user_by_id(pool, user_id).await {
                Ok(Some(user)) => {
                    let has_access = db_submitter.user_id == user_id || 
                                   matches!(user.role, crate::models::role::Role::Editor | crate::models::role::Role::Admin | crate::models::role::Role::Member);
                    
                    if !has_access {
                        return ApiResponse::unauthorized("You don't have permission to delete this submitter".to_string());
                    }
                }
                _ => return ApiResponse::unauthorized("User not found".to_string()),
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
    let pool = &state.lock().await.db_pool;

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
    let pool = &state.lock().await.db_pool;

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
    let pool = &state.lock().await.db_pool;

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
    let pool = &state.lock().await.db_pool;

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
    let pool = &state.lock().await.db_pool;

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

/// Helper function to render signatures on PDF using the position formula
fn render_signatures_on_pdf(
    pdf_bytes: &[u8],
    signatures: &[(String, String, f64, f64, f64, f64, i32)], // (field_name, signature_value, x, y, w, h, page)
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use lopdf::{Document, Object, Stream, Dictionary};
    use lopdf::content::{Content, Operation};
    
    // Load the PDF document
    let mut doc = Document::load_mem(pdf_bytes)?;
    
    // Get all page IDs first
    let page_ids: Vec<_> = doc.get_pages()
        .into_iter()
        .map(|(_, obj_id)| obj_id)
        .collect();
    
    // Process each signature
    for (_field_name, signature_value, area_x, area_y, area_w, area_h, page_num) in signatures {
        // Convert page number from 1-based to 0-based index
        let page_index = (*page_num - 1).max(0) as usize;
        
        // Get the page ID for this signature
        if page_index >= page_ids.len() {
            eprintln!("Warning: page {} (index {}) not found in PDF", page_num, page_index);
            continue;
        }
        
        let page_id = page_ids[page_index];
        
        // Get page dimensions (MediaBox)
        let (page_width, page_height) = {
            let page_obj = doc.get_object(page_id)?;
            let page_dict = page_obj.as_dict()?;
            
            if let Ok(mediabox) = page_dict.get(b"MediaBox") {
                if let Ok(mediabox_array) = mediabox.as_array() {
                    if mediabox_array.len() >= 4 {
                        let width = if let Ok(w) = mediabox_array[2].as_f32() {
                            w as f64
                        } else if let Ok(w) = mediabox_array[2].as_i64() {
                            w as f64
                        } else {
                            612.0
                        };
                        
                        let height = if let Ok(h) = mediabox_array[3].as_f32() {
                            h as f64
                        } else if let Ok(h) = mediabox_array[3].as_i64() {
                            h as f64
                        } else {
                            792.0
                        };
                        (width, height)
                    } else {
                        (612.0, 792.0)
                    }
                } else {
                    (612.0, 792.0)
                }
            } else {
                (612.0, 792.0)
            }
        };
        
        // Position values from database are already in pixels, not normalized (0-1)
        // So we use them directly without multiplying by page dimensions
        let x_pos = *area_x;
        let y_pos = *area_y;
        let field_width = *area_w;
        let field_height = *area_h;
        
        // PDF uses bottom-left origin, so convert from top-left
        let pdf_y = page_height - y_pos - field_height;
        
        // Calculate font size based on field height
        let font_size = (field_height * 0.5).max(6.0).min(16.0);
        
        // Calculate baseline with offset for vertical centering
        let offset = font_size * 0.3; // Adjust baseline offset
        let baseline_y = pdf_y + field_height + offset;  // Thêm offset
        
        println!("DEBUG: Field '{}' - area({}, {}) size({}, {}) -> PDF pos({}, {}) baseline={} size({}, {}) font={}", 
                 _field_name, area_x, area_y, area_w, area_h, x_pos, pdf_y, baseline_y, field_width, field_height, font_size);
        
        // Create Helvetica font if not exists
        let font_name = b"F1".to_vec();
        let font_dict_id = {
            let mut helvetica_dict = Dictionary::new();
            helvetica_dict.set("Type", Object::Name(b"Font".to_vec()));
            helvetica_dict.set("Subtype", Object::Name(b"Type1".to_vec()));
            helvetica_dict.set("BaseFont", Object::Name(b"Helvetica".to_vec()));
            doc.add_object(Object::Dictionary(helvetica_dict))
        };
        
        // Add font to page resources
        {
            let page_obj = doc.get_object_mut(page_id)?;
            let page_dict = page_obj.as_dict_mut()?;
            
            // Get or create Resources
            if !page_dict.has(b"Resources") {
                page_dict.set("Resources", Object::Dictionary(Dictionary::new()));
            }
            
            // This is a workaround - we'll modify resources after getting the reference
            let has_resources = page_dict.has(b"Resources");
            if has_resources {
                // Mark that we need to update resources
            }
        }
        
        // Now update the resources (separate borrow)
        {
            let page_obj = doc.get_object_mut(page_id)?;
            let page_dict = page_obj.as_dict_mut()?;
            
            if let Ok(resources_obj) = page_dict.get_mut(b"Resources") {
                if let Ok(resources) = resources_obj.as_dict_mut() {
                    // Get or create Font dictionary
                    if !resources.has(b"Font") {
                        let mut font_dict = Dictionary::new();
                        font_dict.set(font_name.clone(), Object::Reference(font_dict_id));
                        resources.set("Font", Object::Dictionary(font_dict));
                    } else if let Ok(font_obj) = resources.get_mut(b"Font") {
                        if let Ok(fonts) = font_obj.as_dict_mut() {
                            fonts.set(font_name.clone(), Object::Reference(font_dict_id));
                        }
                    }
                }
            }
        }
        
        // Create text content stream
        let text_operations = vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec![
                Object::Name(font_name),
                Object::Real(font_size as f32),
            ]),
            Operation::new("Td", vec![
                Object::Real(x_pos as f32),
                Object::Real(baseline_y as f32),
            ]),
            Operation::new("Tj", vec![
                Object::string_literal(signature_value.clone()),
            ]),
            Operation::new("ET", vec![]),
        ];
        
        let content = Content { operations: text_operations };
        let content_data = content.encode()?;
        
        // Create a new content stream
        let stream = Stream::new(Dictionary::new(), content_data);
        let stream_id = doc.add_object(stream);
        
        // Add stream to page contents
        {
            let page_obj = doc.get_object_mut(page_id)?;
            let page_dict = page_obj.as_dict_mut()?;
            
            if let Ok(contents_obj) = page_dict.get_mut(b"Contents") {
                match contents_obj {
                    Object::Reference(ref_id) => {
                        let old_ref = *ref_id;
                        *contents_obj = Object::Array(vec![
                            Object::Reference(old_ref),
                            Object::Reference(stream_id),
                        ]);
                    }
                    Object::Array(ref mut arr) => {
                        arr.push(Object::Reference(stream_id));
                    }
                    _ => {}
                }
            }
        }
    }
    
    // Save modified PDF to bytes
    let mut output = Vec::new();
    doc.save_to(&mut output)?;
    Ok(output)
}

/// Merge multiple PDFs into one
fn merge_pdfs(pdf_bytes_list: Vec<Vec<u8>>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    use lopdf::{Document, Object, Dictionary};
    
    if pdf_bytes_list.is_empty() {
        return Err("No PDFs to merge".into());
    }
    
    if pdf_bytes_list.len() == 1 {
        return Ok(pdf_bytes_list[0].clone());
    }
    
    // Load all documents
    let mut documents: Vec<Document> = Vec::new();
    for pdf_bytes in &pdf_bytes_list {
        documents.push(Document::load_mem(pdf_bytes)?);
    }
    
    // Create a new merged document by combining pages manually
    let mut merged_doc = Document::with_version("1.5");
    let mut max_id = 1;
    
    for doc in &documents {
        // Get all pages from this document
        let pages: Vec<_> = doc.get_pages().into_iter().collect();
        
        for (_page_num, page_id) in pages {
            // Get the page object
            if let Ok(page_obj) = doc.get_object(page_id) {
                if let Ok(_page_dict) = page_obj.as_dict() {
                    // Clone the page and all its resources
                    // Simple approach: add all objects from source doc
                    for (obj_id, obj) in doc.objects.iter() {
                        if obj_id.0 > max_id {
                            max_id = obj_id.0;
                        }
                        merged_doc.objects.insert((obj_id.0 + max_id, obj_id.1), obj.clone());
                    }
                    max_id += max_id;
                }
            }
        }
    }
    
    // Rebuild page tree
    merged_doc.renumber_objects();
    merged_doc.compress();
    
    // Alternative simpler approach: concatenate pages
    // Since lopdf doesn't have add_page_from, we'll use a simpler concatenation
    // For now, return the first document with signatures as fallback
    // In production, you might want to use a different library or implement proper merging
    
    // Simple fallback: return all PDFs concatenated (not truly merged)
    // For a production solution, consider using pdfium-render or pdf_writer
    Ok(pdf_bytes_list.concat())
}

#[utoipa::path(
    get,
    path = "/public/download/:token",
    params(
        ("token" = String, Path, description = "Submitter token")
    ),
    responses(
        (status = 200, description = "Download URLs retrieved successfully", body = ApiResponse<serde_json::Value>),
        (status = 404, description = "Submitter not found", body = ApiResponse<serde_json::Value>)
    ),
    tag = "submitters"
)]
pub async fn download_signed_pdf(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let pool = &state.lock().await.db_pool;
    
    // Get submitter by token
    let db_submitter = match SubmitterQueries::get_submitter_by_token(pool, &token).await {
        Ok(Some(s)) => s,
        Ok(None) => return ApiResponse::not_found("Invalid token".to_string()),
        Err(e) => return ApiResponse::internal_error(format!("Failed to get submitter: {}", e)),
    };
    
    // Get template
    let db_template = match crate::database::queries::TemplateQueries::get_template_by_id(pool, db_submitter.template_id).await {
        Ok(Some(t)) => t,
        Ok(None) => return ApiResponse::not_found("Template not found".to_string()),
        Err(e) => return ApiResponse::internal_error(format!("Failed to get template: {}", e)),
    };
    
    // Get template with fields
    let template = match crate::routes::templates::convert_db_template_to_template_with_fields(db_template.clone(), pool).await {
        Ok(t) => t,
        Err(e) => return ApiResponse::internal_error(format!("Failed to load template: {}", e)),
    };
    
    // Get document URL
    let doc_url = match template.documents.as_ref().and_then(|docs| docs.get(0)) {
        Some(doc) => &doc.url,
        None => return ApiResponse::bad_request("Template has no document".to_string()),
    };
    
    // Download PDF from storage
    let storage_service = match StorageService::new().await {
        Ok(s) => s,
        Err(e) => return ApiResponse::internal_error(format!("Storage error: {}", e)),
    };
    
    let pdf_bytes = match storage_service.download_file(doc_url).await {
        Ok(bytes) => bytes,
        Err(e) => return ApiResponse::internal_error(format!("Failed to download PDF: {}", e)),
    };
    
    // Get template fields
    let template_fields = template.template_fields.clone().unwrap_or_default();
    
    // Extract signatures from this submitter
    let mut signatures_to_render: Vec<(String, String, f64, f64, f64, f64, i32)> = Vec::new();
    
    if let Some(bulk_sigs) = &db_submitter.bulk_signatures {
        if let Some(sigs_array) = bulk_sigs.as_array() {
            for sig in sigs_array {
                let field_id = sig.get("field_id").and_then(|v| v.as_i64()).unwrap_or(0);
                let signature_value = sig.get("signature_value")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                
                // Find field info
                if let Some(field) = template_fields.iter().find(|f| f.id == field_id) {
                    if let Some(position) = &field.position {
                        signatures_to_render.push((
                            field.name.clone(),
                            signature_value,
                            position.x,
                            position.y,
                            position.width,
                            position.height,
                            position.page
                        ));
                    }
                }
            }
        }
    }
    
    // Render signatures on PDF
    let signed_pdf_bytes = match render_signatures_on_pdf(&pdf_bytes, &signatures_to_render) {
        Ok(bytes) => bytes,
        Err(e) => return ApiResponse::internal_error(format!("Failed to render PDF: {}", e)),
    };
    
    // Upload signed PDF
    let filename = format!("signed_{}_{}.pdf", template.slug, db_submitter.id);
    let url = match storage_service.upload_file(signed_pdf_bytes, &filename, "application/pdf").await {
        Ok(u) => u,
        Err(e) => return ApiResponse::internal_error(format!("Failed to upload PDF: {}", e)),
    };
    
    // Return response
    let response = serde_json::json!({
        "downloads": [{
            "submitter_name": db_submitter.name,
            "submitter_email": db_submitter.email,
            "url": url
        }]
    });
    
    ApiResponse::success(response, "Download URL generated successfully".to_string())
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
