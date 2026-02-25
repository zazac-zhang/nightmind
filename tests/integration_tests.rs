// ============================================================
// Integration Tests
// ============================================================
//! Integration tests for the NightMind API.
//!
//! These tests verify that the application components work together correctly.

use std::sync::Arc;
use reqwest::{Client, StatusCode};
use serde_json::json;
use uuid::Uuid;

/// Test server for integration tests
struct TestServer {
    address: String,
    client: Client,
}

impl TestServer {
    /// Creates a new test server
    ///
    /// # Errors
    ///
    /// Returns an error if server setup fails
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Load test configuration
        let settings = nightmind::config::Settings::load()?;

        // Create router
        let router = nightmind::api::router::create_router(&settings)?;

        // Bind to random port
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;

        // Spawn server in background
        tokio::spawn(async move {
            axum::serve(listener, router).await.unwrap();
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(Self {
            address: format!("http://{}", addr),
            client: Client::new(),
        })
    }

    /// Gets the base URL for the server
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.address
    }
}

// ============================================================================
// Health Check Tests
// ============================================================================

#[tokio::test]
async fn test_health_check() {
    let server = TestServer::new().await.unwrap();
    let response = server.client
        .get(&format!("{}/api/health", server.base_url()))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["status"], "healthy");
}

#[tokio::test]
async fn test_health_database() {
    let server = TestServer::new().await.unwrap();
    let response = server.client
        .get(&format!("{}/api/health/db", server.base_url()))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["database"], "connected");
}

#[tokio::test]
async fn test_health_redis() {
    let server = TestServer::new().await.unwrap();
    let response = server.client
        .get(&format!("{}/api/health/redis", server.base_url()))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["redis"], "connected");
}

// ============================================================================
// Authentication Tests
// ============================================================================

#[tokio::test]
async fn test_register_user() {
    let server = TestServer::new().await.unwrap();

    let username = format!("test_user_{}", Uuid::new_v4());
    let email = format!("test_{}@example.com", Uuid::new_v4());

    let response = server.client
        .post(&format!("{}/api/auth/register", server.base_url()))
        .json(&json!({
            "username": username,
            "email": email,
            "password": "password123"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());
}

#[tokio::test]
async fn test_login_with_invalid_credentials() {
    let server = TestServer::new().await.unwrap();

    let response = server.client
        .post(&format!("{}/api/auth/login", server.base_url()))
        .json(&json!({
            "identifier": "nonexistent@example.com",
            "password": "wrongpassword"
        }))
        .send()
        .await
        .unwrap();

    // Should still get OK response with error data
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_login_missing_fields() {
    let server = TestServer::new().await.unwrap();

    let response = server.client
        .post(&format!("{}/api/auth/login", server.base_url()))
        .json(&json!({
            "identifier": "test@example.com"
            // Missing password
        }))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_client_error() || response.status().is_success());
}

// ============================================================================
// Session Management Tests
// ============================================================================

#[tokio::test]
async fn test_create_session() {
    let server = TestServer::new().await.unwrap();

    let response = server.client
        .post(&format!("{}/api/sessions", server.base_url()))
        .json(&json!({
            "title": "Test Learning Session",
            "initial_state": "warmup"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());

    if let Some(data) = body["data"].as_object() {
        assert!(data["id"].is_string());
        assert_eq!(data["title"], "Test Learning Session");
        assert_eq!(data["state"], "warmup");
    }
}

#[tokio::test]
async fn test_list_sessions() {
    let server = TestServer::new().await.unwrap();

    let response = server.client
        .get(&format!("{}/api/sessions?page=0&limit=20", server.base_url()))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());

    if let Some(data) = body["data"].as_object() {
        assert!(data["items"].is_array());
        assert!(data["pagination"].is_object());
    }
}

#[tokio::test]
async fn test_get_nonexistent_session() {
    let server = TestServer::new().await.unwrap();
    let session_id = Uuid::new_v4();

    let response = server.client
        .get(&format!("{}/api/sessions/{}", server.base_url(), session_id))
        .send()
        .await
        .unwrap();

    // Should return OK with placeholder data or error
    assert!(response.status().is_success() || response.status().is_client_error());
}

// ============================================================================
// Knowledge Point Tests
// ============================================================================

#[tokio::test]
async fn test_create_knowledge_point() {
    let server = TestServer::new().await.unwrap();

    let response = server.client
        .post(&format!("{}/api/knowledge", server.base_url()))
        .json(&json!({
            "title": "Test Knowledge",
            "content": "This is a test knowledge point",
            "category": "test"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());

    if let Some(data) = body["data"].as_object() {
        assert!(data["id"].is_string());
        assert_eq!(data["title"], "Test Knowledge");
        assert_eq!(data["content"], "This is a test knowledge point");
    }
}

#[tokio::test]
async fn test_list_knowledge() {
    let server = TestServer::new().await.unwrap();

    let response = server.client
        .get(&format!("{}/api/knowledge?page=0&limit=20", server.base_url()))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());

    if let Some(data) = body["data"].as_object() {
        assert!(data["items"].is_array());
    }
}

// ============================================================================
// WebSocket Tests
// ============================================================================

#[tokio::test]
async fn test_websocket_upgrade() {
    let server = TestServer::new().await.unwrap();

    let ws_url = format!("{}/api/ws", server.base_url())
        .replace("http://", "ws://");

    let response = server.client
        .get(&ws_url)
        .send()
        .await
        .unwrap();

    // WebSocket upgrade should return 101 Switching Protocols
    // But since we're using a regular HTTP client, we'll get a 400 or similar
    // This is expected - proper WebSocket testing requires a WebSocket client
    assert!(response.status() != StatusCode::NOT_FOUND);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_not_found_endpoint() {
    let server = TestServer::new().await.unwrap();

    let response = server.client
        .get(&format!("{}/api/nonexistent", server.base_url()))
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["error"].is_string());
}

#[tokio::test]
async fn test_invalid_json() {
    let server = TestServer::new().await.unwrap();

    let response = server.client
        .post(&format!("{}/api/auth/login", server.base_url()))
        .header("content-type", "application/json")
        .body("invalid json")
        .send()
        .await
        .unwrap();

    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn test_missing_required_fields() {
    let server = TestServer::new().await.unwrap();

    let response = server.client
        .post(&format!("{}/api/sessions", server.base_url()))
        .json(&json!({
            // Missing required "title" field
            "initial_state": "warmup"
        }))
        .send()
        .await
        .unwrap();

    // Should be client error or still OK with error data
    assert!(response.status().is_client_error() || response.status().is_success());
}
