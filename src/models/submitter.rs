use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Submitter {
    pub id: Option<i64>,
    pub template_id: Option<i64>,
    pub user_id: Option<i64>,
    pub name: String,
    pub email: String,
    pub status: String, // pending, sent, viewed, signed, completed, declined
    pub signed_at: Option<DateTime<Utc>>,
    pub token: String, // unique token for access
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bulk_signatures: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateSubmitterRequest {
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PublicUpdateSubmitterRequest {
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSubmitterRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PublicSubmissionResponse {
    pub template: crate::models::template::Template,
    pub submitter: Submitter,
}