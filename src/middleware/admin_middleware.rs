use crate::error::AppError;
use crate::utils::jwt::decode_jwt;
use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::Response,
};

pub async fn admin_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(auth_header) if auth_header.starts_with("Bearer ") => {
            let token = &auth_header[7..];
            let claims = decode_jwt(token)
                .map_err(|_| AppError::Unauthorized("Invalid token".to_string()))?;

            if claims.role != "admin" {
                return Err(AppError::Forbidden("Admin role required".to_string()));
            }

            req.extensions_mut().insert(claims);
            Ok(next.run(req).await)
        }
        _ => Err(AppError::Unauthorized(
            "Missing or invalid authorization header".to_string(),
        )),
    }
}
