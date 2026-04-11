// ============================================================
// API Router Module
// ============================================================
//! API route definitions and router configuration.
//!
//! This module defines all HTTP routes for the NightMind API.

use axum::{
    Router,
    routing::{get, post, put, delete},
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    compression::CompressionLayer,
    request_id::{MakeRequestUuid, SetRequestIdLayer},
};
use std::sync::Arc;
use http::Method;
use http::header::HeaderName;

use crate::api::handlers::*;
use crate::config::Settings;
use crate::core::agent::NightMindAgent;
use crate::error::{NightMindError, Result};

/// Creates the main application router
///
/// # Errors
///
/// Returns an error if router creation fails
pub fn create_router(settings: &Settings) -> Result<Router> {
    // Create application state
    let state = create_app_state(settings)?;

    // Create session store
    let session_store = tower_sessions::SessionManagerLayer::new(
        tower_sessions_memory_store::MemoryStore::default(),
    );

    // Configure CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::OPTIONS,
        ])
        .allow_headers(Any);

    // Build public API routes
    let public_routes = Router::new()
        // Health check
        .route("/health", get(health_check))
        .route("/health/db", get(database_health))
        .route("/health/redis", get(redis_health))
        .route("/health/ai", get(ai_service_health))

        // Authentication
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/auth/logout", post(logout))
        .route("/auth/refresh", post(refresh_token))
        .route("/auth/verify", get(verify_token));

    // Build authenticated API routes
    let authenticated_routes = Router::new()
        // User management
        .route("/users/me", get(get_current_user))
        .route("/users/me", put(update_current_user))
        .route("/users/me/password", put(change_password))
        .route("/users/me/sessions", get(get_user_sessions))

        // Session management
        .route("/sessions", post(create_session))
        .route("/sessions", get(list_sessions))
        .route("/sessions/:id", get(get_session))
        .route("/sessions/:id", put(update_session))
        .route("/sessions/:id", delete(delete_session))
        .route("/sessions/:id/pause", post(pause_session))
        .route("/sessions/:id/resume", post(resume_session))
        .route("/sessions/:id/end", post(end_session))
        .route("/sessions/:id/messages", get(get_session_messages))
        .route("/sessions/active", get(get_active_session))

        // Knowledge points
        .route("/knowledge", post(create_knowledge))
        .route("/knowledge", get(list_knowledge))
        .route("/knowledge/:id", get(get_knowledge))
        .route("/knowledge/:id", put(update_knowledge))
        .route("/knowledge/:id", delete(delete_knowledge))
        .route("/knowledge/search", get(search_knowledge))
        .route("/knowledge/categories", get(list_knowledge_categories));

    // WebSocket endpoint
    let websocket_routes = Router::new()
        .route("/ws", get(websocket_handler));

    // Combine all routes
    let api_routes = public_routes
        .merge(authenticated_routes)
        .merge(websocket_routes);

    // Build router with middleware
    let app = Router::new()
        .nest("/api", api_routes)
        .layer(session_store)
        .layer(SetRequestIdLayer::new(
            HeaderName::from_static("x-request-id"),
            MakeRequestUuid,
        ))
        .layer(cors)
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .fallback(not_found_handler)
        .with_state(state);

    Ok(app)
}

/// Creates application state from settings
fn create_app_state(settings: &Settings) -> Result<AppState> {
    let agent = Arc::new(
        NightMindAgent::from_settings(settings)
            .map_err(|e| NightMindError::Internal(format!("Failed to create agent: {e}")))?,
    );

    Ok(AppState {
        settings: Arc::new(settings.clone()),
        db_pool: Arc::new(
            sqlx::PgPool::connect_lazy(&settings.database.url)
                .map_err(|e| NightMindError::Internal(format!("Failed to create pool: {e}")))?
        ),
        redis: Arc::new(
            redis::Client::open(settings.redis.url.clone())
                .unwrap_or_else(|_| redis::Client::open("redis://127.0.0.1/").unwrap())
        ),
        agent,
    })
}

/// Creates a test router without authentication
#[cfg(test)]
#[must_use]
pub fn create_test_router() -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/api/health", get(health_check))
        .fallback(not_found_handler)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_structure() {
        let router = create_test_router();
        let _ = router;
    }
}
