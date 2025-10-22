use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, sqlx::Type, PartialEq)]
#[sqlx(type_name = "user_role")]
#[serde(rename_all = "snake_case")]
pub enum Role {
    #[sqlx(rename = "admin")]
    Admin,
    #[sqlx(rename = "editor")]
    Editor,
    #[sqlx(rename = "member")]
    Member,
    #[sqlx(rename = "agent")]
    Agent,
    #[sqlx(rename = "viewer")]
    Viewer,
}

impl Default for Role {
    fn default() -> Self {
        Role::Member
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Role::Admin => "admin",
            Role::Editor => "editor", 
            Role::Member => "member",
            Role::Agent => "agent",
            Role::Viewer => "viewer",
        };
        write!(f, "{}", s)
    }
}

impl Role {
    pub fn from_string(s: &str) -> Self {
        match s {
            "Admin" => Role::Admin,
            "Editor" => Role::Editor,
            "Member" => Role::Member,
            "Agent" => Role::Agent,
            "Viewer" => Role::Viewer,
            _ => Role::Member, // Default fallback to Member
        }
    }

    pub fn to_lowercase(&self) -> String {
        match self {
            Role::Admin => "admin".to_string(),
            Role::Editor => "editor".to_string(),
            Role::Member => "member".to_string(),
            Role::Agent => "agent".to_string(),
            Role::Viewer => "viewer".to_string(),
        }
    }

    pub fn is_admin(&self) -> bool {
        matches!(self, Role::Admin)
    }
}