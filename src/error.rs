use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub status: u16,
}

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    #[error("Too many requests: {0}")]
    TooManyRequests(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Gateway timeout: {0}")]
    GatewayTimeout(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis pool error: {0}")]
    RedisPool(#[from] deadpool_redis::PoolError),

    #[error("Redis error: {0}")]
    Redis(#[from] deadpool_redis::redis::RedisError),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),

    #[error("Date parse error: {0}")]
    DateParse(#[from] chrono::ParseError),

    #[error("Integer parse error: {0}")]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("UUID parse error: {0}")]
    UuidParse(#[from] uuid::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("System time error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),

    #[error("JSON error: {0}")]
    JsonPayload(#[from] serde_json::Error),
}

impl AppError {
    fn postgres_status_from_code(code: Option<&str>) -> StatusCode {
        match code {
            Some("23505") => StatusCode::CONFLICT,
            Some("23503") | Some("23514") | Some("22P02") => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn postgres_message_from_code(code: Option<&str>) -> &'static str {
        match code {
            Some("23505") => "Resource already exists",
            Some("23503") => "Related resource does not exist",
            Some("23514") => "Data violates database constraints",
            Some("22P02") => "Invalid database input format",
            _ => "Database error",
        }
    }

    fn database_error_code(error: &sqlx::Error) -> Option<String> {
        error
            .as_database_error()
            .and_then(|db_error| db_error.code().map(|code| code.into_owned()))
    }

    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) | AppError::Jwt(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::BadRequest(_)
            | AppError::DateParse(_)
            | AppError::ParseInt(_)
            | AppError::UuidParse(_)
            | AppError::UrlParse(_)
            | AppError::JsonPayload(_) => StatusCode::BAD_REQUEST,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::UnprocessableEntity(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::TooManyRequests(_) => StatusCode::TOO_MANY_REQUESTS,
            AppError::ServiceUnavailable(_) | AppError::RedisPool(_) | AppError::Redis(_) => {
                StatusCode::SERVICE_UNAVAILABLE
            }
            AppError::GatewayTimeout(_) => StatusCode::GATEWAY_TIMEOUT,
            AppError::Database(error) => {
                Self::postgres_status_from_code(Self::database_error_code(error).as_deref())
            }
            AppError::Internal(_) | AppError::EnvVar(_) | AppError::SystemTime(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        }
    }

    fn message(&self) -> &str {
        match self {
            AppError::NotFound(msg)
            | AppError::Unauthorized(msg)
            | AppError::Forbidden(msg)
            | AppError::BadRequest(msg)
            | AppError::Conflict(msg)
            | AppError::UnprocessableEntity(msg)
            | AppError::TooManyRequests(msg)
            | AppError::ServiceUnavailable(msg)
            | AppError::GatewayTimeout(msg)
            | AppError::Internal(msg) => msg.as_str(),
            AppError::Database(error) => {
                Self::postgres_message_from_code(Self::database_error_code(error).as_deref())
            }
            AppError::RedisPool(_) => "Redis pool unavailable",
            AppError::Redis(_) => "Redis error",
            AppError::Jwt(_) => "Invalid token",
            AppError::EnvVar(_) => "Configuration error",
            AppError::DateParse(_) => "Invalid date format",
            AppError::ParseInt(_) => "Invalid numeric value",
            AppError::UuidParse(_) => "Invalid UUID format",
            AppError::UrlParse(_) => "Invalid URL format",
            AppError::SystemTime(_) => "System clock error",
            AppError::JsonPayload(_) => "Invalid JSON payload",
        }
    }

    fn log_if_needed(&self, status: StatusCode) {
        if status.is_server_error() {
            tracing::error!(error = ?self, "request failed with server error");
            return;
        }

        if matches!(
            self,
            AppError::Unauthorized(_)
                | AppError::Forbidden(_)
                | AppError::TooManyRequests(_)
                | AppError::Jwt(_)
        ) {
            tracing::warn!(error = ?self, "request rejected");
        }
    }
}

impl From<crate::services::cache_service::CacheError> for AppError {
    fn from(value: crate::services::cache_service::CacheError) -> Self {
        match value {
            crate::services::cache_service::CacheError::Pool(error) => Self::RedisPool(error),
            crate::services::cache_service::CacheError::Redis(error) => Self::Redis(error),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        self.log_if_needed(status);

        let body = Json(ErrorResponse {
            error: self.message().to_string(),
            status: status.as_u16(),
        });

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;

#[cfg(test)]
mod tests {
    use super::AppError;
    use axum::http::StatusCode;

    #[test]
    fn postgres_23505_maps_to_conflict() {
        assert_eq!(
            AppError::postgres_status_from_code(Some("23505")),
            StatusCode::CONFLICT
        );
        assert_eq!(
            AppError::postgres_message_from_code(Some("23505")),
            "Resource already exists"
        );
    }

    #[test]
    fn postgres_23503_maps_to_bad_request() {
        assert_eq!(
            AppError::postgres_status_from_code(Some("23503")),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::postgres_message_from_code(Some("23503")),
            "Related resource does not exist"
        );
    }

    #[test]
    fn postgres_23514_maps_to_bad_request() {
        assert_eq!(
            AppError::postgres_status_from_code(Some("23514")),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::postgres_message_from_code(Some("23514")),
            "Data violates database constraints"
        );
    }

    #[test]
    fn postgres_22p02_maps_to_bad_request() {
        assert_eq!(
            AppError::postgres_status_from_code(Some("22P02")),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::postgres_message_from_code(Some("22P02")),
            "Invalid database input format"
        );
    }

    #[test]
    fn unknown_postgres_code_maps_to_internal_server_error() {
        assert_eq!(
            AppError::postgres_status_from_code(Some("99999")),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(
            AppError::postgres_message_from_code(Some("99999")),
            "Database error"
        );
    }

    #[test]
    fn missing_postgres_code_maps_to_internal_server_error() {
        assert_eq!(
            AppError::postgres_status_from_code(None),
            StatusCode::INTERNAL_SERVER_ERROR
        );
        assert_eq!(AppError::postgres_message_from_code(None), "Database error");
    }
}