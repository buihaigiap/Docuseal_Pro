use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Submitter {
    pub id: Option<i64>,
    pub submission_id: Option<i64>,
    pub name: String,
    pub email: String,
    pub status: String, // pending, sent, viewed, signed, completed, declined
    pub signed_at: Option<DateTime<Utc>>,
    pub token: String, // unique token for access
    pub fields_data: Option<serde_json::Value>, // filled form data
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateSubmitterRequest {
    pub status: Option<String>,
    pub fields_data: Option<serde_json::Value>,
}