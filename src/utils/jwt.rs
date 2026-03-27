use crate::dtos::claims::Claims;
use crate::error::AppError;
use chrono::{Duration, Utc, TimeZone};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use std::env;

fn get_env_var(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

pub fn encode_access_token(sub: String, role: String) -> Result<String, AppError> {
    let access_secret = get_env_var("JWT_SECRET", "secret");
    let access_exp_seconds = get_env_var("ACCESS_TOKEN_EXPIRE", "900")
        .parse::<i64>()
        .unwrap_or(900);
    
    let now = Utc::now();
    let expire = now + Duration::seconds(access_exp_seconds);

    let claims = Claims {
        sub,
        role,
        iat: now.timestamp() as usize,
        exp: expire.timestamp() as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(access_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))
}

pub fn encode_refresh_token(sub: String, role: String, created_at: i64) -> Result<(String, i64), AppError> {
    let refresh_secret = get_env_var("JWT_REFRESH_SECRET", "refresh_secret");
    let refresh_exp_seconds = get_env_var("REFRESH_TOKEN_EXPIRE", "2592000") // 30 days
        .parse::<i64>()
        .unwrap_or(2592000);
    
    let now = Utc.timestamp_opt(created_at, 0).unwrap();
    let expire = now + Duration::seconds(refresh_exp_seconds);
    let exp_timestamp = expire.timestamp();

    let claims = Claims {
        sub,
        role,
        iat: now.timestamp() as usize,
        exp: exp_timestamp as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(refresh_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok((token, exp_timestamp))
}

pub fn decode_jwt(token: &str) -> Result<Claims, AppError> {
    // Try decoding as access token first
    let access_secret = get_env_var("JWT_SECRET", "secret");
    let validation = Validation::default();
    
    if let Ok(data) = decode::<Claims>(
        token,
        &DecodingKey::from_secret(access_secret.as_bytes()),
        &validation,
    ) {
        return Ok(data.claims);
    }
    
    Err(AppError::Unauthorized("Invalid token".to_string()))
}

pub fn decode_refresh_token(token: &str) -> Result<Claims, AppError> {
    let refresh_secret = get_env_var("JWT_REFRESH_SECRET", "refresh_secret");
    let validation = Validation::default();
    
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(refresh_secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("Invalid refresh token".to_string()))
}
