use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::database::models::DbUser> for User {
    fn from(db_user: crate::database::models::DbUser) -> Self {
        User {
            id: db_user.id,
            name: db_user.name,
            email: db_user.email,
            created_at: db_user.created_at,
        }
    }
}

impl User {
    // User methods removed - using database directly
}