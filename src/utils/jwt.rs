use crate::dtos::claims::Claims;
use crate::error::AppError;
use chrono::{Duration, Utc, TimeZone};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use std::env;
use uuid::Uuid;

fn get_required_secret(key: &str) -> Result<String, AppError> {
    let value = env::var(key)
        .map_err(|_| AppError::Internal(format!("Missing required environment variable: {key}")))?;

    if value.len() < 32 {
        return Err(AppError::Internal(format!(
            "{key} must be at least 32 characters for secure JWT signing"
        )));
    }

    Ok(value)
}

fn get_env_i64(key: &str, default: i64) -> i64 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(default)
}

pub fn encode_access_token(sub: String, role: String) -> Result<String, AppError> {
    let access_secret = get_required_secret("JWT_SECRET")?;
    let access_exp_seconds = get_env_i64("ACCESS_TOKEN_EXPIRE", 900);
    
    let now = Utc::now();
    let expire = now + Duration::seconds(access_exp_seconds);

    let claims = Claims {
        sub,
        role,
        iat: now.timestamp() as usize,
        exp: expire.timestamp() as usize,
        jti: None,
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(access_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))
}

pub fn encode_refresh_token(sub: String, role: String, created_at: i64) -> Result<(String, i64), AppError> {
    let refresh_secret = get_required_secret("JWT_REFRESH_SECRET")?;
    let refresh_exp_seconds = get_env_i64("REFRESH_TOKEN_EXPIRE", 2_592_000); // 30 days
    
    let now = Utc.timestamp_opt(created_at, 0)
        .single()
        .ok_or_else(|| AppError::Internal("Invalid refresh token timestamp".to_string()))?;
    let expire = now + Duration::seconds(refresh_exp_seconds);
    let exp_timestamp = expire.timestamp();

    let claims = Claims {
        sub,
        role,
        iat: now.timestamp() as usize,
        exp: exp_timestamp as usize,
        jti: Some(Uuid::new_v4().to_string()),
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(refresh_secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok((token, exp_timestamp))
}

pub fn decode_jwt(token: &str) -> Result<Claims, AppError> {
    let access_secret = get_required_secret("JWT_SECRET")?;
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    
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
    let refresh_secret = get_required_secret("JWT_REFRESH_SECRET")?;
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;
    
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(refresh_secret.as_bytes()),
        &validation,
    )
    .map(|data| data.claims)
    .map_err(|_| AppError::Unauthorized("Invalid refresh token".to_string()))
}
