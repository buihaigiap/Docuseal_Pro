use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::models::role::Role;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub role: Role,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::database::models::DbUser> for User {
    fn from(db_user: crate::database::models::DbUser) -> Self {
        User {
            id: db_user.id,
            name: db_user.name,
            email: db_user.email,
            role: db_user.role,
            created_at: db_user.created_at,
        }
    }
}

impl User {
    // User methods removed - using database directly
}