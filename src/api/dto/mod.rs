// ============================================================
// API DTO Module
// ============================================================
//! Data Transfer Objects for API requests and responses.
//!
//! This module defines all request and response structures
//! for the REST API and WebSocket communication.

mod requests;
mod responses;
pub mod websocket;

pub use requests::*;
pub use responses::*;
