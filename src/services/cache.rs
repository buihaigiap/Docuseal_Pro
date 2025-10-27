use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::{DateTime, Utc, Duration};

#[derive(Clone)]
pub struct OtpCache {
    cache: Arc<Mutex<HashMap<String, (String, DateTime<Utc>)>>>, // email -> (otp_code, expires_at)
}

impl OtpCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn store_otp(&self, email: &str, otp: &str, ttl_seconds: i64) -> Result<(), String> {
        let mut cache = self.cache.lock().await;
        let expires_at = Utc::now() + Duration::seconds(ttl_seconds);
        cache.insert(email.to_string(), (otp.to_string(), expires_at));
        Ok(())
    }

    pub async fn verify_otp(&self, email: &str, otp: &str) -> Result<bool, String> {
        let mut cache = self.cache.lock().await;

        if let Some((stored_otp, expires_at)) = cache.get(email) {
            // Check if expired
            if Utc::now() > *expires_at {
                cache.remove(email); // Clean up expired entry
                return Ok(false);
            }

            // Check if OTP matches
            if stored_otp == otp {
                cache.remove(email); // Remove after successful verification (one-time use)
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub async fn cleanup_expired(&self) {
        let mut cache = self.cache.lock().await;
        let now = Utc::now();
        cache.retain(|_, (_, expires_at)| *expires_at > now);
    }
}