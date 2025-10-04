use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// Database-specific user model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Create user request - chỉ cần data fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
    pub password_hash: String,
}

// Database-specific template field model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbTemplateField {
    pub id: i64,
    pub template_id: i64,
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub display_order: i32,
    pub position: Option<serde_json::Value>,
    pub options: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Create template field request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplateField {
    pub template_id: i64,
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub display_order: i32,
    pub position: Option<serde_json::Value>,
    pub options: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

// Database-specific template model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbTemplate {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub user_id: i64,
    pub submitters: Option<serde_json::Value>, // JSONB field
    pub documents: Option<serde_json::Value>, // JSONB field
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Create template request - chỉ cần data fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplate {
    pub name: String,
    pub slug: String,
    pub user_id: i64,
    pub documents: Option<serde_json::Value>,
}

// Database-specific submission model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbSubmission {
    pub id: i64,
    pub template_id: i64,
    pub user_id: i64,
    pub status: String,
    pub documents: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

// Create submission request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubmission {
    pub template_id: i64,
    pub user_id: i64,
    pub status: String,
    pub documents: Option<serde_json::Value>,
    pub expires_at: Option<DateTime<Utc>>,
}

// Database submitter model
#[derive(Debug, Clone)]
pub struct DbSubmitter {
    pub id: i64,
    pub submission_id: i64,
    pub name: String,
    pub email: String,
    pub status: String,
    pub signed_at: Option<DateTime<Utc>>,
    pub token: String,
    pub fields_data: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}// Create submitter request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubmitter {
    pub submission_id: i64,
    pub name: String,
    pub email: String,
    pub status: String,
    pub token: String,
    pub fields_data: Option<serde_json::Value>,
}

// Database-specific signature position model
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct DbSignaturePosition {
    pub id: i64,
    pub submitter_id: i64,
    pub bulk_signatures: Option<serde_json::Value>, // JSONB array chứa nhiều signatures
    pub signed_at: Option<DateTime<Utc>>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub version: Option<i32>,
    pub is_active: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub signature_image: Option<String>,
}// Database-specific signature data model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbSignatureData {
    pub id: i64,
    pub submitter_id: i64,
    pub signature_value: Option<String>,
    pub signed_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

// Database-specific bulk signature model - DEPRECATED: Now using bulk_signatures column in signature_positions table
// #[derive(Debug, Serialize, Deserialize, FromRow)]
// pub struct DbBulkSignature {
//     pub id: i64,
//     pub submitter_id: i64,
//     pub signatures: serde_json::Value, // JSONB array chứa [{field_id, field_name, signature_value}, ...]
//     pub signed_at: DateTime<Utc>,
//     pub ip_address: Option<String>,
//     pub user_agent: Option<String>,
//     pub created_at: DateTime<Utc>,
//     pub updated_at: Option<DateTime<Utc>>,
// }