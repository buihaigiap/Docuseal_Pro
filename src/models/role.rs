use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::Type, PartialEq)]
#[sqlx(type_name = "user_role", rename_all = "snake_case")]
pub enum Role {
    Admin,
    TeamMember,
    Recipient,
}

impl Default for Role {
    fn default() -> Self {
        Role::TeamMember
    }
}