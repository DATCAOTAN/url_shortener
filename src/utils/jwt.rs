use axum::http::StatusCode;
use jsonwebtoken::{DecodingKey, Validation, decode};
use crate::dtos::claims::Claims;

pub fn verify_jwt(token: &str) -> Result<Claims, StatusCode> {
    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "mysecret".into());
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(token_data.claims)
}