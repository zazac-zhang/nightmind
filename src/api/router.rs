// ============================================================
// API Router
// ============================================================
//! Route definitions and router configuration for the NightMind API.
//!
//! This module sets up the Axum router with all API endpoints.

use axum::{
    routing::{get, post},
    Router,
};

use crate::api::handlers::{AppState, health_check, websocket_handler};

/// Creates and configures the API router
///
/// # Arguments
///
/// * `state` - Shared application state
///
/// # Returns
///
/// A configured Axum router with all routes mounted
#[must_use]
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health_check))
        // WebSocket endpoint
        .route("/ws", get(websocket_handler))
        // Session management
        .route("/sessions", post(crate::api::handlers::session_create))
        .route("/sessions/:id", get(crate::api::handlers::session_get))
        // Authentication
        .route("/auth/login", post(crate::api::handlers::login))
        .route("/auth/logout", post(crate::api::handlers::logout))
        // Knowledge base
        .route("/knowledge", get(crate::api::handlers::knowledge_list))
        .route("/knowledge", post(crate::api::handlers::knowledge_create))
        .with_state(state)
}

/// Creates a new router instance without state (for testing)
#[cfg(test)]
#[must_use]
pub fn create_test_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        // Note: websocket_handler requires AppState, so it's excluded from test router
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_creation() {
        let router = create_test_router();
        // Router creation should not panic
        let _ = router;
    }
}
