use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use stripe::{
    CheckoutSession, CheckoutSessionMode, CreateCheckoutSession, CreateCheckoutSessionLineItems,
    CreateCheckoutSessionPaymentMethodTypes, Currency,
};

use crate::{
    common::authorization::AuthenticatedUser,
    database::{
        connection::AppState,
        queries::SubscriptionQueries,
        models::CreatePaymentRecord,
    },
    models::user::{SubscriptionPlan, UserSubscriptionStatus, CreatePaymentRequest},
};

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

/// Get current user's subscription status
pub async fn get_user_subscription_status(
    State(state): State<>AppState,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<UserSubscriptionStatus>, (StatusCode, Json<Value>)> {
    match SubscriptionQueries::get_user_subscription_status(&state.pool, user.id).await {
        Ok(Some(db_user)) => {
            let user_model = crate::models::user::User::from(db_user);
            
            let status = UserSubscriptionStatus {
                user_id: user_model.id,
                subscription_status: user_model.subscription_status.clone(),
                subscription_type: user_model.subscription_type.clone(),
                expires_at: user_model.subscription_expires_at,
                free_usage_count: user_model.free_usage_count,
                max_free_usage: user_model.max_free_usage,
                remaining_free: user_model.remaining_free_submissions(),
                can_submit: user_model.can_submit(),
            };

            Ok(Json(status))
        }
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"error": "User not found"})),
        )),
        Err(e) => {
            eprintln!("Error fetching user subscription status: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch subscription status"})),
            ))
        }
    }
}

/// Create Stripe checkout session for subscription payment
pub async fn create_payment_session(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<CreatePaymentRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Get subscription plan
    let plan = match SubscriptionQueries::get_subscription_plan(&state.pool, request.plan_id).await {
        Ok(Some(plan)) => plan,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Subscription plan not found"})),
            ));
        }
        Err(e) => {
            eprintln!("Error fetching subscription plan: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch subscription plan"})),
            ));
        }
    };

    // Create pending payment record
    let payment_data = CreatePaymentRecord {
        user_id: user.id,
        stripe_session_id: None, // Will be updated after session creation
        amount_cents: plan.price_cents,
        currency: "USD".to_string(),
        status: "pending".to_string(),
        metadata: Some(json!({
            "plan_name": plan.name,
            "plan_id": plan.id,
            "user_email": user.email
        })),
    };

    let _payment_record = match SubscriptionQueries::create_payment_record(&state.pool, payment_data).await {
        Ok(record) => record,
        Err(e) => {
            eprintln!("Error creating payment record: {:?}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create payment record"})),
            ));
        }
    };

    // Create Stripe checkout session
    let client = stripe::Client::new(std::env::var("STRIPE_SECRET_KEY").unwrap_or_default());

    let success_url = request.success_url.unwrap_or_else(|| {
        format!("{}/subscription/success", std::env::var("FRONTEND_URL").unwrap_or_default())
    });

    let cancel_url = request.cancel_url.unwrap_or_else(|| {
        format!("{}/subscription/cancel", std::env::var("FRONTEND_URL").unwrap_or_default())
    });

    // Client reference ID format: "user_{user_id}_plan_{plan_id}"
    let client_reference_id = format!("user_{}_plan_{}", user.id, plan.id);

    let checkout_session = CreateCheckoutSession {
        mode: Some(CheckoutSessionMode::Payment),
        success_url: Some(&success_url),
        cancel_url: Some(&cancel_url),
        client_reference_id: Some(&client_reference_id),
        customer_email: Some(&user.email),
        line_items: Some(vec![CreateCheckoutSessionLineItems {
            price_data: Some(stripe::CreateCheckoutSessionLineItemsPriceData {
                currency: Currency::USD,
                product_data: stripe::CreateCheckoutSessionLineItemsPriceDataProductData {
                    name: plan.name.clone(),
                    description: Some(format!(
                        "{} subscription - {} month(s)",
                        plan.name, plan.duration_months
                    )),
                    ..Default::default()
                },
                unit_amount: Some(plan.price_cents as i64),
                ..Default::default()
            }),
            quantity: Some(1),
            ..Default::default()
        }]),
        payment_method_types: Some(vec![CreateCheckoutSessionPaymentMethodTypes::Card]),
        metadata: Some(
            [
                ("user_id".to_string(), user.id.to_string()),
                ("plan_id".to_string(), plan.id.to_string()),
                ("plan_name".to_string(), plan.name),
            ]
            .iter()
            .cloned()
            .collect(),
        ),
        ..Default::default()
    };

    match CheckoutSession::create(&client, checkout_session).await {
        Ok(session) => {
            // Update payment record with session ID
            if let Some(session_id) = &session.id {
                let _ = sqlx::query(
                    "UPDATE payment_records SET stripe_session_id = $1 WHERE user_id = $2 AND id = $3"
                )
                .bind(session_id.as_str())
                .bind(user.id)
                .bind(_payment_record.id)
                .execute(&state.pool)
                .await;
            }

            Ok(Json(json!({
                "checkout_url": session.url,
                "session_id": session.id
            })))
        }
        Err(e) => {
            eprintln!("Error creating Stripe checkout session: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to create payment session"})),
            ))
        }
    }
}

/// Get user's payment history
pub async fn get_payment_history(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(10);
    let offset = (page - 1) * limit;

    match sqlx::query_as!(
        crate::database::models::DbPaymentRecord,
        r#"
        SELECT * FROM payment_records 
        WHERE user_id = $1 
        ORDER BY created_at DESC 
        LIMIT $2 OFFSET $3
        "#,
        user.id,
        limit as i64,
        offset as i64
    )
    .fetch_all(&state.pool)
    .await
    {
        Ok(payments) => {
            Ok(Json(json!({
                "payments": payments,
                "page": page,
                "limit": limit,
                "total": payments.len()
            })))
        }
        Err(e) => {
            eprintln!("Error fetching payment history: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to fetch payment history"})),
            ))
        }
    }
}

/// Cancel current subscription (set to not auto-renew)
pub async fn cancel_subscription(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Set auto_renew to false for current subscription
    match sqlx::query(
        "UPDATE user_subscriptions SET auto_renew = false WHERE user_id = $1 AND status = 'active'"
    )
    .bind(user.id)
    .execute(&state.pool)
    .await
    {
        Ok(_) => Ok(Json(json!({
            "message": "Subscription cancellation scheduled. Your subscription will remain active until the end of the current billing period."
        }))),
        Err(e) => {
            eprintln!("Error cancelling subscription: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to cancel subscription"})),
            ))
        }
    }
}