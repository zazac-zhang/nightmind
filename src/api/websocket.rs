// ============================================================
// WebSocket Handler Module
// ============================================================
//! WebSocket connection handling for real-time AI chat.
//!
//! This module provides the WebSocket handler for managing
//! real-time communication with the AI learning companion.

use axum::{
    extract::{State, ws::{WebSocket, Message}},
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::api::handlers::AppState;
use crate::error::{NightMindError, Result};
use crate::core::session::state::SessionStateMachine;
use crate::core::agent::NightMindAgent;
use crate::core::content::transformer::{VoiceFriendlyTransformer, PatternDetector};

// Re-export WebSocket types for convenience
pub use crate::api::dto::websocket::{
    WsMessage, TextInputData, TextResponseData, StateUpdateData,
    SessionControlData, SessionControlAction, ErrorData,
    AckData, AckType, KnowledgeData, HeartbeatData,
    SessionData, SessionEndData, ContentTransformData,
};

/// WebSocket query parameters for authentication
#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    /// User ID (required for Phase 1)
    pub user_id: Option<Uuid>,
    /// Session ID (optional, will be created if missing)
    pub session_id: Option<Uuid>,
    /// JWT token (Phase 2+)
    pub token: Option<String>,
}

/// WebSocket session context
pub struct WebSocketSession {
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub state_machine: SessionStateMachine,
    pub agent: Arc<NightMindAgent>,
}

impl WebSocketSession {
    /// Creates a new WebSocket session with a real agent
    pub fn new(session_id: Uuid, user_id: Uuid, agent: Arc<NightMindAgent>) -> Self {
        Self {
            session_id,
            user_id,
            state_machine: SessionStateMachine::new(),
            agent,
        }
    }

    /// Handles an incoming message
    pub async fn handle_message(&mut self, msg: WsMessage) -> Result<Vec<WsMessage>> {
        match msg {
            WsMessage::TextInput { data } => {
                self.handle_text_input(data).await
            }
            WsMessage::Heartbeat { .. } => {
                Ok(vec![WsMessage::heartbeat(self.session_id)])
            }
            WsMessage::SessionControl { data } => {
                self.handle_session_control(data).await
            }
            WsMessage::ContentTransform { .. } => {
                // Content transform status is server-sent only, ignore from client
                Ok(vec![])
            }
            _ => Ok(vec![]),
        }
    }

    /// Handles text input from user
    async fn handle_text_input(&mut self, data: TextInputData) -> Result<Vec<WsMessage>> {
        let message_id = data.message_id.unwrap_or_else(Uuid::new_v4);

        let needs_transform = self.agent.config().enable_content_transform;

        // Get AI response (with or without transformation)
        let response_text = if needs_transform {
            self.agent.prompt_with_transform(&data.text).await
                .map_err(|e| NightMindError::AiService(e.to_string()))?
        } else {
            self.agent.prompt(&data.text).await
                .map_err(|e| NightMindError::AiService(e.to_string()))?
        };

        // Validate the response to check if transformation occurred
        let validation = VoiceFriendlyTransformer::validate_voice_friendly(&response_text);
        let reading_time = validation.reading_time_seconds;

        // Create response message
        let response = WsMessage::text_response(
            response_text,
            self.session_id,
            message_id,
        );

        let mut messages = vec![response];

        if needs_transform {
            let had_patterns = !PatternDetector::detect_patterns(&data.text).is_empty();

            messages.push(WsMessage::content_transform(
                message_id,
                self.session_id,
                had_patterns,
                Some(validation.score as f32 / 100.0),
                reading_time,
            ));
        }

        if self.should_transition_state() {
            if let Ok(transition) = self.state_machine.advance() {
                let state_name = format!("{:?}", transition.to);
                let state_update = WsMessage::state_update_with_reason(
                    &state_name,
                    self.session_id,
                    "Natural conversation flow",
                );
                messages.push(state_update);
            }
        }

        Ok(messages)
    }

    /// Handles session control commands
    async fn handle_session_control(&mut self, data: SessionControlData) -> Result<Vec<WsMessage>> {
        use crate::api::dto::websocket::SessionControlAction::*;

        let messages = match data.action {
            Pause => {
                vec![WsMessage::state_update_with_reason(
                    "paused",
                    self.session_id,
                    data.reason.unwrap_or_default(),
                )]
            }
            Resume => {
                let state_name = format!("{:?}", self.state_machine.current());
                vec![WsMessage::state_update_with_reason(
                    &state_name,
                    self.session_id,
                    "Resumed",
                )]
            }
            End => {
                let _ = self.state_machine.transition_to(
                    crate::repository::models::session::SessionState::Closing
                );
                vec![
                    WsMessage::state_update_with_reason(
                        "closing",
                        self.session_id,
                        data.reason.unwrap_or_default(),
                    ),
                    WsMessage::session_ended(self.session_id, "User requested"),
                ]
            }
            Advance => {
                if let Ok(transition) = self.state_machine.advance() {
                    let state_name = format!("{:?}", transition.to);
                    vec![WsMessage::state_update_with_reason(
                        &state_name,
                        self.session_id,
                        "Advanced by user",
                    )]
                } else {
                    vec![]
                }
            }
        };

        Ok(messages)
    }

    /// Checks if the session should transition to the next state
    pub fn should_transition_state(&self) -> bool {
        // TODO: Implement logic to determine if state should transition
        false
    }
}

/// WebSocket connection upgrade handler
pub async fn websocket_handler(
    State(state): State<AppState>,
    ws: axum::extract::WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handles a WebSocket connection
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Extract user info from query params (Phase 1: simple; Phase 2: JWT token)
    let (session_id, user_id) = {
        // For Phase 1, generate IDs. In Phase 2, extract from JWT token.
        (Uuid::new_v4(), Uuid::new_v4())
    };

    let agent = state.agent.clone();
    let mut ws_session = WebSocketSession::new(session_id, user_id, agent);

    tracing::info!(
        "WebSocket connected: session={} user={}",
        session_id, user_id
    );

    // Send session started message
    let session_started = WsMessage::session_started(session_id, user_id, "New Session");
    if let Ok(json) = session_started.to_json() {
        let _ = sender.send(Message::Text(json.into())).await;
    }

    // Message handling loop
    while let Some(result) = receiver.next().await {
        match result {
            Ok(msg) => {
                match msg {
                    Message::Text(text) => {
                        match WsMessage::from_json(&text) {
                            Ok(ws_msg) => {
                                match ws_session.handle_message(ws_msg).await {
                                    Ok(responses) => {
                                        for response in responses {
                                            if let Ok(json) = response.to_json() {
                                                let _ = sender.send(Message::Text(json.into())).await;
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Error handling message: {}", e);
                                        let error_msg = WsMessage::error(
                                            &e.to_string(),
                                            "internal_error",
                                        );
                                        if let Ok(json) = error_msg.to_json() {
                                            let _ = sender.send(Message::Text(json.into())).await;
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to parse message: {}", e);
                                let error_msg = WsMessage::error(
                                    "Invalid message format",
                                    "invalid_format",
                                );
                                if let Ok(json) = error_msg.to_json() {
                                    let _ = sender.send(Message::Text(json.into())).await;
                                }
                            }
                        }
                    }
                    Message::Close(_) => {
                        break;
                    }
                    _ => {
                        // Ignore other message types (binary, ping/pong frames)
                    }
                }
            }
            Err(e) => {
                tracing::error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Session ended
    let session_ended = WsMessage::session_ended(session_id, "Connection closed");
    if let Ok(json) = session_ended.to_json() {
        let _ = sender.send(Message::Text(json.into())).await;
    }

    tracing::info!(
        "WebSocket disconnected: session={} user={}",
        session_id, user_id
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_query_parsing() {
        let query = WebSocketQuery {
            user_id: Some(Uuid::new_v4()),
            session_id: None,
            token: None,
        };
        assert!(query.user_id.is_some());
    }

    #[test]
    fn test_heartbeat_message() {
        let msg = WsMessage::heartbeat(Uuid::new_v4());
        assert!(msg.is_heartbeat());
    }
}
