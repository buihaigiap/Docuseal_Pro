use axum::{extract::State, http::{StatusCode, HeaderMap}, response::IntoResponse};
use bytes::Bytes;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use serde_json::Value;

use crate::{
    database::{queries::SubscriptionQueries, models::CreatePaymentRecord},
    routes::web::AppState,
};

fn verify_stripe_signature(body: &Bytes, sig: &str, secret: &str) -> Result<Value, String> {
    let parts: Vec<&str> = sig.split(',').collect();
    let mut timestamp = None;
    let mut expected_sig = None;

    for part in parts {
        if let Some(t) = part.strip_prefix("t=") {
            timestamp = Some(t.parse::<i64>().map_err(|_| "Invalid timestamp")?);
        } else if let Some(v1) = part.strip_prefix("v1=") {
            expected_sig = Some(v1);
        }
    }

    let timestamp = timestamp.ok_or("Missing timestamp")?;
    let expected_sig = expected_sig.ok_or("Missing v1 signature")?;

    // Check if timestamp is within 5 minutes
    let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    if (now - timestamp).abs() > 300 {
        return Err("Timestamp too old".to_string());
    }

    // Compute HMAC
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).map_err(|_| "Invalid secret")?;
    mac.update(format!("{}.", timestamp).as_bytes());
    mac.update(body);
    let result = mac.finalize();
    let computed_sig = hex::encode(result.into_bytes());

    if computed_sig == expected_sig {
        serde_json::from_slice(body).map_err(|e| format!("Invalid JSON: {}", e))
    } else {
        Err("Signature mismatch".to_string())
    }
}

#[axum::debug_handler]
pub async fn stripe_webhook_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    println!("🔥 Stripe webhook received");

    let sig = headers
        .get("Stripe-Signature")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let endpoint_secret =
        std::env::var("STRIPE_WEBHOOK_SECRET").expect("Missing STRIPE_WEBHOOK_SECRET");

    // ✅ Xác minh chữ ký
    let event: Value = match verify_stripe_signature(&body, sig, &endpoint_secret) {
        Ok(event) => event,
        Err(err) => {
            eprintln!("❌ Signature verification failed: {}", err);
            return (StatusCode::BAD_REQUEST, "Invalid signature").into_response();
        }
    };

    println!("✅ Verified event: {}", event["type"].as_str().unwrap_or("unknown"));

    if let Some(event_type) = event["type"].as_str() {
        match event_type {
            "checkout.session.completed" => {
                if let Some(session) = event["data"]["object"].as_object() {
                    // Email thực
                    let email = session.get("customer_details")
                                       .and_then(|v| v.get("email"))
                                       .and_then(|v| v.as_str())
                                       .unwrap_or("");

                    // Client reference ID fallback metadata
                    let client_ref = session.get("client_reference_id")
                                            .and_then(|v| v.as_str())
                                            .or_else(|| session.get("metadata")
                                                               .and_then(|m| m.get("client_reference_id"))
                                                               .and_then(|v| v.as_str()))
                                            .unwrap_or("");

                    // Amount USD
                    let amount = session.get("amount_total")
                                        .and_then(|v| v.as_i64())
                                        .unwrap_or(0) as f64 / 100.0;

                    // Payment Link
                    let payment_link = session.get("payment_link")
                                              .and_then(|v| v.as_str())
                                              .unwrap_or("");

                    println!("💰 Payment success for {email}, amount: {amount} USD, client_ref: {client_ref}, link: {payment_link}");

                    // Xử lý DB
                    if let Ok(user_id) = client_ref.parse::<i64>() {
                        let data = CreatePaymentRecord {
                            user_id,
                            stripe_session_id: session.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            stripe_payment_intent_id: session.get("payment_intent").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            amount_cents: (amount * 100.0) as i32,
                            currency: "USD".to_string(),
                            status: "completed".to_string(),
                            stripe_price_id: None,
                            metadata: Some(serde_json::json!({
                                "email": email,
                                "client_reference_id": client_ref,
                                "payment_link": payment_link
                            })),
                        };
                        if let Ok(record) = SubscriptionQueries::create_payment_record(&*state.lock().await, data).await {
                            println!("💾 Created payment record {}", record.id);
                        }
                    }

                    // Xử lý DB
                    if let Ok(user_id) = client_ref.parse::<i64>() {
                        let data = CreatePaymentRecord {
                            user_id,
                            stripe_session_id: session.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            stripe_payment_intent_id: session.get("payment_intent").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            amount_cents: (amount * 100.0) as i32,
                            currency: "USD".to_string(),
                            status: "completed".to_string(),
                            stripe_price_id: None,
                            metadata: Some(serde_json::json!({
                                "email": email,
                                "client_reference_id": client_ref,
                                "payment_link": payment_link
                            })),
                        };
                        if let Ok(record) = SubscriptionQueries::create_payment_record(&*state.lock().await, data).await {
                            println!("💾 Created payment record {}", record.id);
                        }
                    }
                }
            },
            "customer.subscription.created" => {
                println!("✅ Received event: customer.subscription.created");
            },
            _ => {
                println!("⚠️ Unhandled event type: {}", event_type);
            }
        }
    }

    (StatusCode::OK, "ok").into_response()
}
