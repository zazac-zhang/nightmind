// ============================================================
// WebSocket Message DTOs
// ============================================================
//! WebSocket message data transfer objects.
//!
//! This module defines all message structures for WebSocket communication.

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// All WebSocket message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum WsMessage {
    /// Text input from user
    TextInput {
        #[serde(flatten)]
        data: TextInputData,
    },
    /// Text response from AI
    TextResponse {
        #[serde(flatten)]
        data: TextResponseData,
    },
    /// Session state update
    StateUpdate {
        #[serde(flatten)]
        data: StateUpdateData,
    },
    /// Session started
    SessionStarted {
        #[serde(flatten)]
        data: SessionData,
    },
    /// Session ended
    SessionEnded {
        #[serde(flatten)]
        data: SessionEndData,
    },
    /// Error occurred
    Error {
        #[serde(flatten)]
        data: ErrorData,
    },
    /// Heartbeat
    Heartbeat {
        #[serde(flatten)]
        data: HeartbeatData,
    },
    /// Knowledge point created
    KnowledgeCreated {
        #[serde(flatten)]
        data: KnowledgeData,
    },
    /// Acknowledgment
    Ack {
        #[serde(flatten)]
        data: AckData,
    },
    /// Session pause/resume
    SessionControl {
        #[serde(flatten)]
        data: SessionControlData,
    },
}

/// User text input data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextInputData {
    /// User's message text
    pub text: String,
    /// Message ID (optional)
    pub message_id: Option<Uuid>,
    /// Session ID
    pub session_id: Uuid,
}

/// AI text response data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextResponseData {
    /// Response text
    pub text: String,
    /// Message ID
    pub message_id: Uuid,
    /// Session ID
    pub session_id: Uuid,
    /// Whether this is a partial response (streaming)
    pub is_partial: bool,
    /// Sequence number for partial responses
    pub sequence: Option<u32>,
}

/// State update data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateUpdateData {
    /// New state
    pub state: String,
    /// Session ID
    pub session_id: Uuid,
    /// Reason for state change
    pub reason: Option<String>,
}

/// Session data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionData {
    /// Session ID
    pub id: Uuid,
    /// User ID
    pub user_id: Uuid,
    /// Session title
    pub title: String,
    /// Current state
    pub state: String,
    /// Started at
    pub started_at: DateTime<Utc>,
}

/// Session end data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionEndData {
    /// Session ID
    pub session_id: Uuid,
    /// End reason
    pub reason: String,
    /// Ended at
    pub ended_at: DateTime<Utc>,
    /// Session summary
    pub summary: Option<String>,
}

/// Error data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorData {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Request ID if available
    pub request_id: Option<String>,
}

/// Heartbeat data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatData {
    /// Timestamp
    pub timestamp: i64,
    /// Session ID
    pub session_id: Uuid,
}

/// Knowledge point data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeData {
    /// Knowledge ID
    pub id: Uuid,
    /// Title
    pub title: String,
    /// Category
    pub category: Option<String>,
    /// Created at
    pub created_at: DateTime<Utc>,
}

/// Acknowledgment data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AckData {
    /// Acknowledged message ID
    pub message_id: Uuid,
    /// Ack type
    pub ack_type: AckType,
    /// Timestamp
    pub timestamp: i64,
}

/// Acknowledgment types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AckType {
    /// Message received
    Received,
    /// Message processed
    Processed,
    /// Message delivered
    Delivered,
}

/// Session control actions
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionControlAction {
    /// Pause the session
    Pause,
    /// Resume the session
    Resume,
    /// End the session
    End,
    /// Skip to next state
    Advance,
}

/// Session control data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionControlData {
    /// Action to perform
    pub action: SessionControlAction,
    /// Session ID
    pub session_id: Uuid,
    /// Reason
    pub reason: Option<String>,
}

impl WsMessage {
    /// Creates a new text input message
    #[must_use]
    pub fn text_input(text: impl Into<String>, session_id: Uuid) -> Self {
        Self::TextInput {
            data: TextInputData {
                text: text.into(),
                message_id: Some(Uuid::new_v4()),
                session_id,
            },
        }
    }

    /// Creates a new text response message
    #[must_use]
    pub fn text_response(text: impl Into<String>, session_id: Uuid, message_id: Uuid) -> Self {
        Self::TextResponse {
            data: TextResponseData {
                text: text.into(),
                message_id,
                session_id,
                is_partial: false,
                sequence: None,
            },
        }
    }

    /// Creates a partial text response (for streaming)
    #[must_use]
    pub fn partial_response(
        text: impl Into<String>,
        session_id: Uuid,
        message_id: Uuid,
        sequence: u32,
    ) -> Self {
        Self::TextResponse {
            data: TextResponseData {
                text: text.into(),
                message_id,
                session_id,
                is_partial: true,
                sequence: Some(sequence),
            },
        }
    }

    /// Creates a state update message
    #[must_use]
    pub fn state_update(state: impl Into<String>, session_id: Uuid) -> Self {
        Self::StateUpdate {
            data: StateUpdateData {
                state: state.into(),
                session_id,
                reason: None,
            },
        }
    }

    /// Creates a state update with reason
    #[must_use]
    pub fn state_update_with_reason(
        state: impl Into<String>,
        session_id: Uuid,
        reason: impl Into<String>,
    ) -> Self {
        Self::StateUpdate {
            data: StateUpdateData {
                state: state.into(),
                session_id,
                reason: Some(reason.into()),
            },
        }
    }

    /// Creates a session started message
    #[must_use]
    pub fn session_started(session_id: Uuid, user_id: Uuid, title: impl Into<String>) -> Self {
        Self::SessionStarted {
            data: SessionData {
                id: session_id,
                user_id,
                title: title.into(),
                state: "warmup".to_string(),
                started_at: Utc::now(),
            },
        }
    }

    /// Creates a session ended message
    #[must_use]
    pub fn session_ended(session_id: Uuid, reason: impl Into<String>) -> Self {
        Self::SessionEnded {
            data: SessionEndData {
                session_id,
                reason: reason.into(),
                ended_at: Utc::now(),
                summary: None,
            },
        }
    }

    /// Creates an error message
    #[must_use]
    pub fn error(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self::Error {
            data: ErrorData {
                code: code.into(),
                message: message.into(),
                request_id: None,
            },
        }
    }

    /// Creates a heartbeat message
    #[must_use]
    pub fn heartbeat(session_id: Uuid) -> Self {
        Self::Heartbeat {
            data: HeartbeatData {
                timestamp: Utc::now().timestamp(),
                session_id,
            },
        }
    }

    /// Creates an acknowledgment message
    #[must_use]
    pub fn ack(message_id: Uuid, ack_type: AckType) -> Self {
        Self::Ack {
            data: AckData {
                message_id,
                ack_type,
                timestamp: Utc::now().timestamp(),
            },
        }
    }

    /// Creates a knowledge created message
    #[must_use]
    pub fn knowledge_created(knowledge_id: Uuid, title: impl Into<String>) -> Self {
        Self::KnowledgeCreated {
            data: KnowledgeData {
                id: knowledge_id,
                title: title.into(),
                category: None,
                created_at: Utc::now(),
            },
        }
    }

    /// Creates a session control message
    #[must_use]
    pub fn session_control(action: SessionControlAction, session_id: Uuid) -> Self {
        Self::SessionControl {
            data: SessionControlData {
                action,
                session_id,
                reason: None,
            },
        }
    }

    /// Checks if message is a heartbeat
    #[must_use]
    pub fn is_heartbeat(&self) -> bool {
        matches!(self, WsMessage::Heartbeat { .. })
    }

    /// Gets the session ID for this message
    #[must_use]
    pub fn session_id(&self) -> Option<Uuid> {
        match self {
            Self::TextInput { data } => Some(data.session_id),
            Self::TextResponse { data } => Some(data.session_id),
            Self::StateUpdate { data } => Some(data.session_id),
            Self::SessionStarted { data } => Some(data.id),
            Self::SessionEnded { data } => Some(data.session_id),
            Self::Heartbeat { data } => Some(data.session_id),
            Self::SessionControl { data } => Some(data.session_id),
            Self::KnowledgeCreated { .. } => None,
            Self::Error { .. } => None,
            Self::Ack { .. } => None,
        }
    }

    /// Serializes message to JSON
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserializes message from JSON
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_input_message() {
        let session_id = Uuid::new_v4();
        let msg = WsMessage::text_input("Hello", session_id);
        assert!(matches!(msg, WsMessage::TextInput { .. }));
        assert_eq!(msg.session_id(), Some(session_id));
    }

    #[test]
    fn test_state_update_message() {
        let session_id = Uuid::new_v4();
        let msg = WsMessage::state_update("deep_dive", session_id);
        assert!(matches!(msg, WsMessage::StateUpdate { .. }));
    }

    #[test]
    fn test_heartbeat_message() {
        let msg = WsMessage::heartbeat(Uuid::new_v4());
        assert!(msg.is_heartbeat());
    }

    #[test]
    fn test_message_serialization() {
        let msg = WsMessage::text_input("Test", Uuid::new_v4());
        let json = msg.to_json().unwrap();
        assert!(json.contains(r#""type":"textInput""#));

        let decoded: WsMessage = WsMessage::from_json(&json).unwrap();
        assert!(matches!(decoded, WsMessage::TextInput { .. }));
    }

    #[test]
    fn test_partial_response() {
        let msg = WsMessage::partial_response("Hello ", Uuid::new_v4(), Uuid::new_v4(), 1);
        match msg {
            WsMessage::TextResponse { data } => {
                assert!(data.is_partial);
                assert_eq!(data.sequence, Some(1));
            }
            _ => panic!("Expected TextResponse"),
        }
    }
}
