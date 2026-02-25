// ============================================================
// Integration Service
// ============================================================
//! External service integrations.
//!
//! This module handles integrations with external APIs and services.

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// HTTP client configuration
#[derive(Debug, Clone)]
pub struct HttpClientConfig {
    /// Request timeout
    pub timeout: Duration,
    /// Base URL for requests
    pub base_url: String,
    /// API key for authentication
    pub api_key: Option<String>,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            base_url: String::new(),
            api_key: None,
        }
    }
}

/// Integration service for external APIs
pub struct IntegrationService {
    /// HTTP client
    client: Client,
    /// Service configuration
    config: HttpClientConfig,
}

impl IntegrationService {
    /// Creates a new integration service
    ///
    /// # Arguments
    ///
    /// * `config` - Client configuration
    #[must_use]
    pub fn new(config: HttpClientConfig) -> Self {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .unwrap();

        Self { client, config }
    }

    /// Gets the base URL
    #[must_use]
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    /// Makes a GET request
    ///
    /// # Arguments
    ///
    /// * `path` - Request path
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails
    pub async fn get(&self, path: &str) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.config.base_url, path);
        let mut request = self.client.get(&url);

        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {api_key}"));
        }

        request.send().await
    }

    /// Makes a POST request
    ///
    /// # Arguments
    ///
    /// * `path` - Request path
    /// * `body` - Request body
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails
    pub async fn post<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.config.base_url, path);
        let mut request = self.client.post(&url).json(body);

        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {api_key}"));
        }

        request.send().await
    }

    /// Makes a PUT request
    ///
    /// # Arguments
    ///
    /// * `path` - Request path
    /// * `body` - Request body
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails
    pub async fn put<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.config.base_url, path);
        let mut request = self.client.put(&url).json(body);

        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {api_key}"));
        }

        request.send().await
    }

    /// Makes a DELETE request
    ///
    /// # Arguments
    ///
    /// * `path` - Request path
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails
    pub async fn delete(&self, path: &str) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.config.base_url, path);
        let mut request = self.client.delete(&url);

        if let Some(api_key) = &self.config.api_key {
            request = request.header("Authorization", format!("Bearer {api_key}"));
        }

        request.send().await
    }
}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    /// Response data
    pub data: T,
    /// Response status
    pub status: String,
    /// Response message
    pub message: Option<String>,
}

/// Webhook event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    /// Event type
    pub event_type: String,
    /// Event payload
    pub payload: serde_json::Value,
    /// Event timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Event ID
    pub id: uuid::Uuid,
}

impl WebhookEvent {
    /// Creates a new webhook event
    #[must_use]
    pub fn new(event_type: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            event_type: event_type.into(),
            payload,
            timestamp: chrono::Utc::now(),
            id: uuid::Uuid::new_v4(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_event_creation() {
        let event = WebhookEvent::new(
            "test.event",
            serde_json::json!({"key": "value"}),
        );

        assert_eq!(event.event_type, "test.event");
    }

    #[test]
    fn test_http_client_config() {
        let config = HttpClientConfig {
            timeout: Duration::from_secs(10),
            base_url: "https://api.example.com".to_string(),
            api_key: Some("key".to_string()),
        };

        assert_eq!(config.base_url, "https://api.example.com");
    }
}
