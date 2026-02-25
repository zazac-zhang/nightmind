// ============================================================
// Error Handling
// ============================================================
//! Unified error handling for NightMind.
//!
//! This module defines the core error types and conversions
//! for consistent error reporting across the application.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::fmt;

/// API error response structure
#[derive(Debug)]
pub struct ErrorResponse {
    pub status: StatusCode,
    pub error_type: String,
    pub message: String,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        let body = json!({
            "error": self.error_type,
            "message": self.message,
        });
        (self.status, Json(body)).into_response()
    }
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.error_type, self.message)
    }
}

impl std::error::Error for ErrorResponse {}

/// NightMind core error type
///
/// This enum represents all possible errors that can occur
/// within the NightMind application.
#[derive(Debug, thiserror::Error)]
pub enum NightMindError {
    /// Database-related errors
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Redis-related errors
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    /// AI/LLM service errors
    #[error("AI service error: {0}")]
    AiService(String),

    /// Vector database (Qdrant) errors
    #[error("Vector store error: {0}")]
    VectorStore(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Resource not found errors
    #[error("Not found: {0}")]
    NotFound(String),

    /// Authentication errors
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Authorization errors (unauthorized access)
    #[error("Unauthorized access")]
    Unauthorized,

    /// Bad request errors (invalid input)
    #[error("Invalid request: {0}")]
    BadRequest(String),

    /// Rate limiting errors
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Session-related errors
    #[error("Session error: {0}")]
    Session(String),

    /// Integration errors (Anki, Obsidian, etc.)
    #[error("Integration error: {service}: {error}")]
    Integration { service: String, error: String },

    /// Generic internal server errors
    #[error("Internal server error: {0}")]
    Internal(String),

    /// IO errors
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// UUID parsing errors
    #[error("Invalid UUID: {0}")]
    InvalidUuid(String),
}

impl NightMindError {
    /// Get the HTTP status code for this error
    #[must_use]
    pub const fn status_code(&self) -> StatusCode {
        match self {
            NightMindError::NotFound(_) => StatusCode::NOT_FOUND,
            NightMindError::Unauthorized | NightMindError::Auth(_) => StatusCode::UNAUTHORIZED,
            NightMindError::BadRequest(_) => StatusCode::BAD_REQUEST,
            NightMindError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            NightMindError::AiService(_) => StatusCode::SERVICE_UNAVAILABLE,
            NightMindError::Database(_)
            | NightMindError::Redis(_)
            | NightMindError::Internal(_)
            | NightMindError::VectorStore(_)
            | NightMindError::Config(_)
            | NightMindError::Session(_)
            | NightMindError::Integration { .. }
            | NightMindError::Io(_)
            | NightMindError::Json(_)
            | NightMindError::InvalidUuid(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get the error type name for API responses
    #[must_use]
    pub const fn error_type(&self) -> &str {
        match self {
            NightMindError::Database(_) => "DATABASE_ERROR",
            NightMindError::Redis(_) => "REDIS_ERROR",
            NightMindError::AiService(_) => "AI_SERVICE_ERROR",
            NightMindError::VectorStore(_) => "VECTOR_STORE_ERROR",
            NightMindError::Config(_) => "CONFIG_ERROR",
            NightMindError::NotFound(_) => "NOT_FOUND",
            NightMindError::Auth(_) => "AUTHENTICATION_ERROR",
            NightMindError::Unauthorized => "UNAUTHORIZED",
            NightMindError::BadRequest(_) => "BAD_REQUEST",
            NightMindError::RateLimitExceeded => "RATE_LIMIT_EXCEEDED",
            NightMindError::Session(_) => "SESSION_ERROR",
            NightMindError::Integration { .. } => "INTEGRATION_ERROR",
            NightMindError::Internal(_) => "INTERNAL_ERROR",
            NightMindError::Io(_) => "IO_ERROR",
            NightMindError::Json(_) => "JSON_ERROR",
            NightMindError::InvalidUuid(_) => "INVALID_UUID",
        }
    }

    /// Convert to an API error response
    #[must_use]
    pub fn into_error_response(self) -> ErrorResponse {
        ErrorResponse {
            status: self.status_code(),
            error_type: self.error_type().to_string(),
            message: self.to_string(),
        }
    }

    /// Create a not found error
    #[must_use]
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound(resource.into())
    }

    /// Create a bad request error
    #[must_use]
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    /// Create an authentication error
    #[must_use]
    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Auth(msg.into())
    }

    /// Create an internal error
    #[must_use]
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

impl IntoResponse for NightMindError {
    fn into_response(self) -> Response {
        // Log the error for debugging
        match &self {
            NightMindError::Database(e) => {
                tracing::error!("Database error: {:?}", e);
            }
            NightMindError::Redis(e) => {
                tracing::error!("Redis error: {:?}", e);
            }
            NightMindError::AiService(e) => {
                tracing::warn!("AI service error: {}", e);
            }
            NightMindError::NotFound(e) => {
                tracing::debug!("Not found: {}", e);
            }
            NightMindError::Unauthorized | NightMindError::Auth(_) => {
                tracing::warn!("Authentication/Authorization error: {}", self);
            }
            NightMindError::BadRequest(e) => {
                tracing::debug!("Bad request: {}", e);
            }
            _ => {
                tracing::error!("Internal error: {}", self);
            }
        }

        self.into_error_response().into_response()
    }
}

/// Type alias for Result with NightMindError
pub type Result<T> = std::result::Result<T, NightMindError>;

// ============================================================================
// Error conversion helpers for external libraries
// ============================================================================

/// Convert UUID errors
impl From<uuid::Error> for NightMindError {
    fn from(err: uuid::Error) -> Self {
        NightMindError::InvalidUuid(err.to_string())
    }
}

/// Convert config errors
impl From<config::ConfigError> for NightMindError {
    fn from(err: config::ConfigError) -> Self {
        NightMindError::Config(err.to_string())
    }
}

/// Convert tokio task join errors
impl From<tokio::task::JoinError> for NightMindError {
    fn from(err: tokio::task::JoinError) -> Self {
        NightMindError::Internal(format!("Task join error: {}", err))
    }
}

/// Convert time errors
impl From<time::error::Error> for NightMindError {
    fn from(err: time::error::Error) -> Self {
        NightMindError::Internal(format!("Time error: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(NightMindError::NotFound("test".to_string()).status_code(), StatusCode::NOT_FOUND);
        assert_eq!(NightMindError::BadRequest("test".to_string()).status_code(), StatusCode::BAD_REQUEST);
        assert_eq!(NightMindError::Unauthorized.status_code(), StatusCode::UNAUTHORIZED);
        assert_eq!(NightMindError::RateLimitExceeded.status_code(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn test_error_type_names() {
        assert_eq!(NightMindError::NotFound("test".to_string()).error_type(), "NOT_FOUND");
        assert_eq!(NightMindError::BadRequest("test".to_string()).error_type(), "BAD_REQUEST");
        assert_eq!(NightMindError::Unauthorized.error_type(), "UNAUTHORIZED");
    }

    #[test]
    fn test_error_constructors() {
        let err = NightMindError::not_found("User");
        assert!(matches!(err, NightMindError::NotFound(_)));

        let err = NightMindError::bad_request("Invalid input");
        assert!(matches!(err, NightMindError::BadRequest(_)));

        let err = NightMindError::auth("Invalid token");
        assert!(matches!(err, NightMindError::Auth(_)));

        let err = NightMindError::internal("Something went wrong");
        assert!(matches!(err, NightMindError::Internal(_)));
    }

    #[test]
    fn test_error_display() {
        let err = NightMindError::NotFound("User not found".to_string());
        assert_eq!(err.to_string(), "Not found: User not found");

        let err = NightMindError::BadRequest("Invalid input".to_string());
        assert_eq!(err.to_string(), "Invalid request: Invalid input");
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_ok() -> Result<String> {
            Ok("success".to_string())
        }

        fn returns_err() -> Result<String> {
            Err(NightMindError::not_found("Resource"))
        }

        assert!(returns_ok().is_ok());
        assert!(returns_err().is_err());
    }
}
