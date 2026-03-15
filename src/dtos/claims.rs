use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub id: i64,
    pub sub: String,
    pub exp: usize,
}