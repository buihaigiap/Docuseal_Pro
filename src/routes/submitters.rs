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
use crate::database::queries::{SubmitterQueries, UserQueries, SubmissionFieldQueries};
use crate::common::jwt::auth_middleware;
use crate::common::authorization::require_admin_or_team_member;
use crate::services::storage::StorageService;
use chrono::Utc;
use serde_json;

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
            
            // Get submission fields for this submitter to validate against
            let submission_fields = match SubmissionFieldQueries::get_submission_fields_by_submitter_id(pool, db_submitter.id).await {
                Ok(fields) => fields,
                Err(e) => return ApiResponse::internal_error(format!("Failed to get submission fields: {}", e)),
            };

            // Validate that all field_ids belong to this submitter's submission fields
            for signature_item in &payload.signatures {
                // Find the field in submission fields
                if let Some(field) = submission_fields.iter().find(|f| f.id == signature_item.field_id) {
                    // Check if submitter is allowed to sign this field based on partner
                    if let Some(ref partner) = field.partner {
                        let allowed = partner == &db_submitter.name || 
                                     partner == &db_submitter.email || 
                                     db_submitter.name.contains(&format!("({})", partner));
                        if !allowed {
                            return ApiResponse::bad_request(format!("Field {} is not assigned to this submitter", signature_item.field_id));
                        }
                    }
                } else {
                    return ApiResponse::bad_request(format!("Field {} not found in submission", signature_item.field_id));
                }
            }

            // Create signatures array with field details
            let mut signatures_array = Vec::new();
            for signature_item in &payload.signatures {
                let field_id = signature_item.field_id;
                let field_name = submission_fields.iter()
                    .find(|f| f.id == field_id)
                    .map(|f| f.name.clone())
                    .unwrap_or_else(|| format!("field_{}", field_id));
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
            // Get the template for basic info
            let template_id = db_submitter.template_id;
            match crate::database::queries::TemplateQueries::get_template_by_id(pool, template_id).await {
                Ok(Some(db_template)) => {
                    // Get submission fields instead of template fields
                    match SubmissionFieldQueries::get_submission_fields_by_submitter_id(pool, db_submitter.id).await {
                        Ok(submission_fields) => {
                            // Convert submission fields to template fields for response
                            let template_fields: Vec<crate::models::template::TemplateField> = submission_fields.into_iter().map(|sf| {
                                crate::models::template::TemplateField {
                                    id: sf.id,
                                    template_id: sf.submitter_id, // Use submitter_id as template_id for compatibility
                                    name: sf.name,
                                    field_type: sf.field_type,
                                    required: sf.required,
                                    display_order: sf.display_order,
                                    position: sf.position.map(|pos| {
                                        // Parse position JSON to FieldPosition
                                        serde_json::from_value(pos).unwrap_or_else(|_| crate::models::template::FieldPosition {
                                            x: 0.0, y: 0.0, width: 100.0, height: 20.0, page: 1, suggested: None, allow_custom: None
                                        })
                                    }),
                                    options: sf.options,
                                    partner: sf.partner,
                                    created_at: sf.created_at,
                                    updated_at: sf.updated_at,
                                }
                            }).collect();

                            // Extract template info
                            let document = db_template.documents.as_ref()
                                .and_then(|docs| {
                                    if let serde_json::Value::Array(arr) = docs {
                                        arr.get(0)
                                    } else {
                                        None
                                    }
                                })
                                .and_then(|doc| serde_json::from_value(doc.clone()).ok());
                            let template_info = crate::models::submitter::PublicTemplateInfo {
                                id: db_template.id,
                                name: db_template.name.clone(),
                                slug: db_template.slug.clone(),
                                user_id: db_template.user_id,
                                document,
                            };

                            // Filter fields based on partner matching submitter's name or email
                            println!("DEBUG: Submitter name: {}, email: {}", db_submitter.name, db_submitter.email);
                            let filtered_fields: Vec<crate::models::template::TemplateField> = template_fields.into_iter()
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
                            println!("DEBUG: Filtered fields count: {}", filtered_fields.len());

                            let response = crate::models::submitter::PublicSubmitterFieldsResponse {
                                template_info,
                                template_fields: filtered_fields,
                            };
                            ApiResponse::success(response, "Submission fields retrieved successfully".to_string())
                        }
                        Err(e) => ApiResponse::internal_error(format!("Failed to get submission fields: {}", e)),
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
                                        // Get submission fields for this submitter
                                        let submission_fields = match SubmissionFieldQueries::get_submission_fields_by_submitter_id(pool, submitter.id).await {
                                            Ok(fields) => fields,
                                            Err(_) => Vec::new(), // Continue without field info if query fails
                                        };
                                        
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
                                                        
                                                        // Enrich with field information from submission fields
                                                        if let Some(field_id) = sig.get("field_id").and_then(|v| v.as_i64()) {
                                                            if let Some(field) = submission_fields.iter().find(|f| f.id == field_id) {
                                                                // Convert submission field to template field format for response
                                                                let field_info = serde_json::json!({
                                                                    "id": field.id,
                                                                    "template_id": field.submitter_id,
                                                                    "name": field.name,
                                                                    "field_type": field.field_type,
                                                                    "required": field.required,
                                                                    "display_order": field.display_order,
                                                                    "position": field.position,
                                                                    "options": field.options,
                                                                    "partner": field.partner,
                                                                    "created_at": field.created_at,
                                                                    "updated_at": field.updated_at
                                                                });
                                                                obj.insert("field_info".to_string(), field_info);
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
    signatures: &[(String, String, String, f64, f64, f64, f64, i32)], // (field_name, field_type, signature_value, x, y, w, h, page)
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
    for (field_name, field_type, signature_value, area_x, area_y, area_w, area_h, page_num) in signatures {
        // Skip empty signatures
        if signature_value.trim().is_empty() {
            continue;
        }
        
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
        
        // ==================== CÔNG THỨC CHÍNH XÁC ====================
        
        // Database lưu tọa độ gốc (TOP-LEFT origin, chưa scale)
        let x_pos = *area_x;
        let y_pos = *area_y;
        let field_width = *area_w;
        let field_height = *area_h;
        
        // Chuyển đổi Y từ TOP-LEFT (Frontend) sang BOTTOM-LEFT (PDF)
        // Frontend: Y=0 ở top, tăng xuống dưới
        // PDF: Y=0 ở bottom, tăng lên trên
        let pdf_y = page_height - y_pos - field_height;
        
        // Calculate font size based on field height
        // Tương tự công thức trong PdfViewer.tsx: getFontSize()
        let font_size = (field_height * 0.65).max(8.0).min(16.0);
        
        // Calculate baseline for vertical centering
        // Text baseline cần offset để text hiển thị ở giữa field
        let baseline_offset = (field_height - font_size) / 2.0;
        let baseline_y = pdf_y + baseline_offset + font_size * 0.25;
        
        // ==================== KẾT THÚC CÔNG THỨC ====================
        
        println!("╔══════════════════════════════════════════════════════════╗");
        println!("║ Field: '{}' ({})", field_name, field_type);
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║ Database (TOP-LEFT):                                     ║");
        println!("║   x={:.2}, y={:.2}, w={:.2}, h={:.2}", area_x, area_y, area_w, area_h);
        println!("║ PDF Page: {:.2} x {:.2}", page_width, page_height);
        println!("║ PDF Coords (BOTTOM-LEFT):                                ║");
        println!("║   x={:.2}, y={:.2}", x_pos, pdf_y);
        println!("║ Value: '{}'", signature_value);
        println!("╚══════════════════════════════════════════════════════════╝");
        
        // Process based on field type
        match field_type.as_str() {
            "checkbox" => {
                // Hiển thị biểu tượng SVG dấu tích nếu giá trị là 'true', nếu không thì ô vuông trống
                if signature_value.to_lowercase() == "true" {
                    // Draw checkmark SVG
                    render_checkbox_with_check(&mut doc, page_id, x_pos, pdf_y, field_width, field_height)?;
                } else {
                    // Draw empty square
                    render_empty_checkbox(&mut doc, page_id, x_pos, pdf_y, field_width, field_height)?;
                }
            },
            "multiple" => {
                // Chia giá trị theo dấu phẩy và nối chúng bằng dấu cách
                let display_value = signature_value.split(',').collect::<Vec<&str>>().join(" ");
                render_text_field(&mut doc, page_id, &display_value, x_pos, pdf_y, field_width, field_height)?;
            },
            "cells" => {
                // Hiển thị trong bố cục lưới với mỗi ký tự trong một ô riêng biệt
                render_cells_field(&mut doc, page_id, &signature_value, x_pos, pdf_y, field_width, field_height)?;
            },
            "radio" => {
                // Hiển thị giá trị đã chọn hoặc chỗ giữ chỗ
                let display_value = if signature_value.is_empty() {
                    format!("Chọn {}", field_name)
                } else {
                    signature_value.to_string()
                };
                render_text_field(&mut doc, page_id, &display_value, x_pos, pdf_y, field_width, field_height)?;
            },
            "initials" => {
                // Render chữ ký từ vector data hoặc text
                if signature_value.starts_with('[') {
                    // Đây là vector signature data - render drawing
                    render_vector_signature(&mut doc, page_id, &signature_value, x_pos, pdf_y, field_width, field_height)?;
                } else if signature_value.starts_with('{') {
                    // JSON object - có thể có text hoặc vector
                    if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(signature_value) {
                        if let Some(text) = json_value.get("text").and_then(|t| t.as_str()) {
                            render_initials_field(&mut doc, page_id, text, x_pos, pdf_y, field_width, field_height)?;
                        } else if let Some(initials) = json_value.get("initials").and_then(|i| i.as_str()) {
                            render_initials_field(&mut doc, page_id, initials, x_pos, pdf_y, field_width, field_height)?;
                        } else {
                            render_initials_field(&mut doc, page_id, "[SIGNATURE]", x_pos, pdf_y, field_width, field_height)?;
                        }
                    } else {
                        render_initials_field(&mut doc, page_id, &signature_value, x_pos, pdf_y, field_width, field_height)?;
                    }
                } else {
                    // Plain text
                    render_initials_field(&mut doc, page_id, &signature_value, x_pos, pdf_y, field_width, field_height)?;
                }
            },
            "image" => {
                // Hiển thị <img> với giá trị làm nguồn, được co giãn để vừa với khu vực trường
                // Note: lopdf không hỗ trợ embed images trực tiếp, sẽ render như text placeholder
                let display_value = format!("[IMAGE: {}]", signature_value);
                render_text_field(&mut doc, page_id, &display_value, x_pos, pdf_y, field_width, field_height)?;
            },
            "file" => {
                // Hiển thị liên kết tải xuống có thể nhấp với tên tệp được trích xuất từ URL
                let filename = extract_filename_from_url(&signature_value);
                let display_value = format!("[DOWNLOAD: {}]", filename);
                render_text_field(&mut doc, page_id, &display_value, x_pos, pdf_y, field_width, field_height)?;
            },
            _ => {
                // Mặc định (trường văn bản): Hiển thị giá trị dưới dạng văn bản thuần túy
                let display_value = if signature_value.is_empty() {
                    field_name.clone()
                } else {
                    signature_value.to_string()
                };
                render_text_field(&mut doc, page_id, &display_value, x_pos, pdf_y, field_width, field_height)?;
            }
        }
    }
    
    // Save modified PDF to bytes
    let mut output = Vec::new();
    doc.save_to(&mut output)?;
    Ok(output)
}

// Helper function to extract filename from URL
fn extract_filename_from_url(url: &str) -> String {
    url.split('/').last().unwrap_or("file").to_string()
}

// Render text field (default)
fn render_text_field(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    text: &str,
    x_pos: f64,
    pdf_y: f64,
    field_width: f64,
    field_height: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    use lopdf::{Object, Stream, Dictionary};
    use lopdf::content::{Content, Operation};

    // Calculate font size based on field height
    let font_size = (field_height * 0.65).max(8.0).min(16.0);

    // Calculate baseline for vertical centering
    let baseline_offset = (field_height - font_size) / 2.0;
    let baseline_y = pdf_y + baseline_offset + font_size * 0.25;

    // Create Helvetica font if not exists
    let font_name = b"F1".to_vec();
    let font_dict_id = {
        let mut helvetica_dict = Dictionary::new();
        helvetica_dict.set("Type", Object::Name(b"Font".to_vec()));
        helvetica_dict.set("Subtype", Object::Name(b"Type1".to_vec()));
        helvetica_dict.set("BaseFont", Object::Name(b"Helvetica".to_vec()));
        helvetica_dict.set("Encoding", Object::Name(b"WinAnsiEncoding".to_vec()));
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
    }

    // Update the resources (separate borrow)
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

    // Create text content stream with proper positioning
    let text_operations = vec![
        // Begin text object
        Operation::new("BT", vec![]),

        // Set font and size
        Operation::new("Tf", vec![
            Object::Name(font_name),
            Object::Real(font_size as f32),
        ]),

        // Set text color to black
        Operation::new("rg", vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(0.0),
        ]),

        // Position text at baseline
        Operation::new("Td", vec![
            Object::Real(x_pos as f32),
            Object::Real(baseline_y as f32),
        ]),

        // Show text
        Operation::new("Tj", vec![
            Object::string_literal(text.to_string()),
        ]),

        // End text object
        Operation::new("ET", vec![]),
    ];

    let content = Content { operations: text_operations };
    let content_data = content.encode()?;

    // Create a new content stream
    let mut stream_dict = Dictionary::new();
    stream_dict.set("Length", Object::Integer(content_data.len() as i64));
    let stream = Stream::new(stream_dict, content_data);
    let stream_id = doc.add_object(stream);

    // Add stream to page contents
    {
        let page_obj = doc.get_object_mut(page_id)?;
        let page_dict = page_obj.as_dict_mut()?;

        if let Ok(contents_obj) = page_dict.get_mut(b"Contents") {
            match contents_obj {
                Object::Reference(ref_id) => {
                    let old_ref = ref_id;
                    *contents_obj = Object::Array(vec![
                        Object::Reference(*old_ref),
                        Object::Reference(stream_id),
                    ]);
                }
                Object::Array(ref mut arr) => {
                    arr.push(Object::Reference(stream_id));
                }
                _ => {
                    // If contents is something else, replace it
                    *contents_obj = Object::Array(vec![Object::Reference(stream_id)]);
                }
            }
        } else {
            // No contents exist, create new
            page_dict.set("Contents", Object::Array(vec![Object::Reference(stream_id)]));
        }
    }

    Ok(())
}

// Render checkbox with checkmark
fn render_checkbox_with_check(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    x_pos: f64,
    pdf_y: f64,
    field_width: f64,
    field_height: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    use lopdf::{Object, Stream, Dictionary};
    use lopdf::content::{Content, Operation};

    // Draw square border
    let border_operations = vec![
        // Set line width
        Operation::new("w", vec![Object::Real(1.0)]),
        // Set color to black
        Operation::new("RG", vec![Object::Real(0.0), Object::Real(0.0), Object::Real(0.0)]),
        // Move to start
        Operation::new("m", vec![Object::Real(x_pos as f32), Object::Real(pdf_y as f32)]),
        // Draw rectangle border
        Operation::new("l", vec![Object::Real((x_pos + field_width) as f32), Object::Real(pdf_y as f32)]),
        Operation::new("l", vec![Object::Real((x_pos + field_width) as f32), Object::Real((pdf_y + field_height) as f32)]),
        Operation::new("l", vec![Object::Real(x_pos as f32), Object::Real((pdf_y + field_height) as f32)]),
        Operation::new("h", vec![]), // Close path
        Operation::new("S", vec![]), // Stroke

        // Draw checkmark
        Operation::new("m", vec![Object::Real((x_pos + field_width * 0.2) as f32), Object::Real((pdf_y + field_height * 0.5) as f32)]),
        Operation::new("l", vec![Object::Real((x_pos + field_width * 0.4) as f32), Object::Real((pdf_y + field_height * 0.3) as f32)]),
        Operation::new("l", vec![Object::Real((x_pos + field_width * 0.8) as f32), Object::Real((pdf_y + field_height * 0.7) as f32)]),
        Operation::new("S", vec![]), // Stroke checkmark
    ];

    let content = Content { operations: border_operations };
    let content_data = content.encode()?;

    // Create a new content stream
    let mut stream_dict = Dictionary::new();
    stream_dict.set("Length", Object::Integer(content_data.len() as i64));
    let stream = Stream::new(stream_dict, content_data);
    let stream_id = doc.add_object(stream);

    // Add stream to page contents
    {
        let page_obj = doc.get_object_mut(page_id)?;
        let page_dict = page_obj.as_dict_mut()?;

        if let Ok(contents_obj) = page_dict.get_mut(b"Contents") {
            match contents_obj {
                Object::Reference(ref_id) => {
                    let old_ref = ref_id;
                    *contents_obj = Object::Array(vec![
                        Object::Reference(*old_ref),
                        Object::Reference(stream_id),
                    ]);
                }
                Object::Array(ref mut arr) => {
                    arr.push(Object::Reference(stream_id));
                }
                _ => {
                    // If contents is something else, replace it
                    *contents_obj = Object::Array(vec![Object::Reference(stream_id)]);
                }
            }
        } else {
            // No contents exist, create new
            page_dict.set("Contents", Object::Array(vec![Object::Reference(stream_id)]));
        }
    }

    Ok(())
}

// Render empty checkbox
fn render_empty_checkbox(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    x_pos: f64,
    pdf_y: f64,
    field_width: f64,
    field_height: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    use lopdf::{Object, Stream, Dictionary};
    use lopdf::content::{Content, Operation};

    // Draw square border only
    let border_operations = vec![
        // Set line width
        Operation::new("w", vec![Object::Real(1.0)]),
        // Set color to black
        Operation::new("RG", vec![Object::Real(0.0), Object::Real(0.0), Object::Real(0.0)]),
        // Move to start
        Operation::new("m", vec![Object::Real(x_pos as f32), Object::Real(pdf_y as f32)]),
        // Draw rectangle border
        Operation::new("l", vec![Object::Real((x_pos + field_width) as f32), Object::Real(pdf_y as f32)]),
        Operation::new("l", vec![Object::Real((x_pos + field_width) as f32), Object::Real((pdf_y + field_height) as f32)]),
        Operation::new("l", vec![Object::Real(x_pos as f32), Object::Real((pdf_y + field_height) as f32)]),
        Operation::new("h", vec![]), // Close path
        Operation::new("S", vec![]), // Stroke
    ];

    let content = Content { operations: border_operations };
    let content_data = content.encode()?;

    // Create a new content stream
    let mut stream_dict = Dictionary::new();
    stream_dict.set("Length", Object::Integer(content_data.len() as i64));
    let stream = Stream::new(stream_dict, content_data);
    let stream_id = doc.add_object(stream);

    // Add stream to page contents
    {
        let page_obj = doc.get_object_mut(page_id)?;
        let page_dict = page_obj.as_dict_mut()?;

        if let Ok(contents_obj) = page_dict.get_mut(b"Contents") {
            match contents_obj {
                Object::Reference(ref_id) => {
                    let old_ref = ref_id;
                    *contents_obj = Object::Array(vec![
                        Object::Reference(*old_ref),
                        Object::Reference(stream_id),
                    ]);
                }
                Object::Array(ref mut arr) => {
                    arr.push(Object::Reference(stream_id));
                }
                _ => {
                    // If contents is something else, replace it
                    *contents_obj = Object::Array(vec![Object::Reference(stream_id)]);
                }
            }
        } else {
            // No contents exist, create new
            page_dict.set("Contents", Object::Array(vec![Object::Reference(stream_id)]));
        }
    }

    Ok(())
}

// Render cells field (grid layout)
fn render_cells_field(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    text: &str,
    x_pos: f64,
    pdf_y: f64,
    field_width: f64,
    field_height: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    use lopdf::{Object, Stream, Dictionary};
    use lopdf::content::{Content, Operation};

    let chars: Vec<char> = text.chars().collect();
    let cell_width = field_width / chars.len() as f64;
    let font_size = (field_height * 0.8).min(cell_width * 0.8);

    // Create Helvetica font if not exists
    let font_name = b"F1".to_vec();
    let font_dict_id = {
        let mut helvetica_dict = Dictionary::new();
        helvetica_dict.set("Type", Object::Name(b"Font".to_vec()));
        helvetica_dict.set("Subtype", Object::Name(b"Type1".to_vec()));
        helvetica_dict.set("BaseFont", Object::Name(b"Helvetica".to_vec()));
        helvetica_dict.set("Encoding", Object::Name(b"WinAnsiEncoding".to_vec()));
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
    }

    // Update the resources (separate borrow)
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

    let mut operations = Vec::new();

    // Draw grid lines
    operations.push(Operation::new("w", vec![Object::Real(0.5)]));
    operations.push(Operation::new("RG", vec![Object::Real(0.7), Object::Real(0.7), Object::Real(0.7)]));

    for i in 0..=chars.len() {
        let x = x_pos + i as f64 * cell_width;
        // Vertical lines
        operations.push(Operation::new("m", vec![Object::Real(x as f32), Object::Real(pdf_y as f32)]));
        operations.push(Operation::new("l", vec![Object::Real(x as f32), Object::Real((pdf_y + field_height) as f32)]));
        operations.push(Operation::new("S", vec![]));
    }

    // Horizontal lines
    operations.push(Operation::new("m", vec![Object::Real(x_pos as f32), Object::Real(pdf_y as f32)]));
    operations.push(Operation::new("l", vec![Object::Real((x_pos + field_width) as f32), Object::Real(pdf_y as f32)]));
    operations.push(Operation::new("S", vec![]));

    operations.push(Operation::new("m", vec![Object::Real(x_pos as f32), Object::Real((pdf_y + field_height) as f32)]));
    operations.push(Operation::new("l", vec![Object::Real((x_pos + field_width) as f32), Object::Real((pdf_y + field_height) as f32)]));
    operations.push(Operation::new("S", vec![]));

    // Draw characters
    operations.push(Operation::new("BT", vec![]));
    operations.push(Operation::new("Tf", vec![
        Object::Name(font_name),
        Object::Real(font_size as f32),
    ]));
    operations.push(Operation::new("rg", vec![Object::Real(0.0), Object::Real(0.0), Object::Real(0.0)]));

    for (i, ch) in chars.iter().enumerate() {
        let cell_x = x_pos + i as f64 * cell_width + cell_width * 0.1;
        let baseline_y = pdf_y + field_height * 0.8;

        operations.push(Operation::new("Td", vec![
            Object::Real(cell_x as f32),
            Object::Real(baseline_y as f32),
        ]));
        operations.push(Operation::new("Tj", vec![Object::string_literal(ch.to_string())]));
    }

    operations.push(Operation::new("ET", vec![]));

    let content = Content { operations };
    let content_data = content.encode()?;

    // Create a new content stream
    let mut stream_dict = Dictionary::new();
    stream_dict.set("Length", Object::Integer(content_data.len() as i64));
    let stream = Stream::new(stream_dict, content_data);
    let stream_id = doc.add_object(stream);

    // Add stream to page contents
    {
        let page_obj = doc.get_object_mut(page_id)?;
        let page_dict = page_obj.as_dict_mut()?;

        if let Ok(contents_obj) = page_dict.get_mut(b"Contents") {
            match contents_obj {
                Object::Reference(ref_id) => {
                    let old_ref = ref_id;
                    *contents_obj = Object::Array(vec![
                        Object::Reference(*old_ref),
                        Object::Reference(stream_id),
                    ]);
                }
                Object::Array(ref mut arr) => {
                    arr.push(Object::Reference(stream_id));
                }
                _ => {
                    // If contents is something else, replace it
                    *contents_obj = Object::Array(vec![Object::Reference(stream_id)]);
                }
            }
        } else {
            // No contents exist, create new
            page_dict.set("Contents", Object::Array(vec![Object::Reference(stream_id)]));
        }
    }

    Ok(())
}

// Render initials field with special positioning
fn render_initials_field(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    text: &str,
    x_pos: f64,
    pdf_y: f64,
    field_width: f64,
    field_height: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    use lopdf::{Object, Stream, Dictionary};
    use lopdf::content::{Content, Operation};

    // Special font size for initials (smaller, more condensed)
    let font_size = (field_height * 0.6).max(10.0).min(18.0);

    // Create Helvetica font if not exists
    let font_name = b"F1".to_vec();
    let font_dict_id = {
        let mut helvetica_dict = Dictionary::new();
        helvetica_dict.set("Type", Object::Name(b"Font".to_vec()));
        helvetica_dict.set("Subtype", Object::Name(b"Type1".to_vec()));
        helvetica_dict.set("BaseFont", Object::Name(b"Helvetica-Bold".to_vec()));
        helvetica_dict.set("Encoding", Object::Name(b"WinAnsiEncoding".to_vec()));
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
    }

    // Update the resources (separate borrow)
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

    // Calculate positioning for initials (absolute positioning, custom line height)
    let baseline_y = pdf_y + field_height * 0.75; // Higher baseline for initials

    // Create text content stream with special positioning for initials
    let text_operations = vec![
        // Begin text object
        Operation::new("BT", vec![]),

        // Set font and size (bold for initials)
        Operation::new("Tf", vec![
            Object::Name(font_name),
            Object::Real(font_size as f32),
        ]),

        // Set text color to black
        Operation::new("rg", vec![
            Object::Real(0.0),
            Object::Real(0.0),
            Object::Real(0.0),
        ]),

        // Position text at baseline (absolute positioning)
        Operation::new("Td", vec![
            Object::Real(x_pos as f32),
            Object::Real(baseline_y as f32),
        ]),

        // Show text
        Operation::new("Tj", vec![
            Object::string_literal(text.to_string()),
        ]),

        // End text object
        Operation::new("ET", vec![]),
    ];

    let content = Content { operations: text_operations };
    let content_data = content.encode()?;

    // Create a new content stream
    let mut stream_dict = Dictionary::new();
    stream_dict.set("Length", Object::Integer(content_data.len() as i64));
    let stream = Stream::new(stream_dict, content_data);
    let stream_id = doc.add_object(stream);

    // Add stream to page contents
    {
        let page_obj = doc.get_object_mut(page_id)?;
        let page_dict = page_obj.as_dict_mut()?;

        if let Ok(contents_obj) = page_dict.get_mut(b"Contents") {
            match contents_obj {
                Object::Reference(ref_id) => {
                    let old_ref = ref_id;
                    *contents_obj = Object::Array(vec![
                        Object::Reference(*old_ref),
                        Object::Reference(stream_id),
                    ]);
                }
                Object::Array(ref mut arr) => {
                    arr.push(Object::Reference(stream_id));
                }
                _ => {
                    // If contents is something else, replace it
                    *contents_obj = Object::Array(vec![Object::Reference(stream_id)]);
                }
            }
        } else {
            // No contents exist, create new
            page_dict.set("Contents", Object::Array(vec![Object::Reference(stream_id)]));
        }
    }

    Ok(())
}

// Render vector signature from JSON points array
fn render_vector_signature(
    doc: &mut lopdf::Document,
    page_id: lopdf::ObjectId,
    vector_json: &str,
    x_pos: f64,
    pdf_y: f64,
    field_width: f64,
    field_height: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    use lopdf::{Object, Stream, Dictionary};
    use lopdf::content::{Content, Operation};

    // Parse vector data - expecting array of arrays [[{x,y,time,color},...], [{x,y,time,color},...]]
    let strokes: Vec<Vec<serde_json::Value>> = serde_json::from_str(vector_json)
        .unwrap_or_else(|_| Vec::new());

    if strokes.is_empty() {
        return Ok(());
    }

    // Calculate bounding box from all points to scale properly
    let mut min_x = f64::MAX;
    let mut max_x = f64::MIN;
    let mut min_y = f64::MAX;
    let mut max_y = f64::MIN;

    for stroke in &strokes {
        for point in stroke {
            if let (Some(x), Some(y)) = (
                point.get("x").and_then(|v| v.as_f64()),
                point.get("y").and_then(|v| v.as_f64())
            ) {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
            }
        }
    }

    // Calculate scale to fit in field
    let sig_width = max_x - min_x;
    let sig_height = max_y - min_y;
    
    let scale_x = if sig_width > 0.0 { field_width / sig_width } else { 1.0 };
    let scale_y = if sig_height > 0.0 { field_height / sig_height } else { 1.0 };
    let scale = scale_x.min(scale_y) * 0.9; // Use 90% to add padding

    // Center the signature
    let offset_x = x_pos + (field_width - sig_width * scale) / 2.0 - min_x * scale;
    let offset_y = pdf_y + (field_height - sig_height * scale) / 2.0 - min_y * scale;

    // Create path operations for drawing signature
    let mut operations = Vec::new();

    // Set line properties
    operations.push(Operation::new("w", vec![Object::Real(2.0)])); // Line width
    operations.push(Operation::new("RG", vec![Object::Real(0.0), Object::Real(0.0), Object::Real(0.0)])); // Black color
    operations.push(Operation::new("J", vec![Object::Integer(1)])); // Round line cap
    operations.push(Operation::new("j", vec![Object::Integer(1)])); // Round line join

    // Draw each stroke
    for stroke in &strokes {
        if stroke.is_empty() {
            continue;
        }

        let mut first_point = true;
        for point in stroke {
            if let (Some(x), Some(y)) = (
                point.get("x").and_then(|v| v.as_f64()),
                point.get("y").and_then(|v| v.as_f64())
            ) {
                let scaled_x = offset_x + x * scale;
                let scaled_y = offset_y + y * scale;

                if first_point {
                    // Move to start of stroke
                    operations.push(Operation::new("m", vec![
                        Object::Real(scaled_x as f32),
                        Object::Real(scaled_y as f32)
                    ]));
                    first_point = false;
                } else {
                    // Draw line to next point
                    operations.push(Operation::new("l", vec![
                        Object::Real(scaled_x as f32),
                        Object::Real(scaled_y as f32)
                    ]));
                }
            }
        }
        
        // Stroke this path
        operations.push(Operation::new("S", vec![]));
    }

    let content = Content { operations };
    let content_data = content.encode()?;

    // Create a new content stream
    let mut stream_dict = Dictionary::new();
    stream_dict.set("Length", Object::Integer(content_data.len() as i64));
    let stream = Stream::new(stream_dict, content_data);
    let stream_id = doc.add_object(stream);

    // Add stream to page contents
    {
        let page_obj = doc.get_object_mut(page_id)?;
        let page_dict = page_obj.as_dict_mut()?;

        if let Ok(contents_obj) = page_dict.get_mut(b"Contents") {
            match contents_obj {
                Object::Reference(ref_id) => {
                    let old_ref = ref_id;
                    *contents_obj = Object::Array(vec![
                        Object::Reference(*old_ref),
                        Object::Reference(stream_id),
                    ]);
                }
                Object::Array(ref mut arr) => {
                    arr.push(Object::Reference(stream_id));
                }
                _ => {
                    *contents_obj = Object::Array(vec![Object::Reference(stream_id)]);
                }
            }
        } else {
            page_dict.set("Contents", Object::Array(vec![Object::Reference(stream_id)]));
        }
    }

    Ok(())
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
        (status = 200, description = "PDF file downloaded successfully"),
        (status = 404, description = "Submitter not found")
    ),
    tag = "submitters"
)]
pub async fn download_signed_pdf(
    State(state): State<AppState>,
    Path(token): Path<String>,
) -> impl IntoResponse {
    let pool = &state.lock().await.db_pool;
    
    // Get submitter by token
    let db_submitter = match SubmitterQueries::get_submitter_by_token(pool, &token).await {
        Ok(Some(s)) => s,
        Ok(None) => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Invalid token"))
                .unwrap();
        },
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Failed to get submitter: {}", e)))
                .unwrap();
        }
    };
    
    // Get template
    let db_template = match crate::database::queries::TemplateQueries::get_template_by_id(pool, db_submitter.template_id).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Template not found"))
                .unwrap();
        },
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Failed to get template: {}", e)))
                .unwrap();
        }
    };
    
    // Get template with fields
    let template = match crate::routes::templates::convert_db_template_to_template_with_fields(db_template.clone(), pool).await {
        Ok(t) => t,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Failed to load template: {}", e)))
                .unwrap();
        }
    };
    
    // Get document URL
    let doc_url = match template.documents.as_ref().and_then(|docs| docs.get(0)) {
        Some(doc) => &doc.url,
        None => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Template has no document"))
                .unwrap();
        }
    };
    
    // Download PDF from storage
    let storage_service = match StorageService::new().await {
        Ok(s) => s,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Storage error: {}", e)))
                .unwrap();
        }
    };
    
    let pdf_bytes = match storage_service.download_file(doc_url).await {
        Ok(bytes) => bytes,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Failed to download PDF: {}", e)))
                .unwrap();
        }
    };
    
    // Get submission fields for this submitter (not template fields)
    let submission_fields = match SubmissionFieldQueries::get_submission_fields_by_submitter_id(pool, db_submitter.id).await {
        Ok(fields) => fields,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Failed to get submission fields: {}", e)))
                .unwrap();
        }
    };
    
    // Extract signatures from this submitter
    let mut signatures_to_render: Vec<(String, String, String, f64, f64, f64, f64, i32)> = Vec::new();
    
    if let Some(bulk_sigs) = &db_submitter.bulk_signatures {
        if let Some(sigs_array) = bulk_sigs.as_array() {
            for sig in sigs_array {
                let field_id = sig.get("field_id").and_then(|v| v.as_i64()).unwrap_or(0);
                let signature_value = sig.get("signature_value")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                
                // Find field info in submission fields (not template fields)
                if let Some(field) = submission_fields.iter().find(|f| f.id == field_id) {
                    if let Some(position_json) = &field.position {
                        // Parse position from JSON
                        if let Ok(position) = serde_json::from_value::<crate::models::template::FieldPosition>(position_json.clone()) {
                            signatures_to_render.push((
                                field.name.clone(),
                                field.field_type.clone(),
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
    }
    
    // Render signatures on PDF
    let signed_pdf_bytes = match render_signatures_on_pdf(&pdf_bytes, &signatures_to_render) {
        Ok(bytes) => bytes,
        Err(e) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(format!("Failed to render PDF: {}", e)))
                .unwrap();
        }
    };
    
    // Return PDF file directly
    let filename = format!("signed_{}_{}.pdf", template.slug, db_submitter.id);
    
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(header::CONTENT_DISPOSITION, format!("attachment; filename=\"{}\"", filename))
        .body(Body::from(signed_pdf_bytes))
        .unwrap()
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
