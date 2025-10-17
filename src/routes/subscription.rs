use axum::{
    extract::{State, Query},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    common::jwt::Claims,
    database::queries::SubscriptionQueries,
    models::user::{UserSubscriptionStatus, CreatePaymentRequest},
    routes::web::AppState,
};

#[derive(Debug, Deserialize)]
pub struct SubscriptionStatusQuery {
    user_id: Option<i64>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct SubscriptionStatusResponse {
    pub user_id: i64,
    pub subscription_status: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub free_usage_count: i32,
    pub remaining_free: i32,
    pub can_submit: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct PaymentLinkResponse {
    pub payment_url: String,
    pub user_id: i64,
}

/// Get payment link for subscription upgrade
#[utoipa::path(
    get,
    path = "/api/subscription/payment-link",
    tag = "subscription",
    responses(
        (status = 200, description = "Payment link retrieved successfully", body = PaymentLinkResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_payment_link(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<PaymentLinkResponse>, (StatusCode, String)> {
    
    // Get payment link từ .env
    let payment_link = std::env::var("PLINK")
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Payment link not configured".to_string()))?;
    
    // Tạo URL với client_reference_id để identify user trong webhook
    let payment_url = format!("https://buy.stripe.com/{}?client_reference_id={}", 
        payment_link, claims.sub);

    Ok(Json(PaymentLinkResponse {
        payment_url,
        user_id: claims.sub,
    }))
}

/// Get subscription status for authenticated user
#[utoipa::path(
    get,
    path = "/api/subscription/status",
    tag = "subscription",
    responses(
        (status = 200, description = "Subscription status retrieved successfully", body = SubscriptionStatusResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_subscription_status(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<SubscriptionStatusResponse>, (StatusCode, String)> {
    let pool = &state.lock().await.db_pool;
    let user = SubscriptionQueries::get_user_subscription_status(pool, claims.sub)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    let remaining_free = if user.subscription_status == "free" {
        (10 - user.free_usage_count).max(0)
    } else {
        0
    };

    let can_submit = match user.subscription_status.as_str() {
        "premium" => {
            if let Some(expires_at) = user.subscription_expires_at {
                expires_at > chrono::Utc::now()
            } else {
                false
            }
        },
        "free" => user.free_usage_count < 10,
        _ => false
    };

    Ok(Json(SubscriptionStatusResponse {
        user_id: user.id,
        subscription_status: user.subscription_status,
        expires_at: user.subscription_expires_at,
        free_usage_count: user.free_usage_count,
        remaining_free,
        can_submit,
    }))
}

/// Increment usage count when user submits document
pub async fn increment_usage_count(
    pool: &sqlx::PgPool,
    user_id: i64,
) -> Result<i32, sqlx::Error> {
    SubscriptionQueries::increment_user_usage(pool, user_id).await
}

/// Increment usage count by specific amount
pub async fn increment_usage_count_by(
    pool: &sqlx::PgPool,
    user_id: i64,
    count: i32,
) -> Result<i32, sqlx::Error> {
    SubscriptionQueries::increment_user_usage_by(pool, user_id, count).await
}

/// Check if user can submit document
pub async fn can_user_submit(
    pool: &sqlx::PgPool,
    user_id: i64,
) -> Result<bool, sqlx::Error> {
    let user = SubscriptionQueries::get_user_subscription_status(pool, user_id)
        .await?
        .ok_or(sqlx::Error::RowNotFound)?;

    let can_submit = match user.subscription_status.as_str() {
        "premium" => {
            if let Some(expires_at) = user.subscription_expires_at {
                expires_at > chrono::Utc::now()
            } else {
                false
            }
        },
        "free" => user.free_usage_count < 10,
        _ => false
    };

    Ok(can_submit)
}