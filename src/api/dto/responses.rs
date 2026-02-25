// ============================================================
// Response DTOs
// ============================================================
//! Response data transfer objects.
//!
//! This module defines all response structures for API endpoints.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Indicates if the request was successful
    pub success: bool,
    /// Response message
    pub message: String,
    /// Response data
    pub data: Option<T>,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}

impl<T> ApiResponse<T> {
    /// Creates a success response with data
    #[must_use]
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            message: "Success".to_string(),
            data: Some(data),
            metadata: None,
        }
    }

    /// Creates a success response with message
    #[must_use]
    pub fn success_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
            metadata: None,
        }
    }

    /// Creates an error response
    #[must_use]
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
            metadata: None,
        }
    }

    /// Creates an empty success response
    #[must_use]
    pub fn ok() -> Self {
        Self {
            success: true,
            message: "OK".to_string(),
            data: None,
            metadata: None,
        }
    }
}

/// User profile response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfileResponse {
    /// User ID
    pub id: Uuid,
    /// Username
    pub username: String,
    /// Email
    pub email: String,
    /// Display name
    pub display_name: Option<String>,
    /// User role
    pub role: String,
    /// Account status
    pub is_active: bool,
    /// Verification status
    pub is_verified: bool,
    /// Account creation date
    pub created_at: DateTime<Utc>,
}

/// Login response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    /// Authentication token
    pub token: String,
    /// User profile
    pub user: UserProfileResponse,
    /// Token expiry timestamp
    pub expires_at: DateTime<Utc>,
}

/// Session summary response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummaryResponse {
    /// Session ID
    pub id: Uuid,
    /// Session title
    pub title: String,
    /// Current state
    pub state: String,
    /// Start time
    pub started_at: DateTime<Utc>,
    /// Whether session is active
    pub is_active: bool,
    /// Duration in seconds (for active sessions)
    pub duration_seconds: Option<i64>,
}

/// Session detail response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDetailResponse {
    /// Session ID
    pub id: Uuid,
    /// User ID
    pub user_id: Uuid,
    /// Session title
    pub title: String,
    /// Current state
    pub state: String,
    /// Topic stack
    pub topic_stack: Option<serde_json::Value>,
    /// Cognitive load (0.0 - 1.0)
    pub cognitive_load: f32,
    /// Start time
    pub started_at: DateTime<Utc>,
    /// Last activity time
    pub last_activity_at: DateTime<Utc>,
    /// End time (if completed)
    pub ended_at: Option<DateTime<Utc>>,
    /// Metadata
    pub metadata: Option<serde_json::Value>,
    /// Messages count
    pub message_count: u64,
}

/// Knowledge point response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgePointResponse {
    /// Knowledge ID
    pub id: Uuid,
    /// Title
    pub title: String,
    /// Content
    pub content: String,
    /// Category
    pub category: Option<String>,
    /// Tags
    pub tags: Vec<String>,
    /// User ID
    pub user_id: Uuid,
    /// Session ID (if associated)
    pub session_id: Option<Uuid>,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Updated at
    pub updated_at: DateTime<Utc>,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// Service version
    pub version: String,
    /// Database status
    pub database: String,
    /// Redis status
    pub redis: String,
    /// AI service status
    pub ai_service: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl HealthResponse {
    /// Creates a healthy response
    #[must_use]
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database: "connected".to_string(),
            redis: "connected".to_string(),
            ai_service: "available".to_string(),
            timestamp: Utc::now(),
        }
    }

    /// Creates a degraded response
    #[must_use]
    pub fn degraded() -> Self {
        Self {
            status: "degraded".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            database: "unknown".to_string(),
            redis: "unknown".to_string(),
            ai_service: "unknown".to_string(),
            timestamp: Utc::now(),
        }
    }
}

/// Error detail response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error type
    pub error: String,
    /// Error message
    pub message: String,
    /// Additional details
    pub details: Option<String>,
    /// Request ID for tracing
    pub request_id: Option<String>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl ErrorResponse {
    /// Creates an error response from components
    #[must_use]
    pub fn new(error: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            details: None,
            request_id: None,
            timestamp: Utc::now(),
        }
    }

    /// Creates an error response with details
    #[must_use]
    pub fn with_details(
        error: impl Into<String>,
        message: impl Into<String>,
        details: impl Into<String>,
    ) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            details: Some(details.into()),
            request_id: None,
            timestamp: Utc::now(),
        }
    }

    /// Creates an error response with request ID
    #[must_use]
    pub fn with_request_id(
        error: impl Into<String>,
        message: impl Into<String>,
        request_id: Uuid,
    ) -> Self {
        Self {
            error: error.into(),
            message: message.into(),
            details: None,
            request_id: Some(request_id.to_string()),
            timestamp: Utc::now(),
        }
    }
}

/// Validation error detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrorDetail {
    /// Field with validation error
    pub field: String,
    /// Error message
    pub message: String,
    /// Value that failed validation
    pub value: Option<serde_json::Value>,
}

/// Validation error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationErrorResponse {
    /// General error message
    pub error: String,
    /// List of validation errors
    pub errors: Vec<ValidationErrorDetail>,
    /// Request ID
    pub request_id: Option<String>,
}

impl ValidationErrorResponse {
    /// Creates a validation error response
    #[must_use]
    pub fn new(errors: Vec<ValidationErrorDetail>) -> Self {
        Self {
            error: "Validation failed".to_string(),
            errors,
            request_id: None,
        }
    }

    /// Creates a validation error response with request ID
    #[must_use]
    pub fn with_request_id(
        errors: Vec<ValidationErrorDetail>,
        request_id: Uuid,
    ) -> Self {
        Self {
            error: "Validation failed".to_string(),
            errors,
            request_id: Some(request_id.to_string()),
        }
    }
}

/// Empty response for DELETE operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyResponse {
    /// Message indicating success
    pub message: String,
}

impl EmptyResponse {
    /// Creates a success empty response
    #[must_use]
    pub fn deleted() -> Self {
        Self {
            message: "Resource deleted successfully".to_string(),
        }
    }

    /// Creates a custom empty response
    #[must_use]
    pub fn with_message(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("test data");
        assert!(response.success);
        assert_eq!(response.data, Some("test data"));
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<()> = ApiResponse::error("Not found");
        assert!(!response.success);
    }

    #[test]
    fn test_health_response() {
        let health = HealthResponse::healthy();
        assert_eq!(health.status, "healthy");
        assert!(!health.version.is_empty());
    }

    #[test]
    fn test_validation_error_response() {
        let errors = vec![
            ValidationErrorDetail {
                field: "email".to_string(),
                message: "Invalid email".to_string(),
                value: Some(serde_json::json!("bad_email")),
            }
        ];

        let response = ValidationErrorResponse::new(errors);
        assert_eq!(response.error, "Validation failed");
        assert_eq!(response.errors.len(), 1);
    }

    #[test]
    fn test_list_query_defaults() {
        let query = crate::api::dto::requests::ListQuery {
            page: None,
            limit: None,
            sort: None,
            order: None,
        };

        assert_eq!(query.page(), 0);
        assert_eq!(query.limit(), 20);
    }
}
