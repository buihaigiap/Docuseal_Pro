use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;

type HmacSha256 = Hmac<Sha256>;

/// Generate HMAC-SHA256 hash of token using secret
pub fn hash_token(secret: &str, token: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(token.as_bytes());
    let result = mac.finalize().into_bytes();
    hex::encode(result)
}

/// Generate a secure random token (32 bytes base64 encoded)
pub fn generate_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    base64::encode(&bytes).replace('/', "_").trim_end_matches('=').to_string()
}