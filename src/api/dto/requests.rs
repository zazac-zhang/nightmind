// ============================================================
// Request DTOs
// ============================================================
//! Request data transfer objects.
//!
//! This module defines all request structures for API endpoints.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// Login request
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct LoginRequest {
    /// Username or email
    #[validate(length(min = 1, max = 100))]
    pub identifier: String,
    /// User password
    #[validate(length(min = 8))]
    pub password: String,
}

/// Register request
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct RegisterRequest {
    /// Username
    #[validate(length(min = 3, max = 50))]
    pub username: String,
    /// Email address
    #[validate(email)]
    pub email: String,
    /// Password
    #[validate(length(min = 8))]
    pub password: String,
    /// Optional display name
    #[validate(length(max = 100))]
    pub display_name: Option<String>,
}

/// Create session request
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct CreateSessionRequest {
    /// Session title
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    /// Optional initial state
    pub initial_state: Option<String>,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Update session request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateSessionRequest {
    /// New title
    pub title: Option<String>,
    /// New state
    pub state: Option<String>,
    /// Topic stack (JSON array)
    pub topic_stack: Option<serde_json::Value>,
    /// Cognitive load (0.0 - 1.0)
    pub cognitive_load: Option<f32>,
    /// Metadata
    pub metadata: Option<serde_json::Value>,
}

/// Create knowledge point request
#[derive(Debug, Clone, Deserialize, Serialize, Validate)]
pub struct CreateKnowledgeRequest {
    /// Knowledge title
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    /// Knowledge content
    #[validate(length(min = 1))]
    pub content: String,
    /// Category
    pub category: Option<String>,
    /// Tags
    pub tags: Option<Vec<String>>,
    /// Related session ID
    pub session_id: Option<Uuid>,
}

/// Update knowledge request
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateKnowledgeRequest {
    /// Title
    pub title: Option<String>,
    /// Content
    pub content: Option<String>,
    /// Category
    pub category: Option<String>,
    /// Tags
    pub tags: Option<Vec<String>>,
}

/// List query parameters
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListQuery {
    /// Page number (0-indexed)
    pub page: Option<usize>,
    /// Items per page
    pub limit: Option<usize>,
    /// Sort field
    pub sort: Option<String>,
    /// Sort order
    pub order: Option<String>,
}

impl ListQuery {
    /// Gets the page number (default 0)
    #[must_use]
    pub fn page(&self) -> usize {
        self.page.unwrap_or(0)
    }

    /// Gets the limit (default 20)
    #[must_use]
    pub fn limit(&self) -> usize {
        self.limit.unwrap_or(20).min(100)
    }

    /// Calculates the offset
    #[must_use]
    pub fn offset(&self) -> usize {
        self.page() * self.limit()
    }
}

/// Pagination info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    /// Current page number
    pub page: usize,
    /// Items per page
    pub limit: usize,
    /// Total number of items
    pub total: u64,
    /// Total number of pages
    pub pages: usize,
}

impl PaginationInfo {
    /// Creates pagination info from total count
    #[must_use]
    pub fn new(page: usize, limit: usize, total: u64) -> Self {
        let pages = if total == 0 {
            0
        } else {
            ((total as usize).saturating_sub(1) / limit) + 1
        };

        Self {
            page,
            limit,
            total,
            pages,
        }
    }

    /// Creates paginated response
    #[must_use]
    pub fn response<T>(self, items: Vec<T>) -> PagedResponse<T> {
        PagedResponse {
            items,
            pagination: self,
        }
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedResponse<T> {
    /// Items in current page
    pub items: Vec<T>,
    /// Pagination metadata
    pub pagination: PaginationInfo,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_query_defaults() {
        let query = ListQuery {
            page: None,
            limit: None,
            sort: None,
            order: None,
        };

        assert_eq!(query.page(), 0);
        assert_eq!(query.limit(), 20);
        assert_eq!(query.offset(), 0);
    }

    #[test]
    fn test_list_query_custom() {
        let query = ListQuery {
            page: Some(2),
            limit: Some(10),
            sort: Some("created_at".to_string()),
            order: Some("desc".to_string()),
        };

        assert_eq!(query.page(), 2);
        assert_eq!(query.limit(), 10);
        assert_eq!(query.offset(), 20);
    }

    #[test]
    fn test_pagination_info() {
        let pagination = PaginationInfo::new(0, 10, 100);

        assert_eq!(pagination.page, 0);
        assert_eq!(pagination.limit, 10);
        assert_eq!(pagination.total, 100);
        assert_eq!(pagination.pages, 10);
    }

    #[test]
    fn test_pagination_info_empty() {
        let pagination = PaginationInfo::new(0, 10, 0);

        assert_eq!(pagination.pages, 0);
    }
}
