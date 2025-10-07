use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Serialize};
use std::env;
use chrono::{Utc, Duration};

#[derive(Serialize)]
struct Claims {
    sub: i64,
    email: String,
    role: String,
    exp: usize,
}

fn main() {
    let secret = env::var("JWT_SECRET").unwrap_or_else(|_| "your-super-secret-jwt-key-change-this-in-production".to_string());
    let sub: i64 = env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(1);
    let email = env::args().nth(2).unwrap_or_else(|| "admin@example.com".to_string());
    let role = env::args().nth(3).unwrap_or_else(|| "admin".to_string());

    let exp = (Utc::now() + Duration::days(365)).timestamp() as usize;

    let claims = Claims { sub, email, role, exp };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .expect("Failed to encode JWT");

    println!("{}", token);
}
