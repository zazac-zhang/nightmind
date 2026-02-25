// ============================================================
// NightMind Library
// ============================================================

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod api;
pub mod config;
pub mod core;
pub mod error;
pub mod repository;
pub mod services;

// Re-exports
pub use error::{NightMindError, Result};
