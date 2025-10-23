use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::models::role::Role;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub role: Role,
    pub is_active: bool,
    pub activation_token: Option<String>,
    pub subscription_status: String,
    pub subscription_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub free_usage_count: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<crate::database::models::DbUser> for User {
    fn from(db_user: crate::database::models::DbUser) -> Self {
        User {
            id: db_user.id,
            name: db_user.name,
            email: db_user.email,
            role: db_user.role,
            is_active: db_user.is_active,
            activation_token: db_user.activation_token,
            subscription_status: db_user.subscription_status,
            subscription_expires_at: db_user.subscription_expires_at,
            free_usage_count: db_user.free_usage_count,
            created_at: db_user.created_at,
        }
    }
}

impl User {
    // User methods removed - using database directly
    
    /// Check if user can submit (has premium subscription or free usage left)
    pub fn can_submit(&self) -> bool {
        match self.subscription_status.as_str() {
            "premium" => {
                // Check if subscription is still valid
                if let Some(expires_at) = self.subscription_expires_at {
                    expires_at > chrono::Utc::now()
                } else {
                    false
                }
            },
            "free" => self.free_usage_count < 10, // Max 10 free submissions
            _ => false
        }
    }
    
    /// Get remaining free submissions
    pub fn remaining_free_submissions(&self) -> i32 {
        if self.subscription_status == "free" {
            (10 - self.free_usage_count).max(0)
        } else {
            0
        }
    }
    
    /// Check if subscription is expired
    pub fn is_subscription_expired(&self) -> bool {
        if let Some(expires_at) = self.subscription_expires_at {
            expires_at <= chrono::Utc::now()
        } else {
            self.subscription_status == "premium" // If premium but no expires_at, consider expired
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserSubscriptionStatus {
    pub user_id: i64,
    pub subscription_status: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub free_usage_count: i32,
    pub remaining_free: i32,
    pub can_submit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CreatePaymentRequest {
    pub success_url: Option<String>,
    pub cancel_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct TeamMember {
    pub id: Option<i64>,
    pub name: String,
    pub email: String,
    pub role: Role,
    pub status: String, // "active" or "pending"
    pub created_at: chrono::DateTime<chrono::Utc>,
}