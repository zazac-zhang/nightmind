// ============================================================
// Agent System Module
// ============================================================
//! AI agent system for learning companion functionality.
//!
//! This module provides the agent builder, prompts, and tools
//! for the NightMind AI learning companion.

pub mod builder;
pub mod prompts;
pub mod tools;

// Re-export Rig completion trait for convenience
pub use rig::completion::Prompt;

// Re-export common types
pub use builder::{
    AgentBuilder, AgentConfig, AgentManager, NightMindAgent,
    Role,
};
pub use prompts::{
    PersonalityConfig, PromptCategory, PromptManager,
    SYSTEM_PROMPT,
    WARMUP_PROMPT, DEEPDIVE_PROMPT, REVIEW_PROMPT, SEED_PROMPT, CLOSING_PROMPT,
};
