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
use axum_extra::extract::Multipart;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde_json;

use crate::common::responses::ApiResponse;
use crate::models::template::{
    Template, UpdateTemplateRequest, CloneTemplateRequest,
    CreateTemplateFromHtmlRequest, MergeTemplatesRequest
};
use crate::database::connection::DbPool;
use crate::database::models::{CreateTemplate};
use crate::database::queries::TemplateQueries;
use crate::services::storage::StorageService;
use crate::common::jwt::auth_middleware;

pub type AppState = Arc<Mutex<DbPool>>;

pub fn create_template_router() -> Router<AppState> {
    Router::new()
        .route("/templates", get(get_templates))
        .route("/templates/:id", get(get_template))
        .route("/templates/:id", put(update_template))
        .route("/templates/:id", delete(delete_template))
        .route("/templates/:id/clone", post(clone_template))
        .route("/templates/html", post(create_template_from_html))
        .route("/templates/pdf", post(create_template_from_pdf))
        .route("/templates/docx", post(create_template_from_docx))
        .route("/templates/merge", post(merge_templates))
        .route("/files/:key", get(download_file))
        .route("/files/:key/preview", get(preview_file))
        .layer(middleware::from_fn(auth_middleware))
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
            let templates: Vec<Template> = db_templates.into_iter()
                .map(|db_template| convert_db_template_to_template(db_template))
                .collect();
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
            let template = convert_db_template_to_template(db_template);
            ApiResponse::success(template, "Template retrieved successfully".to_string())
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

    // Convert fields and submitters to JSON values
    let fields_json = payload.fields.as_ref().map(|f| serde_json::to_value(f).unwrap_or(serde_json::Value::Null));
    let submitters_json = payload.submitters.as_ref().map(|s| serde_json::to_value(s).unwrap_or(serde_json::Value::Null));

    match TemplateQueries::update_template(pool, id, user_id, payload.name.as_deref(), fields_json.as_ref(), submitters_json.as_ref()).await {
        Ok(Some(db_template)) => {
            let template = convert_db_template_to_template(db_template);
            ApiResponse::success(template, "Template updated successfully".to_string())
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
            let template = convert_db_template_to_template(db_template);
            ApiResponse::created(template, "Template cloned successfully".to_string())
        }
        Ok(None) => ApiResponse::not_found("Original template not found".to_string()),
        Err(e) => ApiResponse::internal_error(format!("Failed to clone template: {}", e)),
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
        fields: None, // TODO: Extract fields from HTML
        submitters: payload.submitters.map(|s| serde_json::to_value(s).unwrap_or(serde_json::Value::Null)),
        documents: Some(serde_json::json!([{
            "filename": filename,
            "content_type": "text/html",
            "size": payload.html.len(),
            "url": file_key
        }])),
    };

    match TemplateQueries::create_template(pool, create_template).await {
        Ok(db_template) => {
            let template = convert_db_template_to_template(db_template);
            ApiResponse::created(template, "Template created from HTML successfully".to_string())
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
        fields: None, // TODO: Extract fields from PDF
        submitters: None,
        documents: Some(serde_json::json!([{
            "filename": filename,
            "content_type": "application/pdf",
            "size": 0, // TODO: Get actual file size
            "url": file_key
        }])),
    };

    match TemplateQueries::create_template(pool, create_template).await {
        Ok(db_template) => {
            let template = convert_db_template_to_template(db_template);
            ApiResponse::created(template, "Template created from PDF successfully".to_string())
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
        fields: None, // TODO: Extract fields from DOCX
        submitters: None,
        documents: Some(serde_json::json!([{
            "filename": filename,
            "content_type": "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "size": 0, // TODO: Get actual file size
            "url": file_key
        }])),
    };

    match TemplateQueries::create_template(pool, create_template).await {
        Ok(db_template) => {
            let template = convert_db_template_to_template(db_template);
            ApiResponse::created(template, "Template created from DOCX successfully".to_string())
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
        fields: None, // TODO: Extract fields from HTML
        submitters: payload.submitters.map(|s| serde_json::to_value(s).unwrap_or(serde_json::Value::Null)),
        documents: Some(serde_json::json!([{
            "filename": format!("{}.html", payload.name.to_lowercase().replace(" ", "_")),
            "content_type": "text/html",
            "size": payload.html.len(),
            "url": "local-storage" // Placeholder URL
        }])),
    };

    match TemplateQueries::create_template(pool, create_template).await {
        Ok(db_template) => {
            let template = convert_db_template_to_template(db_template);
            ApiResponse::created(template, "Template created from HTML successfully (without storage)".to_string())
        }
        Err(e) => {
            ApiResponse::internal_error(format!("Failed to create template: {}", e))
        }
    }
}

// Helper function to convert database template to API template
fn convert_db_template_to_template(db_template: crate::database::models::DbTemplate) -> Template {
    Template {
        id: db_template.id,
        name: db_template.name,
        slug: db_template.slug,
        user_id: db_template.user_id,
        fields: db_template.fields.and_then(|v| serde_json::from_value(v).ok()),
        submitters: db_template.submitters.and_then(|v| serde_json::from_value(v).ok()),
        documents: db_template.documents.and_then(|v| serde_json::from_value(v).ok()),
        created_at: db_template.created_at,
        updated_at: db_template.updated_at,
    }
}

#[utoipa::path(
    get,
    path = "/api/files/{key}",
    params(
        ("key" = String, Path, description = "File key in storage")
    ),
    responses(
        (status = 200, description = "File downloaded successfully"),
        (status = 404, description = "File not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    tag = "files"
)]
pub async fn download_file(
    Path(key): Path<String>,
    Extension(_user_id): Extension<i64>,
) -> Result<Response<Body>, (StatusCode, Json<ApiResponse<()>>)> {
    // Initialize storage service
    let storage = match StorageService::new().await {
        Ok(storage) => storage,
        Err(e) => {
            let (status, json_resp) = ApiResponse::<()>::internal_error(format!("Failed to initialize storage: {}", e));
            return Err((status, json_resp));
        }
    };

    // Download file from storage
    let file_data = match storage.download_file(&key).await {
        Ok(data) => data,
        Err(e) => {
            let (status, json_resp) = ApiResponse::<()>::not_found(format!("File not found: {}", e));
            return Err((status, json_resp));
        }
    };

    // Determine content type based on file extension
    let content_type = if key.ends_with(".html") {
        "text/html"
    } else if key.ends_with(".pdf") {
        "application/pdf"
    } else if key.ends_with(".docx") {
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    } else {
        "application/octet-stream"
    };

    // Create response with file data
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_DISPOSITION, format!("inline; filename=\"{}\"", key))
        .body(Body::from(file_data))
        .unwrap();

    Ok(response)
}

#[utoipa::path(
    get,
    path = "/api/files/{key}/preview",
    params(
        ("key" = String, Path, description = "File key in storage")
    ),
    responses(
        (status = 200, description = "File content previewed successfully"),
        (status = 404, description = "File not found"),
        (status = 500, description = "Internal server error")
    ),
    security(("bearer_auth" = [])),
    tag = "files"
)]
pub async fn preview_file(
    Path(key): Path<String>,
    Extension(_user_id): Extension<i64>,
) -> Result<String, (StatusCode, Json<ApiResponse<()>>)> {
    // Initialize storage service
    let storage = match StorageService::new().await {
        Ok(storage) => storage,
        Err(e) => {
            let (status, json_resp) = ApiResponse::<()>::internal_error(format!("Failed to initialize storage: {}", e));
            return Err((status, json_resp));
        }
    };

    // Download file from storage
    let file_data = match storage.download_file(&key).await {
        Ok(data) => data,
        Err(e) => {
            let (status, json_resp) = ApiResponse::<()>::not_found(format!("File not found: {}", e));
            return Err((status, json_resp));
        }
    };

    // Convert to string for preview (only for text-based files)
    match String::from_utf8(file_data.clone()) {
        Ok(content) => {
            // Check if it's actually text content (not binary)
            if content.contains('\0') || content.chars().any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t') {
                let (status, json_resp) = ApiResponse::<()>::bad_request("Cannot preview binary file. Use download endpoint instead.".to_string());
                return Err((status, json_resp));
            }
            Ok(content)
        },
        Err(_) => {
            let (status, json_resp) = ApiResponse::<()>::bad_request("Cannot preview binary file. Use download endpoint instead.".to_string());
            Err((status, json_resp))
        }
    }
}