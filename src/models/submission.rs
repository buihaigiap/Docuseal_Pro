use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

use super::submitter::Submitter;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Submission {
    pub id: i64,
    pub template_id: i64,
    pub user_id: i64,
    pub status: String, // pending, completed, expired
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documents: Option<Vec<Document>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitters: Option<Vec<Submitter>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Document {
    pub id: i64,
    pub submission_id: i64,
    pub filename: String,
    pub content_type: String,
    pub file_url: String,
    pub created_at: DateTime<Utc>,
}

use crate::models::submitter::CreateSubmitterRequest;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateSubmissionRequest {
    pub template_id: i64,
    pub name: Option<String>,
    pub submitters: Vec<CreateSubmitterRequest>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateSubmissionRequest {
    pub status: Option<String>,
    pub submitters: Option<Vec<Submitter>>,
}