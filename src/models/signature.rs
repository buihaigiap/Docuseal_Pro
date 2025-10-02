use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SignaturePosition {
    pub id: Option<i64>,
    pub submitter_id: i64,
    pub field_name: String,
    pub page: i32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSignaturePosition {
    pub submitter_id: i64,
    pub field_name: String,
    pub page: i32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PublicCreateSignaturePosition {
    pub field_name: String,
    pub page: i32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SignatureData {
    pub id: Option<i64>,
    pub submitter_id: i64,
    pub signature_image: String, // Base64 encoded image
    pub signed_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSignatureData {
    pub submitter_id: i64,
    pub signature_image: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PublicCreateSignatureData {
    pub signature_image: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SignatureRequest {
    pub positions: Vec<SignaturePositionData>,
    pub signature_data: String, // Base64 encoded signature image
    pub fields_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SignaturePositionData {
    pub field_name: String,
    pub page: i32,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}