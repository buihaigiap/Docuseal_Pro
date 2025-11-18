use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct EmailTemplate {
    pub id: i64,
    pub user_id: i64,
    pub template_type: String, // 'invitation', 'reminder', 'completion'
    pub subject: String,
    pub body: String,
    pub body_format: String, // 'text' or 'html'
    pub is_default: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::database::models::DbEmailTemplate> for EmailTemplate {
    fn from(db_template: crate::database::models::DbEmailTemplate) -> Self {
        EmailTemplate {
            id: db_template.id,
            user_id: db_template.user_id,
            template_type: db_template.template_type,
            subject: db_template.subject,
            body: db_template.body,
            body_format: db_template.body_format,
            is_default: db_template.is_default,
            created_at: db_template.created_at,
            updated_at: db_template.updated_at,
        }
    }
}