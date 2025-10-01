use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Template {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub user_id: i64,
    pub fields: Option<Vec<Field>>,
    pub submitters: Option<Vec<Submitter>>,
    pub documents: Option<Vec<Document>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Field {
    pub name: String,
    pub field_type: String, // text, signature, date, etc.
    pub required: bool,
    pub position: Option<FieldPosition>,
    pub options: Option<Vec<String>>, // for select/radio fields
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct FieldPosition {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub page: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Submitter {
    pub name: String,
    pub email: String,
    pub role: Option<String>,
    pub order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Document {
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub url: String,
}

// Request/Response structs for API
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateTemplateRequest {
    pub name: String,
    pub fields: Option<Vec<Field>>,
    pub submitters: Option<Vec<Submitter>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub fields: Option<Vec<Field>>,
    pub submitters: Option<Vec<Submitter>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CloneTemplateRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateTemplateFromHtmlRequest {
    pub name: String,
    pub html: String,
    pub submitters: Option<Vec<Submitter>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateTemplateFromPdfRequest {
    pub name: String,
    pub pdf_data: String, // base64 encoded
    pub submitters: Option<Vec<Submitter>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateTemplateFromDocxRequest {
    pub name: String,
    pub docx_data: String, // base64 encoded
    pub submitters: Option<Vec<Submitter>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct MergeTemplatesRequest {
    pub template_ids: Vec<i64>,
    pub name: String,
}