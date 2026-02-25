// ============================================================
// WebSocket Integration Tests
// ============================================================
//! WebSocket integration tests for NightMind.
//!
//! These tests verify real-time WebSocket communication.

use uuid::Uuid;

// ============================================================================
// WebSocket Message Serialization Tests
// ============================================================================

#[test]
fn test_websocket_message_serialization() {
    use nightmind::api::dto::websocket::*;

    let session_id = Uuid::new_v4();
    let msg = WsMessage::text_input("Test message", session_id);

    assert!(matches!(msg, WsMessage::TextInput { .. }));

    let json = msg.to_json().unwrap();
    assert!(json.contains(r#""type":"textInput""#));

    let decoded: WsMessage = WsMessage::from_json(&json).unwrap();
    assert!(matches!(decoded, WsMessage::TextInput { .. }));
}

#[test]
fn test_websocket_message_types() {
    use nightmind::api::dto::websocket::*;

    let session_id = Uuid::new_v4();

    // Test all message types can be created and serialized
    let messages = vec![
        WsMessage::text_input("Test", session_id),
        WsMessage::text_response("Response", session_id, Uuid::new_v4()),
        WsMessage::state_update("warmup", session_id),
        WsMessage::session_started(session_id, Uuid::new_v4(), "Test Session"),
        WsMessage::session_ended(session_id, "Test ended"),
        WsMessage::error("Test error", "test_code"),
        WsMessage::heartbeat(session_id),
        WsMessage::knowledge_created(Uuid::new_v4(), "Test Knowledge"),
        WsMessage::ack(Uuid::new_v4(), AckType::Received),
    ];

    for msg in messages {
        let json = msg.to_json();
        assert!(json.is_ok(), "Failed to serialize message");
    }
}

#[test]
fn test_websocket_partial_response() {
    use nightmind::api::dto::websocket::*;

    let session_id = Uuid::new_v4();
    let message_id = Uuid::new_v4();

    let partial = WsMessage::partial_response("Hello ", session_id, message_id, 1);

    match partial {
        WsMessage::TextResponse { data } => {
            assert!(data.is_partial);
            assert_eq!(data.sequence, Some(1));
        }
        _ => panic!("Expected TextResponse"),
    }
}

#[test]
fn test_session_control_actions() {
    use nightmind::api::dto::websocket::*;

    let session_id = Uuid::new_v4();

    let actions = vec![
        SessionControlAction::Pause,
        SessionControlAction::Resume,
        SessionControlAction::End,
        SessionControlAction::Advance,
    ];

    for action in actions {
        let msg = WsMessage::session_control(action, session_id);
        let json = msg.to_json();
        assert!(json.is_ok(), "Failed to serialize session control message");
    }
}

// ============================================================================
// WebSocket Session State Tests
// ============================================================================

#[test]
fn test_websocket_session_creation() {
    use nightmind::api::websocket::WebSocketSession;

    let session_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();

    let session = WebSocketSession::new(session_id, user_id);

    assert_eq!(session.session_id, session_id);
    assert_eq!(session.user_id, user_id);
    assert_eq!(format!("{:?}", session.state_machine.current()), "Warmup");
}

#[test]
fn test_websocket_session_should_transition() {
    use nightmind::api::websocket::WebSocketSession;

    let session = WebSocketSession::new(Uuid::new_v4(), Uuid::new_v4());

    // By default, should not transition
    assert!(!session.should_transition_state());
}
