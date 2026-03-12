// use sqlx::PgPool;
// use crate::dtos::auth::{RegisterRequest, LoginRequest, RefreshTokenRequest, AuthResponse};
// use crate::repositories::auth_repository;
// use crate::models::user::User;

// pub async fn register(pool: &PgPool, req: RegisterRequest) -> Result<AuthResponse, sqlx::Error> {
//     let user = auth_repository::register(pool, req).await?;
//     let tokens = auth_repository::generate_tokens(&user)?;
//     Ok(AuthResponse {
//         access_token: tokens.access_token,
//         refresh_token: tokens.refresh_token,
//         token_type: "Bearer".to_string(),
//         expires_in: 3600,
//     })
// }
// pub async fn login(pool: &PgPool, req: LoginRequest) -> Result<AuthResponse, sqlx::Error> {
//     let user = auth_repository::login(pool, req).await?;
//     let tokens = auth_repository::generate_tokens(&user)?;
//     Ok(AuthResponse {
//         access_token: tokens.access_token,
//         refresh_token: tokens.refresh_token,
//         token_type: "Bearer".to_string(),
//         expires_in: 3600,
//     })
// }
// pub async fn logout(pool: &PgPool, refresh_token: String) -> Result<(), sqlx::Error> {
//     auth_repository::logout(pool, refresh_token).await
// }
// pub async fn refresh_token(pool: &PgPool, req: RefreshTokenRequest) -> Result<AuthResponse, sqlx::Error> {
//     let user = auth_repository::refresh_token(pool, req.refresh_token).await?;
//     let tokens = auth_repository::generate_tokens(&user)?;
//     Ok(AuthResponse {
//         access_token: tokens.access_token,
//         refresh_token: tokens.refresh_token,
//         token_type: "Bearer".to_string(),
//         expires_in: 3600,
//     })
// }