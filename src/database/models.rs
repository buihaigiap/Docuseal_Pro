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

// Database-specific template model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbTemplate {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub user_id: i64,
    pub fields: Option<serde_json::Value>, // JSONB field
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
    pub fields: Option<serde_json::Value>,
    pub submitters: Option<serde_json::Value>,
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
    pub submitters: Option<serde_json::Value>,
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
    pub submitters: Option<serde_json::Value>,
    pub expires_at: Option<DateTime<Utc>>,
}

// Database-specific submitter model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
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
}

// Create submitter request
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
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbSignaturePosition {
    pub id: i64,
    pub submitter_id: i64,
    pub field_name: String,
    pub page: i32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub created_at: DateTime<Utc>,
}

// Database-specific signature data model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbSignatureData {
    pub id: i64,
    pub submitter_id: i64,
    pub signature_image: String,
    pub signed_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}