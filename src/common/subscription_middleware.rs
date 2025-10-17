use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{Json, Response},
};
use serde_json::json;

use crate::{
    database::{connection::AppState, queries::SubscriptionQueries},
    common::authorization::get_user_from_token,
};

/// Middleware to check if user can submit documents (has premium or free submissions left)
pub async fn check_submission_limit(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // Extract user from token
    let user = match get_user_from_token(&req, &state.pool).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "User not found"})),
            ));
        }
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid token"})),
            ));
        }
    };

    // Check if user can submit
    if !user.can_submit() {
        return Err((
            StatusCode::PAYMENT_REQUIRED,
            Json(json!({
                "error": "Submission limit exceeded",
                "message": "You have reached your free submission limit. Please upgrade to premium to continue.",
                "subscription_status": user.subscription_status,
                "free_usage_count": user.free_usage_count,
                "max_free_usage": user.max_free_usage,
                "upgrade_required": true
            })),
        ));
    }

    // Add user to request extensions for use in handlers
    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}

/// Middleware to track usage after successful submission
pub async fn track_submission_usage(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let response = next.run(req).await;

    // Only increment usage if the request was successful
    if response.status().is_success() {
        if let Some(user) = req.extensions().get::<crate::models::user::User>() {
            // Only increment for free users
            if user.subscription_status == "free" {
                if let Err(e) = SubscriptionQueries::increment_user_usage(&state.pool, user.id).await {
                    eprintln!("Failed to increment user usage: {:?}", e);
                }
            }
        }
    }

    Ok(response)
}

/// Check if user has an active premium subscription
pub async fn check_premium_subscription(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // Extract user from token
    let user = match get_user_from_token(&req, &state.pool).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "User not found"})),
            ));
        }
        Err(_) => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({"error": "Invalid token"})),
            ));
        }
    };

    // Check if user has premium subscription
    if user.subscription_status != "premium" || user.is_subscription_expired() {
        return Err((
            StatusCode::PAYMENT_REQUIRED,
            Json(json!({
                "error": "Premium subscription required",
                "message": "This feature requires a premium subscription.",
                "subscription_status": user.subscription_status,
                "upgrade_required": true
            })),
        ));
    }

    Ok(next.run(req).await)
}