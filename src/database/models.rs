use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use crate::models::role::Role;

// Database-specific user model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Create user request - chỉ cần data fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUser {
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub role: Role,
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
    pub deleted_at: Option<DateTime<Utc>>,
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

// Database submitter model
#[derive(Debug, Clone)]
pub struct DbSubmitter {
    pub id: i64,
    pub template_id: i64, // Changed from submission_id
    pub user_id: i64,     // New field
    pub name: String,
    pub email: String,
    pub status: String,
    pub signed_at: Option<DateTime<Utc>>,
    pub token: String,
    pub bulk_signatures: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}// Create submitter request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubmitter {
    pub template_id: i64, // Changed from submission_id
    pub user_id: i64,     // New field
    pub name: String,
    pub email: String,
    pub status: String,
    pub token: String,
}

// Database-specific signature data model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DbSignatureData {
    pub id: i64,
    pub submitter_id: i64,
    pub signature_value: Option<String>,
    pub signed_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

