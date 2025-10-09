use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey, errors::Error};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use crate::models::role::Role;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: i64, // user id
    pub email: String,
    pub role: String, // Changed from Role to String for JWT compatibility
    pub exp: usize, // expiration time
}

pub fn generate_jwt(user_id: i64, email: &str, role: &Role, secret: &str) -> Result<String, Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let role_str = match role {
        Role::Admin => "Admin",
        Role::TeamMember => "TeamMember", 
        Role::Recipient => "Recipient",
    };

    let claims = Claims {
        sub: user_id,
        email: email.to_owned(),
        role: role_str.to_string(),
        exp: expiration,
    };

    let header = Header::new(Algorithm::HS256);
    let encoding_key = EncodingKey::from_secret(secret.as_ref());

    encode(&header, &claims, &encoding_key)
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, Error> {
    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::new(Algorithm::HS256);

    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
    Ok(token_data.claims)
}

use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

pub async fn auth_middleware(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| header.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => {
            println!("No authorization header");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "your-super-secret-jwt-key-change-this-in-production".to_string());
    println!("Using secret: {}", secret);
    println!("Token: {}", token);
    let claims = match verify_jwt(token, &secret) {
        Ok(claims) => {
            println!("JWT verified successfully: {:?}", claims);
            claims
        },
        Err(e) => {
            println!("JWT verification failed: {:?}", e);
            return Err(StatusCode::UNAUTHORIZED);
        }
    };

    // Convert role string to Role enum
    let role = match claims.role.as_str() {
        "Admin" => Role::Admin,
        "TeamMember" => Role::TeamMember,
        "Recipient" => Role::Recipient,
        _ => return Err(StatusCode::UNAUTHORIZED),
    };

    // Add user_id and role to request extensions
    request.extensions_mut().insert(claims.sub);
    request.extensions_mut().insert(role);

    Ok(next.run(request).await)
}