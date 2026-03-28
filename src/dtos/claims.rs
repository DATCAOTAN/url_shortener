use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub iat: usize,
    pub exp: usize,
    pub jti: Option<String>,
}
