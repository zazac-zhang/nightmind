// ============================================================
// WebSocket Handler Module
// ============================================================
//! WebSocket connection handling for real-time AI chat.
//!
//! This module provides the WebSocket handler for managing
//! real-time communication with the AI learning companion.

use axum::{
    extract::{
        State,
        ws::{WebSocket, Message},
        WebSocketUpgrade,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use crate::api::handlers::AppState;
use crate::error::{NightMindError, Result};
use crate::core::session::state::SessionStateMachine;
use crate::core::agent::{NightMindAgent, AgentBuilder};

// Re-export WebSocket types for convenience
pub use crate::api::dto::websocket::{
    WsMessage, TextInputData, TextResponseData, StateUpdateData,
    SessionControlData, SessionControlAction, ErrorData,
    AckData, AckType, KnowledgeData, HeartbeatData,
    SessionData, SessionEndData,
};

/// WebSocket session context
pub struct WebSocketSession {
    /// Session ID
    pub session_id: Uuid,
    /// User ID
    pub user_id: Uuid,
    /// Current state machine
    pub state_machine: SessionStateMachine,
    /// AI agent
    pub agent: NightMindAgent,
}

impl WebSocketSession {
    /// Creates a new WebSocket session
    pub fn new(session_id: Uuid, user_id: Uuid) -> Self {
        let state_machine = SessionStateMachine::new();
        let agent = AgentBuilder::default()
            .build()
            .unwrap_or_else(|_| AgentBuilder::default().build_simple());

        Self {
            session_id,
            user_id,
            state_machine,
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
            _ => Ok(vec![]),
        }
    }

    /// Handles text input from user
    async fn handle_text_input(&mut self, data: TextInputData) -> Result<Vec<WsMessage>> {
        // Get current state as string
        let current_state = format!("{:?}", self.state_machine.current());

        // Generate AI response using agent
        let response_text = self.agent.prompt(&data.text).await
            .map_err(|e| NightMindError::AiService(e.to_string()))?;

        // Create response message
        let message_id = Uuid::new_v4();
        let response = WsMessage::text_response(
            response_text,
            self.session_id,
            message_id,
        );

        // Check if we should transition state
        if self.should_transition_state() {
            if let Ok(transition) = self.state_machine.advance() {
                let state_name = format!("{:?}", transition.to);
                let state_update = WsMessage::state_update_with_reason(
                    &state_name,
                    self.session_id,
                    "Natural conversation flow",
                );
                return Ok(vec![response, state_update]);
            }
        }

        Ok(vec![response])
    }

    /// Handles session control commands
    async fn handle_session_control(&mut self, data: SessionControlData) -> Result<Vec<WsMessage>> {
        use crate::api::dto::websocket::SessionControlAction::*;

        let messages = match data.action {
            Pause => {
                // Pause means transition to a paused state - for now just return current state
                vec![WsMessage::state_update_with_reason(
                    "paused",
                    self.session_id,
                    data.reason.unwrap_or_default(),
                )]
            }
            Resume => {
                // Resume returns to the actual state
                let state_name = format!("{:?}", self.state_machine.current());
                vec![WsMessage::state_update_with_reason(
                    &state_name,
                    self.session_id,
                    "Resumed",
                )]
            }
            End => {
                // End the session by transitioning to Closing
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
                // Advance to next state
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
    fn should_transition_state(&self) -> bool {
        // TODO: Implement logic to determine if state should transition
        false
    }
}

/// WebSocket connection handler
pub async fn websocket_handler(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

/// Handles a WebSocket connection
async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // TODO: Extract and validate session token from query params
    // For now, create a new session
    let session_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let mut ws_session = WebSocketSession::new(session_id, user_id);

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
                                // Handle the message
                                match ws_session.handle_message(ws_msg).await {
                                    Ok(responses) => {
                                        // Send all response messages
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
                        // Ignore other message types
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_session_creation() {
        let session_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let session = WebSocketSession::new(session_id, user_id);

        assert_eq!(session.session_id, session_id);
        assert_eq!(session.user_id, user_id);
        assert_eq!(format!("{:?}", session.state_machine.current()), "Warmup");
    }

    #[test]
    fn test_heartbeat_message() {
        let msg = WsMessage::heartbeat(Uuid::new_v4());
        assert!(msg.is_heartbeat());
    }
}
