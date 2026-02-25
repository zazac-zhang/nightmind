// ============================================================
// API Handlers
// ============================================================
//! WebSocket and HTTP request handlers for the NightMind API.
//!
//! This module provides handler functions for processing incoming requests
//! and managing WebSocket connections.

use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    response::{IntoResponse, Json, Response},
    http::StatusCode,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::api::dto::*;
use crate::error::{NightMindError, Result};
use crate::config::Settings;

/// Application state shared by all handlers
#[derive(Clone)]
pub struct AppState {
    /// Application settings
    pub settings: Arc<Settings>,
    /// Database connection pool
    pub db_pool: Arc<sqlx::PgPool>,
    /// Redis client
    pub redis: Arc<redis::Client>,
}

// ============================================================
// Health Check Handlers
// ============================================================

/// Health check handler
pub async fn health_check() -> impl IntoResponse {
    Json(HealthResponse::healthy())
}

/// Database health check handler
pub async fn database_health(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    sqlx::query("SELECT 1")
        .fetch_one(state.db_pool.as_ref())
        .await
        .map_err(|e| NightMindError::Internal(format!("Database health check failed: {}", e)))?;

    Ok(Json(serde_json::json!({
        "status": "healthy",
        "database": "connected"
    })))
}

/// Redis health check handler
pub async fn redis_health(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let mut conn = state.redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| NightMindError::Internal(format!("Redis connection failed: {}", e)))?;

    redis::cmd("PING")
        .query_async::<String>(&mut conn)
        .await
        .map_err(|e| NightMindError::Internal(format!("Redis health check failed: {}", e)))?;

    Ok(Json(serde_json::json!({
        "status": "healthy",
        "redis": "connected"
    })))
}

/// AI service health check handler
pub async fn ai_service_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "ai_service": "available"
    }))
}

/// Not found handler
pub async fn not_found_handler() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, Json(ErrorResponse::new(
        "not_found",
        "The requested resource was not found",
    )))
}

// ============================================================
// Authentication Handlers
// ============================================================

/// Login handler
pub async fn login(
    State(_state): State<AppState>,
    Json(_req): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>> {
    let user_id = Uuid::new_v4();
    let token = format!("jwt_token_{}", user_id);

    let response = LoginResponse {
        token,
        user: UserProfileResponse {
            id: user_id,
            username: "test_user".to_string(),
            email: "test@example.com".to_string(),
            display_name: Some("Test User".to_string()),
            role: "user".to_string(),
            is_active: true,
            is_verified: true,
            created_at: chrono::Utc::now(),
        },
        expires_at: chrono::Utc::now() + chrono::Duration::days(7),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Register handler
pub async fn register(
    State(_state): State<AppState>,
    Json(_req): Json<RegisterRequest>,
) -> Result<Json<ApiResponse<UserProfileResponse>>> {
    let user_id = Uuid::new_v4();

    let response = UserProfileResponse {
        id: user_id,
        username: "new_user".to_string(),
        email: "new@example.com".to_string(),
        display_name: None,
        role: "user".to_string(),
        is_active: true,
        is_verified: false,
        created_at: chrono::Utc::now(),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Logout handler
pub async fn logout(
    State(_state): State<AppState>,
) -> Result<Json<ApiResponse<()>>> {
    Ok(Json(ApiResponse::ok()))
}

/// Refresh token handler
pub async fn refresh_token(
    State(_state): State<AppState>,
) -> Result<Json<ApiResponse<()>>> {
    Ok(Json(ApiResponse::success_with_message(
        (),
        "Token refreshed",
    )))
}

/// Verify token handler
pub async fn verify_token(
    State(_state): State<AppState>,
) -> Result<Json<ApiResponse<bool>>> {
    Ok(Json(ApiResponse::success(true)))
}

// ============================================================
// User Management Handlers
// ============================================================

/// Get current user handler
pub async fn get_current_user(
    State(_state): State<AppState>,
) -> Result<Json<ApiResponse<UserProfileResponse>>> {
    let user_id = Uuid::new_v4();

    let response = UserProfileResponse {
        id: user_id,
        username: "current_user".to_string(),
        email: "current@example.com".to_string(),
        display_name: Some("Current User".to_string()),
        role: "user".to_string(),
        is_active: true,
        is_verified: true,
        created_at: chrono::Utc::now(),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Update current user handler
pub async fn update_current_user(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<()>>> {
    Ok(Json(ApiResponse::success_with_message(
        (),
        "Profile updated",
    )))
}

/// Change password handler
pub async fn change_password(
    State(_state): State<AppState>,
    Json(_req): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<()>>> {
    Ok(Json(ApiResponse::ok()))
}

/// Get user sessions handler
pub async fn get_user_sessions(
    State(_state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<SessionSummaryResponse>>>> {
    Ok(Json(ApiResponse::success(Vec::new())))
}

// ============================================================
// Session Management Handlers
// ============================================================

/// Create session handler
pub async fn create_session(
    State(_state): State<AppState>,
    Json(_req): Json<CreateSessionRequest>,
) -> Result<Json<ApiResponse<SessionDetailResponse>>> {
    let session_id = Uuid::new_v4();

    let response = SessionDetailResponse {
        id: session_id,
        user_id: Uuid::new_v4(),
        title: "New Learning Session".to_string(),
        state: "warmup".to_string(),
        topic_stack: None,
        cognitive_load: 0.0,
        started_at: chrono::Utc::now(),
        last_activity_at: chrono::Utc::now(),
        ended_at: None,
        metadata: None,
        message_count: 0,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// List sessions handler
pub async fn list_sessions(
    State(_state): State<AppState>,
    Query(_query): Query<ListQuery>,
) -> Result<Json<ApiResponse<PagedResponse<SessionSummaryResponse>>>> {
    let pagination = PaginationInfo::new(0, 20, 0);
    let response = pagination.response(Vec::new());
    Ok(Json(ApiResponse::success(response)))
}

/// Get session handler
pub async fn get_session(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<Json<ApiResponse<SessionDetailResponse>>> {
    let session_id = Uuid::new_v4();

    let response = SessionDetailResponse {
        id: session_id,
        user_id: Uuid::new_v4(),
        title: "Learning Session".to_string(),
        state: "warmup".to_string(),
        topic_stack: None,
        cognitive_load: 0.3,
        started_at: chrono::Utc::now(),
        last_activity_at: chrono::Utc::now(),
        ended_at: None,
        metadata: None,
        message_count: 5,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Update session handler
pub async fn update_session(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
    Json(_req): Json<UpdateSessionRequest>,
) -> Result<Json<ApiResponse<()>>> {
    Ok(Json(ApiResponse::success_with_message(
        (),
        "Session updated",
    )))
}

/// Delete session handler
pub async fn delete_session(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<Json<ApiResponse<EmptyResponse>>> {
    Ok(Json(ApiResponse::success(EmptyResponse::deleted())))
}

/// Pause session handler
pub async fn pause_session(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>> {
    Ok(Json(ApiResponse::success_with_message(
        (),
        "Session paused",
    )))
}

/// Resume session handler
pub async fn resume_session(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>> {
    Ok(Json(ApiResponse::success_with_message(
        (),
        "Session resumed",
    )))
}

/// End session handler
pub async fn end_session(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>> {
    Ok(Json(ApiResponse::success_with_message(
        (),
        "Session ended",
    )))
}

/// Get session messages handler
pub async fn get_session_messages(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
    Query(_query): Query<ListQuery>,
) -> Result<Json<ApiResponse<PagedResponse<serde_json::Value>>>> {
    let pagination = PaginationInfo::new(0, 20, 0);
    let response = pagination.response(Vec::new());
    Ok(Json(ApiResponse::success(response)))
}

/// Get active session handler
pub async fn get_active_session(
    State(_state): State<AppState>,
) -> Result<Json<ApiResponse<Option<SessionDetailResponse>>>> {
    Ok(Json(ApiResponse::success(None)))
}

// ============================================================
// Knowledge Point Handlers
// ============================================================

/// Create knowledge handler
pub async fn create_knowledge(
    State(_state): State<AppState>,
    Json(_req): Json<CreateKnowledgeRequest>,
) -> Result<Json<ApiResponse<KnowledgePointResponse>>> {
    let knowledge_id = Uuid::new_v4();

    let response = KnowledgePointResponse {
        id: knowledge_id,
        title: "New Knowledge".to_string(),
        content: "Knowledge content".to_string(),
        category: None,
        tags: vec![],
        user_id: Uuid::new_v4(),
        session_id: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// List knowledge handler
pub async fn list_knowledge(
    State(_state): State<AppState>,
    Query(_query): Query<ListQuery>,
) -> Result<Json<ApiResponse<PagedResponse<KnowledgePointResponse>>>> {
    let pagination = PaginationInfo::new(0, 20, 0);
    let response = pagination.response(Vec::new());
    Ok(Json(ApiResponse::success(response)))
}

/// Get knowledge handler
pub async fn get_knowledge(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<Json<ApiResponse<KnowledgePointResponse>>> {
    let knowledge_id = Uuid::new_v4();

    let response = KnowledgePointResponse {
        id: knowledge_id,
        title: "Knowledge Point".to_string(),
        content: "Knowledge content here".to_string(),
        category: Some("general".to_string()),
        tags: vec!["tag1".to_string()],
        user_id: Uuid::new_v4(),
        session_id: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    Ok(Json(ApiResponse::success(response)))
}

/// Update knowledge handler
pub async fn update_knowledge(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
    Json(_req): Json<UpdateKnowledgeRequest>,
) -> Result<Json<ApiResponse<()>>> {
    Ok(Json(ApiResponse::success_with_message(
        (),
        "Knowledge updated",
    )))
}

/// Delete knowledge handler
pub async fn delete_knowledge(
    State(_state): State<AppState>,
    Path(_id): Path<Uuid>,
) -> Result<Json<ApiResponse<EmptyResponse>>> {
    Ok(Json(ApiResponse::success(EmptyResponse::deleted())))
}

/// Search knowledge handler
pub async fn search_knowledge(
    State(_state): State<AppState>,
    Query(_query): Query<serde_json::Value>,
) -> Result<Json<ApiResponse<PagedResponse<KnowledgePointResponse>>>> {
    let pagination = PaginationInfo::new(0, 20, 0);
    let response = pagination.response(Vec::new());
    Ok(Json(ApiResponse::success(response)))
}

/// List knowledge categories handler
pub async fn list_knowledge_categories(
    State(_state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<String>>>> {
    Ok(Json(ApiResponse::success(Vec::new())))
}

// ============================================================
// WebSocket Handler
// ============================================================

/// WebSocket connection upgrade handler
pub async fn websocket_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    crate::api::websocket::websocket_handler(State(state), ws).await
}

// ============================================================
// Middleware
// ============================================================

pub mod middleware {
    use super::*;

    /// Authentication middleware
    pub async fn auth_middleware(
        _state: AppState,
        _request: axum::extract::Request,
        _next: axum::middleware::Next,
    ) -> Response {
        // TODO: Implement JWT token validation
        _next.run(_request).await
    }

    /// Logging middleware
    pub async fn logging_middleware(
        _request: axum::extract::Request,
        _next: axum::middleware::Next,
    ) -> Response {
        let start = std::time::Instant::now();
        let method = _request.method().clone();
        let uri = _request.uri().clone();

        let response = _next.run(_request).await;

        let duration = start.elapsed();
        let status = response.status();

        tracing::info!(
            method = %method,
            uri = %uri,
            status = %status,
            duration_ms = duration.as_millis(),
            "Request completed"
        );

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_response() {
        let health = HealthResponse::healthy();
        assert_eq!(health.status, "healthy");
    }
}
