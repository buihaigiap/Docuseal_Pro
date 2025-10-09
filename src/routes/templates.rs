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
use chrono::Utc;
use axum_extra::extract::Multipart;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json;
use pdf_extract::extract_text_from_mem;
use base64;
// use docx_rs::DocxFile;

// fn extract_text_from_file(key: &str, data: &[u8]) -> Result<String, String> {
//     println!("Extracting text from file with key: {}" , key);
//     let key_lower = key.to_lowercase();
//     if key_lower.ends_with(".pdf") {
//         let result = extract_text_from_mem(data).map_err(|e| format!("Failed to extract text from PDF: {}", e));
//         println!("Extracting text from PDF file: {:?}", result);
//         result
//     } else if key_lower.ends_with(".docx") {
//         // TODO: Implement DOCX text extraction
//         Err("DOCX text extraction not yet implemented".to_string())
//     } else if key_lower.ends_with(".txt") {
//         String::from_utf8(data.to_vec()).map_err(|e| format!("Invalid UTF-8: {}", e))
//     } else {
//         Err("Unsupported file type for preview".to_string())
//     }
// }

fn get_content_type_from_filename(filename: &str) -> &'static str {
    let filename_lower = filename.to_lowercase();
    if filename_lower.ends_with(".pdf") {
        "application/pdf"
    } else if filename_lower.ends_with(".docx") {
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    } else if filename_lower.ends_with(".doc") {
        "application/msword"
    } else if filename_lower.ends_with(".txt") {
        "text/plain"
    } else if filename_lower.ends_with(".html") || filename_lower.ends_with(".htm") {
        "text/html"
    } else if filename_lower.ends_with(".jpg") || filename_lower.ends_with(".jpeg") {
        "image/jpeg"
    } else if filename_lower.ends_with(".png") {
        "image/png"
    } else if filename_lower.ends_with(".gif") {
        "image/gif"
    } else if filename_lower.ends_with(".webp") {
        "image/webp"
    } else if filename_lower.ends_with(".bmp") {
        "image/bmp"
    } else if filename_lower.ends_with(".tiff") || filename_lower.ends_with(".tif") {
        "image/tiff"
    } else {
        "application/octet-stream"
    }
}

use crate::common::responses::ApiResponse;
use crate::models::template::{
    Template, UpdateTemplateRequest, CloneTemplateRequest,
    CreateTemplateFromHtmlRequest, MergeTemplatesRequest,
    TemplateField, FieldPosition, SuggestedPosition,
    CreateTemplateFieldRequest, UpdateTemplateFieldRequest,
    FileUploadResponse, CreateTemplateFromFileRequest, CreateTemplateRequest
};
use crate::database::connection::DbPool;
use crate::database::models::{CreateTemplate, CreateTemplateField};
use crate::database::queries::TemplateQueries;
use crate::services::storage::StorageService;
use crate::common::jwt::auth_middleware;

pub type AppState = Arc<Mutex<DbPool>>;

#[utoipa::path(
    get,
    path = "/api/templates/{id}/full-info",
    params(
        ("id" = i64, Path, description = "Template ID")
    ),
    responses(
        (status = 200, description = "Template full information retrieved successfully", body = ApiResponse<serde_json::Value>),
        (status = 404, description = "Template not found", body = ApiResponse<serde_json::Value>),
        (status = 500, description = "Internal server error", body = ApiResponse<serde_json::Value>)
    ),
    security(("bearer_auth" = [])),
    tag = "templates"
)]
pub async fn get_template_full_info(
    State(state): State<AppState>,
    Path(template_id): Path<i64>,
    Extension(user_id): Extension<i64>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let pool = &*state.lock().await;

    // Verify template belongs to user
    match TemplateQueries::get_template_by_id(pool, template_id).await {
        Ok(Some(db_template)) => {
            if db_template.user_id != user_id {
                return ApiResponse::forbidden("Access denied".to_string());
            }

            // Convert template to API model with fields loaded
            let template = match convert_db_template_to_template_with_fields(db_template.clone(), pool).await {
                Ok(template) => template,
                Err(e) => return ApiResponse::internal_error(format!("Failed to load template fields: {}", e)),
            };

            // Get all submitters for this template directly
            match crate::database::queries::SubmitterQueries::get_submitters_by_template(pool, template_id).await {
                Ok(db_submitters) => {
                    let submitters = db_submitters.into_iter().map(|db_sub| crate::models::submitter::Submitter {
                        id: Some(db_sub.id),
                        template_id: Some(db_sub.template_id),
                        user_id: Some(db_sub.user_id),
                        name: db_sub.name,
                        email: db_sub.email,
                        status: db_sub.status,
                        signed_at: db_sub.signed_at,
                        token: db_sub.token,
                        bulk_signatures: db_sub.bulk_signatures,
                        created_at: db_sub.created_at,
                        updated_at: db_sub.updated_at,
                    }).collect::<Vec<_>>();

                    let data = serde_json::json!({
                        "template": template,
                        "submitters": submitters,
                        "total_submitters": submitters.len()
                    });

                    ApiResponse::success(data, "Template full information retrieved successfully".to_string())
                }
                Err(e) => ApiResponse::internal_error(format!("Failed to get submitters: {}", e)),
            }
        }
        Ok(None) => ApiResponse::not_found("Template not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to retrieve template: {}", e)),
    }
}

pub fn create_template_router() -> Router<AppState> {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/files/preview/*key", get(preview_file));

    // Authenticated routes
    let auth_routes = Router::new()
        .route("/templates", get(get_templates))
        .route("/templates", post(create_template))
        .route("/templates/:id", get(get_template))
        .route("/templates/:id/full-info", get(get_template_full_info))
        .route("/templates/:id", put(update_template))
        .route("/templates/:id", delete(delete_template))
        .route("/templates/:id/clone", post(clone_template))
        .route("/templates/html", post(create_template_from_html))
        .route("/templates/pdf", post(create_template_from_pdf))
        .route("/templates/docx", post(create_template_from_docx))
        .route("/templates/from-file", post(create_template_from_file))
        .route("/templates/merge", post(merge_templates))
        // Template Fields routes
        .route("/templates/:template_id/fields", get(get_template_fields))
        .route("/templates/:template_id/fields", post(create_template_field))
        .route("/templates/:template_id/fields/:field_id", put(update_template_field))
        .route("/templates/:template_id/fields/:field_id", delete(delete_template_field))
        // File upload must come before wildcard route
        .route("/files/upload", post(upload_file))
        .route("/files/*key", get(download_file))
        .layer(middleware::from_fn(auth_middleware));

    // Merge public and authenticated routes
    public_routes.merge(auth_routes)
}

#[utoipa::path(
    get,
    path = "/api/templates",
    responses(
        (status = 200, description = "List all templates", body = ApiResponse<Vec<Template>>),
        (status = 500, description = "Internal server error", body = ApiResponse<Vec<Template>>)
    ),
    security(("bearer_auth" = [])),
    tag = "templates"
)]
pub async fn get_templates(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
) -> (StatusCode, Json<ApiResponse<Vec<Template>>>) {
    let pool = &*state.lock().await;

    match TemplateQueries::get_all_templates(pool, user_id).await {
        Ok(db_templates) => {
            let mut templates = Vec::new();
            for db_template in db_templates {
                let template = convert_db_template_to_template_without_fields(db_template);
                templates.push(template);
            }
            ApiResponse::success(templates, "Templates retrieved successfully".to_string())
        }
        Err(e) => ApiResponse::internal_error(format!("Failed to retrieve templates: {}", e)),
    }
}

#[utoipa::path(
    get,
    path = "/api/templates/{id}",
    params(
        ("id" = i64, Path, description = "Template ID")
    ),
    responses(
        (status = 200, description = "Template found", body = ApiResponse<Template>),
        (status = 404, description = "Template not found", body = ApiResponse<Template>),
        (status = 500, description = "Internal server error", body = ApiResponse<Template>)
    ),
    security(("bearer_auth" = [])),
    tag = "templates"
)]
pub async fn get_template(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Extension(user_id): Extension<i64>,
) -> (StatusCode, Json<ApiResponse<Template>>) {
    let pool = &*state.lock().await;

    match TemplateQueries::get_template_by_id(pool, id).await {
        Ok(Some(db_template)) => {
            // Check if template belongs to user
            if db_template.user_id != user_id {
                return ApiResponse::not_found("Template not found".to_string());
            }
            match convert_db_template_to_template_with_fields(db_template, pool).await {
                Ok(template) => ApiResponse::success(template, "Template retrieved successfully".to_string()),
                Err(e) => ApiResponse::internal_error(format!("Failed to load template fields: {}", e)),
            }
        }
        Ok(None) => ApiResponse::not_found("Template not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to retrieve template: {}", e)),
    }
}

#[utoipa::path(
    put,
    path = "/api/templates/{id}",
    params(
        ("id" = i64, Path, description = "Template ID")
    ),
    request_body = UpdateTemplateRequest,
    responses(
        (status = 200, description = "Template updated successfully", body = ApiResponse<Template>),
        (status = 404, description = "Template not found", body = ApiResponse<Template>),
        (status = 500, description = "Internal server error", body = ApiResponse<Template>)
    ),
    security(("bearer_auth" = [])),
    tag = "templates"
)]
pub async fn update_template(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<UpdateTemplateRequest>,
) -> (StatusCode, Json<ApiResponse<Template>>) {
    let pool = &*state.lock().await;

    // Update template (fields are managed separately via template_fields endpoints)
    match TemplateQueries::update_template(pool, id, user_id, payload.name.as_deref()).await {
        Ok(Some(db_template)) => {
            match convert_db_template_to_template_with_fields(db_template, pool).await {
                Ok(template) => ApiResponse::success(template, "Template updated successfully".to_string()),
                Err(e) => ApiResponse::internal_error(format!("Failed to load template fields: {}", e)),
            }
        }
        Ok(None) => ApiResponse::not_found("Template not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to update template: {}", e)),
    }
}

#[utoipa::path(
    delete,
    path = "/api/templates/{id}",
    params(
        ("id" = i64, Path, description = "Template ID")
    ),
    responses(
        (status = 200, description = "Template deleted successfully", body = ApiResponse<String>),
        (status = 404, description = "Template not found", body = ApiResponse<String>),
        (status = 500, description = "Internal server error", body = ApiResponse<String>)
    ),
    security(("bearer_auth" = [])),
    tag = "templates"
)]
pub async fn delete_template(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Extension(user_id): Extension<i64>,
) -> (StatusCode, Json<ApiResponse<String>>) {
    let pool = &*state.lock().await;

    match TemplateQueries::delete_template(pool, id, user_id).await {
        Ok(true) => ApiResponse::success("Template deleted successfully".to_string(), "Template deleted successfully".to_string()),
        Ok(false) => ApiResponse::not_found("Template not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to delete template: {}", e)),
    }
}

#[utoipa::path(
    post,
    path = "/api/templates/{id}/clone",
    params(
        ("id" = i64, Path, description = "Template ID to clone")
    ),
    request_body = CloneTemplateRequest,
    responses(
        (status = 201, description = "Template cloned successfully", body = ApiResponse<Template>),
        (status = 404, description = "Original template not found", body = ApiResponse<Template>),
        (status = 500, description = "Internal server error", body = ApiResponse<Template>)
    ),
    security(("bearer_auth" = [])),
    tag = "templates"
)]
pub async fn clone_template(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<CloneTemplateRequest>,
) -> (StatusCode, Json<ApiResponse<Template>>) {
    let pool = &*state.lock().await;

    // Generate a unique slug for the cloned template
    let slug = format!("{}-clone-{}", payload.name.to_lowercase().replace(" ", "-"), chrono::Utc::now().timestamp());

    match TemplateQueries::clone_template(pool, id, user_id, &payload.name, &slug).await {
        Ok(Some(db_template)) => {
            // Clone template fields from original template
            use crate::database::queries::TemplateFieldQueries;
            let _ = TemplateFieldQueries::clone_template_fields(pool, id, db_template.id).await;

            match convert_db_template_to_template_with_fields(db_template, pool).await {
                Ok(template) => ApiResponse::created(template, "Template cloned successfully".to_string()),
                Err(e) => ApiResponse::internal_error(format!("Failed to load template fields: {}", e)),
            }
        }
        Ok(None) => ApiResponse::not_found("Original template not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to clone template: {}", e)),
    }
}

#[utoipa::path(
    post,
    path = "/api/templates",
    request_body = CreateTemplateRequest,
    responses(
        (status = 201, description = "Template created successfully", body = ApiResponse<Template>),
        (status = 500, description = "Internal server error", body = ApiResponse<Template>)
    ),
    security(("bearer_auth" = [])),
    tag = "templates"
)]
pub async fn create_template(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<CreateTemplateRequest>,
) -> (StatusCode, Json<ApiResponse<Template>>) {
    let pool = &*state.lock().await;

    // Decode base64 document
    let document_data = match base64::decode(&payload.document) {
        Ok(data) => data,
        Err(e) => return ApiResponse::bad_request(format!("Invalid base64 document: {}", e)),
    };

    // Initialize storage service
    let storage = match StorageService::new().await {
        Ok(storage) => storage,
        Err(e) => return ApiResponse::internal_error(format!("Failed to initialize storage: {}", e)),
    };

    // Generate filename and upload document
    let filename = format!("{}.txt", payload.name.to_lowercase().replace(" ", "_"));
    let file_key = match storage.upload_file(document_data, &filename, "text/plain").await {
        Ok(key) => key,
        Err(e) => return ApiResponse::internal_error(format!("Failed to upload document: {}", e)),
    };

    // Generate unique slug
    let slug = format!("template-{}-{}", payload.name.to_lowercase().replace(" ", "-"), chrono::Utc::now().timestamp());

    // Create template in database
    let create_template = CreateTemplate {
        name: payload.name.clone(),
        slug: slug.clone(),
        user_id: user_id,
        documents: Some(serde_json::json!([{
            "filename": filename,
            "content_type": "text/plain",
            "size": 0,
            "url": file_key
        }])),
    };

    match TemplateQueries::create_template(pool, create_template).await {
        Ok(db_template) => {
            let template_id = db_template.id;

            // Create fields if provided
            if let Some(fields) = payload.fields {
                for field_req in fields {
                    let create_field = CreateTemplateField {
                        template_id,
                        name: field_req.name,
                        field_type: field_req.field_type,
                        required: field_req.required,
                        display_order: field_req.display_order.unwrap_or(0),
                        position: field_req.position.map(|p| serde_json::to_value(p).unwrap_or(serde_json::Value::Null)),
                        options: field_req.options.map(|o| serde_json::to_value(o).unwrap_or(serde_json::Value::Null)),
                        metadata: None,
                    };

                    if let Err(e) = crate::database::queries::TemplateFieldQueries::create_template_field(pool, create_field).await {
                        // Try to clean up template if field creation fails
                        let _ = TemplateQueries::delete_template(pool, template_id, user_id).await;
                        return ApiResponse::internal_error(format!("Failed to create template field: {}", e));
                    }
                }
            }

            match convert_db_template_to_template_with_fields(db_template, pool).await {
                Ok(template) => ApiResponse::created(template, "Template created successfully".to_string()),
                Err(e) => {
                    // Try to clean up template if loading fields fails
                    let _ = TemplateQueries::delete_template(pool, template_id, user_id).await;
                    ApiResponse::internal_error(format!("Failed to load template fields: {}", e))
                }
            }
        }
        Err(e) => {
            ApiResponse::internal_error(format!("Failed to create template: {}", e))
        }
    }
}

// Placeholder handlers for creating templates from different sources
// These would need actual implementation for PDF/HTML processing

#[utoipa::path(
    post,
    path = "/api/templates/html",
    request_body = CreateTemplateFromHtmlRequest,
    responses(
        (status = 201, description = "Template created from HTML", body = ApiResponse<Template>),
        (status = 500, description = "Internal server error", body = ApiResponse<Template>)
    ),
    security(("bearer_auth" = [])),
    tag = "templates"
)]
pub async fn create_template_from_html(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<CreateTemplateFromHtmlRequest>,
) -> (StatusCode, Json<ApiResponse<Template>>) {
    let pool = &*state.lock().await;

    // Initialize storage service
    let storage = match StorageService::new().await {
        Ok(storage) => {
            eprintln!("Storage initialized successfully");
            storage
        },
        Err(e) => {
            eprintln!("Storage init error: {:?}", e);
            // For now, skip storage and just create template in DB
            eprintln!("Skipping storage upload, creating template in DB only");
            return create_template_without_storage(pool, payload, user_id).await;
        }
    };

    // Convert HTML to bytes
    let html_data = payload.html.as_bytes().to_vec();
    let filename = format!("{}.html", payload.name.to_lowercase().replace(" ", "_"));

    // Upload HTML file to storage
    let file_key = match storage.upload_file(html_data, &filename, "text/html").await {
        Ok(key) => key,
        Err(e) => return ApiResponse::internal_error(format!("Failed to upload HTML file: {}", e)),
    };

    // Generate unique slug
    let slug = format!("html-{}-{}", payload.name.to_lowercase().replace(" ", "-"), chrono::Utc::now().timestamp());

    // Create template in database
    let create_template = CreateTemplate {
        name: payload.name.clone(),
        slug: slug.clone(),
        user_id: user_id,
        // fields: None, // Removed - fields will be added separately
        documents: None, // Skip documents for now
    };

    match TemplateQueries::create_template(pool, create_template).await {
        Ok(db_template) => {
            match convert_db_template_to_template_with_fields(db_template, pool).await {
                Ok(template) => ApiResponse::created(template, "Template created from HTML successfully".to_string()),
                Err(e) => {
                    // Try to delete uploaded file if database operation fails
                    let _ = storage.delete_file(&file_key).await;
                    ApiResponse::internal_error(format!("Failed to load template fields: {}", e))
                }
            }
        }
        Err(e) => {
            // Try to delete uploaded file if database operation fails
            let _ = storage.delete_file(&file_key).await;
            ApiResponse::internal_error(format!("Failed to create template: {}", e))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/templates/pdf",
    request_body = CreateTemplateFromPdfRequest,
    responses(
        (status = 201, description = "Template created from PDF", body = ApiResponse<Template>),
        (status = 500, description = "Internal server error", body = ApiResponse<Template>)
    ),
    security(("bearer_auth" = [])),
    tag = "templates"
)]
pub async fn create_template_from_pdf(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    mut multipart: Multipart,
) -> (StatusCode, Json<ApiResponse<Template>>) {
    let pool = &*state.lock().await;

    // Initialize storage service
    let storage = match StorageService::new().await {
        Ok(storage) => storage,
        Err(e) => return ApiResponse::internal_error(format!("Failed to initialize storage: {}", e)),
    };

    let mut pdf_data = Vec::new();
    let mut filename = String::new();
    let mut template_name = String::new();

    // Parse multipart form data
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "pdf" => {
                filename = field.file_name().unwrap_or("template.pdf").to_string();
                pdf_data = field.bytes().await.unwrap_or_default().to_vec();
            }
            "name" => {
                template_name = String::from_utf8(field.bytes().await.unwrap_or_default().to_vec())
                    .unwrap_or_else(|_| "Untitled Template".to_string());
            }
            _ => {}
        }
    }

    if pdf_data.is_empty() {
        return ApiResponse::bad_request("PDF file is required".to_string());
    }

    if template_name.is_empty() {
        template_name = "PDF Template".to_string();
    }

    // Upload file to storage
    let file_key = match storage.upload_file(pdf_data, &filename, "application/pdf").await {
        Ok(key) => key,
        Err(e) => return ApiResponse::internal_error(format!("Failed to upload file: {}", e)),
    };

    // Generate unique slug
    let slug = format!("pdf-{}-{}", template_name.to_lowercase().replace(" ", "-"), chrono::Utc::now().timestamp());

    // Create template in database
    let create_template = CreateTemplate {
        name: template_name.clone(),
        slug: slug.clone(),
        user_id: user_id,
        // fields: None, // TODO: Extract fields from PDF - REMOVED
        documents: Some(serde_json::json!([{
            "filename": filename,
            "content_type": "application/pdf",
            "size": 0,
            "url": file_key
        }])),
    };

    match TemplateQueries::create_template(pool, create_template).await {
        Ok(db_template) => {
            match convert_db_template_to_template_with_fields(db_template, pool).await {
                Ok(template) => ApiResponse::created(template, "Template created from PDF successfully".to_string()),
                Err(e) => {
                    // Try to delete uploaded file if database operation fails
                    let _ = storage.delete_file(&file_key).await;
                    ApiResponse::internal_error(format!("Failed to load template fields: {}", e))
                }
            }
        }
        Err(e) => {
            // Try to delete uploaded file if database operation fails
            let _ = storage.delete_file(&file_key).await;
            ApiResponse::internal_error(format!("Failed to create template: {}", e))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/templates/docx",
    request_body = CreateTemplateFromDocxRequest,
    responses(
        (status = 201, description = "Template created from DOCX", body = ApiResponse<Template>),
        (status = 500, description = "Internal server error", body = ApiResponse<Template>)
    ),
    security(("bearer_auth" = [])),
    tag = "templates"
)]
pub async fn create_template_from_docx(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    mut multipart: Multipart,
) -> (StatusCode, Json<ApiResponse<Template>>) {
    let pool = &*state.lock().await;

    // Initialize storage service
    let storage = match StorageService::new().await {
        Ok(storage) => storage,
        Err(e) => return ApiResponse::internal_error(format!("Failed to initialize storage: {}", e)),
    };

    let mut docx_data = Vec::new();
    let mut filename = String::new();
    let mut template_name = String::new();

    // Parse multipart form data
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "docx" => {
                filename = field.file_name().unwrap_or("template.docx").to_string();
                docx_data = field.bytes().await.unwrap_or_default().to_vec();
            }
            "name" => {
                template_name = String::from_utf8(field.bytes().await.unwrap_or_default().to_vec())
                    .unwrap_or_else(|_| "Untitled Template".to_string());
            }
            _ => {}
        }
    }

    if docx_data.is_empty() {
        return ApiResponse::bad_request("DOCX file is required".to_string());
    }

    if template_name.is_empty() {
        template_name = "DOCX Template".to_string();
    }

    // Upload file to storage
    let file_key = match storage.upload_file(docx_data, &filename, "application/vnd.openxmlformats-officedocument.wordprocessingml.document").await {
        Ok(key) => key,
        Err(e) => return ApiResponse::internal_error(format!("Failed to upload file: {}", e)),
    };

    // Generate unique slug
    let slug = format!("docx-{}-{}", template_name.to_lowercase().replace(" ", "-"), chrono::Utc::now().timestamp());

    // Create template in database
    let create_template = CreateTemplate {
        name: template_name.clone(),
        slug: slug.clone(),
        user_id: user_id,
        // fields: None, // TODO: Extract fields from DOCX - REMOVED
        documents: Some(serde_json::json!([{
            "filename": filename,
            "content_type": "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "size": 0, // TODO: Get actual file size
            "url": file_key
        }])),
    };

    match TemplateQueries::create_template(pool, create_template).await {
        Ok(db_template) => {
            match convert_db_template_to_template_with_fields(db_template, pool).await {
                Ok(template) => ApiResponse::created(template, "Template created from DOCX successfully".to_string()),
                Err(e) => {
                    // Try to delete uploaded file if database operation fails
                    let _ = storage.delete_file(&file_key).await;
                    ApiResponse::internal_error(format!("Failed to load template fields: {}", e))
                }
            }
        }
        Err(e) => {
            // Try to delete uploaded file if database operation fails
            let _ = storage.delete_file(&file_key).await;
            ApiResponse::internal_error(format!("Failed to create template: {}", e))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/templates/merge",
    request_body = MergeTemplatesRequest,
    responses(
        (status = 201, description = "Templates merged successfully", body = ApiResponse<Template>),
        (status = 500, description = "Internal server error", body = ApiResponse<Template>)
    ),
    security(("bearer_auth" = [])),
    tag = "templates"
)]
pub async fn merge_templates(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(_payload): Json<MergeTemplatesRequest>,
) -> (StatusCode, Json<ApiResponse<Template>>) {
    // TODO: Implement template merging logic
    ApiResponse::internal_error("Template merging not yet implemented".to_string())
}

// Helper function to create template without storage (for testing)
async fn create_template_without_storage(
    pool: &sqlx::PgPool,
    payload: CreateTemplateFromHtmlRequest,
    user_id: i64,
) -> (StatusCode, Json<ApiResponse<Template>>) {
    // Generate unique slug
    let slug = format!("html-{}-{}", payload.name.to_lowercase().replace(" ", "-"), chrono::Utc::now().timestamp());

    // Create template in database
    let create_template = CreateTemplate {
        name: payload.name.clone(),
        slug: slug.clone(),
        user_id: user_id,
        // fields: None, // Removed - fields will be added separately
        documents: None, // Skip documents for now
    };

    match TemplateQueries::create_template(pool, create_template).await {
        Ok(db_template) => {
            match convert_db_template_to_template_with_fields(db_template, pool).await {
                Ok(template) => ApiResponse::created(template, "Template created from HTML successfully (without storage)".to_string()),
                Err(e) => ApiResponse::internal_error(format!("Failed to load template fields: {}", e)),
            }
        }
        Err(e) => {
            ApiResponse::internal_error(format!("Failed to create template: {}", e))
        }
    }
}

// Helper function to convert database template to API template (sync version for simple cases)
pub fn convert_db_template_to_template(db_template: crate::database::models::DbTemplate) -> Template {
    Template {
        id: db_template.id,
        name: db_template.name,
        slug: db_template.slug,
        user_id: db_template.user_id,
        template_fields: None, // Will be loaded separately if needed
        submitters: None, // No longer stored in templates
        documents: db_template.documents.and_then(|v| serde_json::from_value(v).ok()),
        created_at: db_template.created_at,
        updated_at: db_template.updated_at,
    }
}

// Helper function to convert database template to API template with fields loaded (async)
pub fn convert_db_template_to_template_without_fields(
    db_template: crate::database::models::DbTemplate,
) -> Template {
    Template {
        id: db_template.id,
        name: db_template.name,
        slug: db_template.slug,
        user_id: db_template.user_id,
        template_fields: None,
        submitters: None,
        documents: db_template.documents.and_then(|v| serde_json::from_value(v).ok()),
        created_at: db_template.created_at,
        updated_at: db_template.updated_at,
    }
}

pub async fn convert_db_template_to_template_with_fields(
    db_template: crate::database::models::DbTemplate,
    pool: &sqlx::PgPool
) -> Result<Template, sqlx::Error> {
    use crate::database::queries::TemplateFieldQueries;

    let template_fields = TemplateFieldQueries::get_template_fields(pool, db_template.id).await?
        .into_iter()
        .map(|db_field| TemplateField {
            id: db_field.id,
            template_id: db_field.template_id,
            name: db_field.name,
            field_type: db_field.field_type,
            required: db_field.required,
            display_order: db_field.display_order,
            position: db_field.position.and_then(|v| serde_json::from_value(v).ok()),
            options: db_field.options.and_then(|v| serde_json::from_value(v).ok()),
            created_at: db_field.created_at,
            updated_at: db_field.updated_at,
        })
        .collect::<Vec<_>>();

    Ok(Template {
        id: db_template.id,
        name: db_template.name,
        slug: db_template.slug,
        user_id: db_template.user_id,
        template_fields: Some(template_fields),
        submitters: None, // No longer stored in templates
        documents: db_template.documents.and_then(|v| serde_json::from_value(v).ok()),
        created_at: db_template.created_at,
        updated_at: db_template.updated_at,
    })
}

#[utoipa::path(
    get,
    path = "/api/files/{key}",
    params(
        ("key" = String, Path, description = "File path in storage (e.g., 'templates/1759746273_test.pdf')")
    ),
    responses(
        (status = 200, description = "File downloaded successfully"),
        (status = 404, description = "File not found")
    ),
    security(("bearer_auth" = [])),
    tag = "files"
)]
pub async fn download_file(
    Path(key): Path<String>,
    Extension(_user_id): Extension<i64>,
) -> Response<Body> {
    // Initialize storage service
    let storage = match StorageService::new().await {
        Ok(storage) => storage,
        Err(_) => {
            // Return default PDF on storage error
            const DEFAULT_PDF: &[u8] = b"%PDF-1.4\n1 0 obj\n<<\n/Type /Catalog\n/Pages 2 0 R\n>>\nendobj\n2 0 obj\n<<\n/Type /Pages\n/Kids [3 0 R]\n/Count 1\n>>\nendobj\n3 0 obj\n<<\n/Type /Page\n/Parent 2 0 R\n/MediaBox [0 0 612 792]\n/Contents 4 0 R\n>>\nendobj\n4 0 obj\n<<\n/Length 0\n>>\nstream\n\nendstream\nendobj\nxref\n0 5\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000170 00000 n \ntrailer\n<<\n/Size 5\n/Root 1 0 R\n>>\nstartxref\n226\n%%EOF";
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/pdf")
                .header(header::CONTENT_DISPOSITION, format!("inline; filename=\"{}\"", key))
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Expose-Headers", "*")
                .header("Content-Length", DEFAULT_PDF.len().to_string())
                .body(Body::from(DEFAULT_PDF.to_vec()))
                .unwrap();
            return response;
        }
    };

    // Download file from storage
    let file_data = match storage.download_file(&key).await {
        Ok(data) => data,
        Err(_) => {
            println!("File not found in storage: {}", key);
            // Return 404 Not Found response
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::CONTENT_TYPE, "text/plain")
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Expose-Headers", "*")
                .body(Body::from("File not found"))
                .unwrap();
            return response;
        }
    };
    // Determine content type based on file extension
    let content_type = get_content_type_from_filename(&key);

    // Create response with file data
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_DISPOSITION, format!("inline; filename=\"{}\"", key))
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Expose-Headers", "*")
        .header("Content-Length", file_data.len().to_string())
        .body(Body::from(file_data))
        .unwrap();

    response
}

#[utoipa::path(
    get,
    path = "/api/files/preview/{key}",
    params(
        ("key" = String, Path, description = "File path in storage (e.g., 'templates/test.pdf' or 'templates/test.docx')")
    ),
    responses(
        (status = 200, description = "File preview returned successfully (binary file data for frontend display)"),
        (status = 404, description = "File not found"),
        (status = 500, description = "Internal server error")
    ),
    tag = "files"
)]
pub async fn preview_file(
    Path(key): Path<String>,
) -> Response<Body> {
    // Initialize storage service
    let storage = match StorageService::new().await {
        Ok(storage) => storage,
        Err(_) => {
            // Return default PDF on storage error
            const DEFAULT_PDF: &[u8] = b"%PDF-1.4\n1 0 obj\n<<\n/Type /Catalog\n/Pages 2 0 R\n>>\nendobj\n2 0 obj\n<<\n/Type /Pages\n/Kids [3 0 R]\n/Count 1\n>>\nendobj\n3 0 obj\n<<\n/Type /Page\n/Parent 2 0 R\n/MediaBox [0 0 612 792]\n/Contents 4 0 R\n>>\nendobj\n4 0 obj\n<<\n/Length 0\n>>\nstream\n\nendstream\nendobj\nxref\n0 5\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n0000000115 00000 n \n0000000170 00000 n \ntrailer\n<<\n/Size 5\n/Root 1 0 R\n>>\nstartxref\n226\n%%EOF";
            let response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/pdf")
                .header(header::CONTENT_DISPOSITION, format!("inline; filename=\"{}\"", key))
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Expose-Headers", "*")
                .header("Content-Length", DEFAULT_PDF.len().to_string())
                .body(Body::from(DEFAULT_PDF.to_vec()))
                .unwrap();
            return response;
        }
    };

    // Download file from storage
    let file_data = match storage.download_file(&key).await {
        Ok(data) => data,
        Err(_) => {
            println!("File not found in storage: {}", key);
            // Return 404 Not Found response
            let response = Response::builder()
                .status(StatusCode::NOT_FOUND)
                .header(header::CONTENT_TYPE, "text/plain")
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Expose-Headers", "*")
                .body(Body::from("File not found"))
                .unwrap();
            return response;
        }
    };

    // Determine content type based on file extension
    let content_type = get_content_type_from_filename(&key);

    // Create response with file data for preview (inline display)
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_DISPOSITION, format!("inline; filename=\"{}\"", key))
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Expose-Headers", "*")
        .header("Content-Length", file_data.len().to_string())
        .body(Body::from(file_data))
        .unwrap();

    response
}

// ===== TEMPLATE FIELDS ENDPOINTS =====

#[utoipa::path(
    get,
    path = "/api/templates/{template_id}/fields",
    params(
        ("template_id" = i64, Path, description = "Template ID")
    ),
    responses(
        (status = 200, description = "Template fields retrieved successfully", body = ApiResponse<Vec<TemplateField>>),
        (status = 404, description = "Template not found", body = ApiResponse<Vec<TemplateField>>),
        (status = 500, description = "Internal server error", body = ApiResponse<Vec<TemplateField>>)
    ),
    security(("bearer_auth" = [])),
    tag = "template_fields"
)]
pub async fn get_template_fields(
    State(state): State<AppState>,
    Path(template_id): Path<i64>,
    Extension(user_id): Extension<i64>,
) -> (StatusCode, Json<ApiResponse<Vec<TemplateField>>>) {
    let pool = &*state.lock().await;

    // Verify template belongs to user
    match TemplateQueries::get_template_by_id(pool, template_id).await {
        Ok(Some(db_template)) => {
            if db_template.user_id != user_id {
                return ApiResponse::not_found("Template not found".to_string());
            }
        }
        Ok(None) => return ApiResponse::not_found("Template not found".to_string()),
        Err(e) => return ApiResponse::internal_error(format!("Failed to verify template: {}", e)),
    }

    match crate::database::queries::TemplateFieldQueries::get_template_fields(pool, template_id).await {
        Ok(fields) => {
            let template_fields: Vec<TemplateField> = fields.into_iter()
                .map(|db_field| TemplateField {
                    id: db_field.id,
                    template_id: db_field.template_id,
                    name: db_field.name,
                    field_type: db_field.field_type,
                    required: db_field.required,
                    display_order: db_field.display_order,
                    position: db_field.position.and_then(|v| serde_json::from_value(v).ok()),
                    options: db_field.options.and_then(|v| serde_json::from_value(v).ok()),
                    created_at: db_field.created_at,
                    updated_at: db_field.updated_at,
                })
                .collect();

            ApiResponse::success(template_fields, "Template fields retrieved successfully".to_string())
        }
        Err(e) => ApiResponse::internal_error(format!("Failed to retrieve template fields: {}", e)),
    }
}

#[utoipa::path(
    post,
    path = "/api/templates/{template_id}/fields",
    params(
        ("template_id" = i64, Path, description = "Template ID")
    ),
    request_body = CreateTemplateFieldRequest,
    responses(
        (status = 201, description = "Template field(s) created successfully", body = ApiResponse<Vec<TemplateField>>),
        (status = 404, description = "Template not found", body = ApiResponse<Vec<TemplateField>>),
        (status = 500, description = "Internal server error", body = ApiResponse<Vec<TemplateField>>)
    ),
    security(("bearer_auth" = [])),
    tag = "template_fields"
)]
pub async fn create_template_field(
    State(state): State<AppState>,
    Path(template_id): Path<i64>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<serde_json::Value>,
) -> (StatusCode, Json<ApiResponse<Vec<TemplateField>>>) {
    let pool = &*state.lock().await;

    // Verify template belongs to user
    match TemplateQueries::get_template_by_id(pool, template_id).await {
        Ok(Some(db_template)) => {
            if db_template.user_id != user_id {
                return ApiResponse::not_found("Template not found".to_string());
            }
        }
        Ok(None) => return ApiResponse::not_found("Template not found".to_string()),
        Err(e) => return ApiResponse::internal_error(format!("Failed to verify template: {}", e)),
    }

    // Check if it's bulk request (has "fields" array) or single field
    let field_requests: Vec<CreateTemplateFieldRequest> = if let Some(fields) = payload.get("fields") {
        if let Some(fields_array) = fields.as_array() {
            fields_array.iter()
                .filter_map(|f| serde_json::from_value(f.clone()).ok())
                .collect()
        } else {
            return ApiResponse::bad_request("Invalid fields format".to_string());
        }
    } else {
        // Single field request
        match serde_json::from_value::<CreateTemplateFieldRequest>(payload) {
            Ok(field) => vec![field],
            Err(_) => return ApiResponse::bad_request("Invalid field format".to_string()),
        }
    };

    if field_requests.is_empty() {
        return ApiResponse::bad_request("No fields provided".to_string());
    }

    let mut created_fields = Vec::new();

    for field_req in field_requests {
        let create_field = CreateTemplateField {
            template_id,
            name: field_req.name,
            field_type: field_req.field_type,
            required: field_req.required,
            display_order: field_req.display_order.unwrap_or(0),
            position: field_req.position.map(|p| serde_json::to_value(p).unwrap_or(serde_json::Value::Null)),
            options: field_req.options.map(|o| serde_json::to_value(o).unwrap_or(serde_json::Value::Null)),
            metadata: None,
        };

        match crate::database::queries::TemplateFieldQueries::create_template_field(pool, create_field).await {
            Ok(db_field) => {
                let template_field = TemplateField {
                    id: db_field.id,
                    template_id: db_field.template_id,
                    name: db_field.name,
                    field_type: db_field.field_type,
                    required: db_field.required,
                    display_order: db_field.display_order,
                    position: db_field.position.and_then(|v| serde_json::from_value(v).ok()),
                    options: db_field.options.and_then(|v| serde_json::from_value(v).ok()),
                    created_at: db_field.created_at,
                    updated_at: db_field.updated_at,
                };
                created_fields.push(template_field);
            }
            Err(e) => return ApiResponse::internal_error(format!("Failed to create template field: {}", e)),
        }
    }

    ApiResponse::created(created_fields, "Template fields created successfully".to_string())
}

#[utoipa::path(
    put,
    path = "/api/templates/{template_id}/fields/{field_id}",
    params(
        ("template_id" = i64, Path, description = "Template ID"),
        ("field_id" = i64, Path, description = "Field ID")
    ),
    request_body = UpdateTemplateFieldRequest,
    responses(
        (status = 200, description = "Template field updated successfully", body = ApiResponse<TemplateField>),
        (status = 404, description = "Template field not found", body = ApiResponse<TemplateField>),
        (status = 500, description = "Internal server error", body = ApiResponse<TemplateField>)
    ),
    security(("bearer_auth" = [])),
    tag = "template_fields"
)]
pub async fn update_template_field(
    State(state): State<AppState>,
    Path((template_id, field_id)): Path<(i64, i64)>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<UpdateTemplateFieldRequest>,
) -> (StatusCode, Json<ApiResponse<TemplateField>>) {
    let pool = &*state.lock().await;

    // Verify template belongs to user
    match TemplateQueries::get_template_by_id(pool, template_id).await {
        Ok(Some(db_template)) => {
            if db_template.user_id != user_id {
                return ApiResponse::not_found("Template not found".to_string());
            }
        }
        Ok(None) => return ApiResponse::not_found("Template not found".to_string()),
        Err(e) => return ApiResponse::internal_error(format!("Failed to verify template: {}", e)),
    }

    let update_field = CreateTemplateField {
        template_id,
        name: payload.name.unwrap_or_else(|| "temp".to_string()),
        field_type: payload.field_type.unwrap_or_else(|| "text".to_string()),
        required: payload.required.unwrap_or(false),
        display_order: payload.display_order.unwrap_or(0),
        position: payload.position.map(|p| serde_json::to_value(p).unwrap_or(serde_json::Value::Null)),
        options: payload.options.map(|o| serde_json::to_value(o).unwrap_or(serde_json::Value::Null)),
        metadata: None,
    };

    match crate::database::queries::TemplateFieldQueries::update_template_field(pool, field_id, update_field).await {
        Ok(Some(db_field)) => {
            let template_field = TemplateField {
                id: db_field.id,
                template_id: db_field.template_id,
                name: db_field.name,
                field_type: db_field.field_type,
                required: db_field.required,
                display_order: db_field.display_order,
                position: db_field.position.and_then(|v| serde_json::from_value(v).ok()),
                options: db_field.options.and_then(|v| serde_json::from_value(v).ok()),
                created_at: db_field.created_at,
                updated_at: db_field.updated_at,
            };

            ApiResponse::success(template_field, "Template field updated successfully".to_string())
        }
        Ok(None) => ApiResponse::not_found("Template field not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to update template field: {}", e)),
    }
}

#[utoipa::path(
    delete,
    path = "/api/templates/{template_id}/fields/{field_id}",
    params(
        ("template_id" = i64, Path, description = "Template ID"),
        ("field_id" = i64, Path, description = "Field ID")
    ),
    responses(
        (status = 200, description = "Template field deleted successfully", body = ApiResponse<serde_json::Value>),
        (status = 404, description = "Template field not found", body = ApiResponse<serde_json::Value>),
        (status = 500, description = "Internal server error", body = ApiResponse<serde_json::Value>)
    ),
    security(("bearer_auth" = [])),
    tag = "template_fields"
)]
pub async fn delete_template_field(
    State(state): State<AppState>,
    Path((template_id, field_id)): Path<(i64, i64)>,
    Extension(user_id): Extension<i64>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let pool = &*state.lock().await;

    // Verify template belongs to user
    match TemplateQueries::get_template_by_id(pool, template_id).await {
        Ok(Some(db_template)) => {
            if db_template.user_id != user_id {
                return ApiResponse::not_found("Template not found".to_string());
            }
        }
        Ok(None) => return ApiResponse::not_found("Template not found".to_string()),
        Err(e) => return ApiResponse::internal_error(format!("Failed to verify template: {}", e)),
    }

    match crate::database::queries::TemplateFieldQueries::delete_template_field(pool, field_id).await {
        Ok(true) => ApiResponse::success(serde_json::json!({"deleted": true}), "Template field deleted successfully".to_string()),
        Ok(false) => ApiResponse::not_found("Template field not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to delete template field: {}", e)),
    }
}

// ===== FILE UPLOAD ENDPOINT =====

#[utoipa::path(
    post,
    path = "/api/files/upload",
    request_body = FileUploadRequest,
    responses(
        (status = 201, description = "File uploaded successfully", body = ApiResponse<FileUploadResponse>),
        (status = 400, description = "Bad request - No file provided or invalid file type", body = ApiResponse<FileUploadResponse>),
        (status = 500, description = "Internal server error", body = ApiResponse<FileUploadResponse>)
    ),
    security(("bearer_auth" = [])),
    tag = "files"
)]
pub async fn upload_file(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    mut multipart: Multipart,
) -> (StatusCode, Json<ApiResponse<FileUploadResponse>>) {
    let _pool = &*state.lock().await;

    // Initialize storage service
    let storage = match StorageService::new().await {
        Ok(storage) => storage,
        Err(e) => return ApiResponse::internal_error(format!("Failed to initialize storage: {}", e)),
    };

    let mut file_data = Vec::new();
    let mut filename = String::new();
    let mut content_type = String::new();

    // Parse multipart form data
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let field_name = field.name().unwrap_or("").to_string();

        match field_name.as_str() {
            "file" => {
                filename = field.file_name().unwrap_or("uploaded_file").to_string();
                
                // Determine content type from filename
                content_type = get_content_type_from_filename(&filename).to_string();
                
                file_data = field.bytes().await.unwrap_or_default().to_vec();
            }
            _ => {}
        }
    }

    if file_data.is_empty() {
        return ApiResponse::bad_request("File is required".to_string());
    }

    // Validate file type - only allow PDF, DOCX, and images
    let allowed_types = [
        "application/pdf",
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "application/msword",
        "image/jpeg",
        "image/png",
        "image/gif",
        "image/webp",
        "image/bmp",
        "image/tiff"
    ];

    if !allowed_types.contains(&content_type.as_str()) {
        return ApiResponse::bad_request(format!("File type not allowed. Supported types: PDF, DOCX, DOC, JPG, PNG, GIF, WEBP, BMP, TIFF. Detected type: {}", content_type));
    }

    // Upload file to storage
    let file_key = match storage.upload_file(file_data.clone(), &filename, &content_type).await {
        Ok(key) => key,
        Err(e) => return ApiResponse::internal_error(format!("Failed to upload file: {}", e)),
    };

    // Determine file type category
    let file_type = if content_type == "application/pdf" {
        "pdf".to_string()
    } else if content_type.starts_with("application/vnd.openxmlformats-officedocument.wordprocessingml") || content_type == "application/msword" {
        "document".to_string()
    } else if content_type.starts_with("image/") {
        "image".to_string()
    } else {
        "unknown".to_string()
    };

    // Generate file URL (this could be a direct S3 URL or API endpoint)
    let file_url = format!("/api/files/{}", file_key);

    // Create response
    let upload_response = FileUploadResponse {
        id: file_key.clone(),
        filename: filename.clone(),
        file_type,
        file_size: file_data.len() as i64,
        url: file_url,
        content_type,
        uploaded_at: chrono::Utc::now(),
    };

    ApiResponse::created(upload_response, "File uploaded successfully".to_string())
}

// ===== CREATE TEMPLATE FROM UPLOADED FILE =====

#[utoipa::path(
    post,
    path = "/api/templates/from-file",
    request_body = CreateTemplateFromFileRequest,
    responses(
        (status = 201, description = "Template created from uploaded file", body = ApiResponse<Template>),
        (status = 400, description = "Bad request - Invalid file ID", body = ApiResponse<Template>),
        (status = 404, description = "File not found", body = ApiResponse<Template>),
        (status = 500, description = "Internal server error", body = ApiResponse<Template>)
    ),
    security(("bearer_auth" = [])),
    tag = "templates"
)]
pub async fn create_template_from_file(
    State(state): State<AppState>,
    Extension(user_id): Extension<i64>,
    Json(payload): Json<CreateTemplateFromFileRequest>,
) -> (StatusCode, Json<ApiResponse<Template>>) {
    let pool = &*state.lock().await;

    // Initialize storage service to verify file exists
    let storage = match StorageService::new().await {
        Ok(storage) => storage,
        Err(e) => return ApiResponse::internal_error(format!("Failed to initialize storage: {}", e)),
    };

    // Check if file exists in storage
    let file_exists = match storage.file_exists(&payload.file_id).await {
        Ok(exists) => exists,
        Err(e) => return ApiResponse::internal_error(format!("Failed to check file existence: {}", e)),
    };

    if !file_exists {
        return ApiResponse::not_found("File not found in storage".to_string());
    }

    // Determine content type from file extension
    let content_type = get_content_type_from_filename(&payload.file_id);
    
    // Generate unique slug
    let slug = format!("file-{}-{}", payload.name.to_lowercase().replace(" ", "-"), chrono::Utc::now().timestamp());

    // Create template in database
    let create_template = CreateTemplate {
        name: payload.name.clone(),
        slug: slug.clone(),
        user_id: user_id,
        documents: Some(serde_json::json!([{
            "filename": payload.file_id.split('/').last().unwrap_or(&payload.file_id),
            "content_type": content_type,
            "size": 0, // TODO: Get actual file size
            "url": payload.file_id.clone()
        }])),
    };

    match TemplateQueries::create_template(pool, create_template).await {
        Ok(db_template) => {
            match convert_db_template_to_template_with_fields(db_template, pool).await {
                Ok(template) => ApiResponse::created(template, "Template created from file successfully".to_string()),
                Err(e) => ApiResponse::internal_error(format!("Failed to load template fields: {}", e))
            }
        }
        Err(e) => ApiResponse::internal_error(format!("Failed to create template: {}", e))
    }
}