use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct Account {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::database::models::DbAccount> for Account {
    fn from(db_account: crate::database::models::DbAccount) -> Self {
        Account {
            id: db_account.id,
            name: db_account.name,
            slug: db_account.slug,
            created_at: db_account.created_at,
            updated_at: db_account.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreateAccountRequest {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UpdateAccountRequest {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccountWithUsers {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub users: Vec<crate::models::user::User>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct AccountLinkedAccount {
    pub id: i64,
    pub account_id: i64,
    pub linked_account_id: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::database::models::DbAccountLinkedAccount> for AccountLinkedAccount {
    fn from(db: crate::database::models::DbAccountLinkedAccount) -> Self {
        AccountLinkedAccount {
            id: db.id,
            account_id: db.account_id,
            linked_account_id: db.linked_account_id,
            created_at: db.created_at,
        }
    }
}
