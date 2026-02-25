// ============================================================
// Data Transfer Objects
// ============================================================
//! Request and response DTOs for the NightMind API.
//!
//! This module defines the structure of data transferred between
//! clients and the API.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDto {
    /// Unique user identifier
    pub id: Uuid,
    /// User display name
    pub display_name: String,
    /// User email
    pub email: String,
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Session DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDto {
    /// Unique session identifier
    pub id: Uuid,
    /// Associated user ID
    pub user_id: Uuid,
    /// Session status
    pub status: String,
    /// Session start time
    pub started_at: DateTime<Utc>,
    /// Session end time (if completed)
    pub ended_at: Option<DateTime<Utc>>,
}

/// Message DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDto {
    /// Unique message identifier
    pub id: Uuid,
    /// Associated session ID
    pub session_id: Uuid,
    /// Message content
    pub content: String,
    /// Message role (user/assistant/system)
    pub role: String,
    /// Message timestamp
    pub timestamp: DateTime<Utc>,
}

/// Knowledge item DTO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDto {
    /// Unique knowledge identifier
    pub id: Uuid,
    /// Knowledge title
    pub title: String,
    /// Knowledge content
    pub content: String,
    /// Associated user ID
    pub user_id: Uuid,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Request ID for tracing
    pub request_id: Option<String>,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// Response items
    pub items: Vec<T>,
    /// Total count
    pub total: u64,
    /// Current page
    pub page: u32,
    /// Page size
    pub page_size: u32,
}

/// WebSocket message type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    /// Text message
    #[serde(rename = "text")]
    Text { content: String },
    /// Binary data
    #[serde(rename = "binary")]
    Binary { data: Vec<u8> },
    /// Control message
    #[serde(rename = "control")]
    Control { command: String },
    /// Error message
    #[serde(rename = "error")]
    Error { message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_dto_serialization() {
        let user = UserDto {
            id: Uuid::new_v4(),
            display_name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let json = serde_json::to_string(&user);
        assert!(json.is_ok());
    }
}
