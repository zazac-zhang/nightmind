// ============================================================
// Knowledge Retrieval Tool for Agents
// ============================================================
//! Agent tool for retrieving knowledge during conversations.
//!
//! This module provides a Rig-compatible tool that agents can use
//! to retrieve relevant knowledge from the user's learning history.

use crate::core::memory::{KnowledgeRetriever, RetrievalContext};
use crate::repository::models::SessionState;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Knowledge retrieval tool for agents
pub struct KnowledgeRetrieverTool {
    /// Knowledge retriever instance
    retriever: Arc<KnowledgeRetriever>,
}

impl KnowledgeRetrieverTool {
    /// Creates a new knowledge retriever tool
    #[must_use]
    pub fn new(retriever: Arc<KnowledgeRetriever>) -> Self {
        Self { retriever }
    }
}

#[async_trait::async_trait]
impl Tool for KnowledgeRetrieverTool {
    const NAME: &'static str = "knowledge_retrieval";

    fn description(&self) -> &'static str {
        r#"
Retrieves relevant knowledge from the user's learning history based on the conversation context.

Use this tool when you need to:
- Recall what the user has previously learned
- Provide personalized context based on their knowledge base
- Reference concepts they've studied in the past
- Review items that are due for spaced repetition

The tool automatically adapts to:
- Current cognitive load (simplifies results during high fatigue)
- Session state (Warmup, DeepDive, Review, Seed, Closing)
- Review schedule (prioritizes due items)

Returns a formatted context with relevant knowledge points.
        "#
    }

    /// Examples for the agent
    fn examples(&self) -> Vec<String> {
        vec![
            r#"
User: "What did I learn about Rust yesterday?"
Agent Tool Call: knowledge_retrieval(message="Rust learning", cognitive_load=0.5)
Result: "【相关知识库】
1. Rust ownership system
2. Borrow checker rules
..."
            "#.to_string(),
            r#"
User: "I'm feeling tired, can we review something simple?"
Agent Tool Call: knowledge_retrieval(message="simple review", cognitive_load=0.8)
Result: "【相关知识库】
1. Basic Rust syntax (简化版)
..."
            "#.to_string(),
        ]
    }

    /// Execute the tool
    async fn execute(&self, input: &ToolInput) -> Result<ToolOutput, ToolError> {
        // Parse input
        let params = RetrievalParams::parse(input)?;

        // Determine session state from input
        let session_state = params
            .session_state
            .unwrap_or(SessionState::Warmup)
            .clone();

        // Retrieve knowledge
        let context: RetrievalContext = self
            .retriever
            .retrieve_for_context(
                params.user_id,
                &params.message,
                params.cognitive_load,
                session_state,
                params.limit.unwrap_or(5),
            )
            .await
            .map_err(|e| ToolError::Execution(format!("Retrieval failed: {}", e)))?;

        // Format output
        let output = RetrievalOutput {
            context: context.context_text,
            knowledge_count: context.total_count,
            cognitive_load: params.cognitive_load,
            retrieved_at: context.retrieved_at,
        };

        Ok(ToolOutput {
            tool_name: Self::NAME.to_string(),
            data: serde_json::to_value(output)
                .map_err(|e| ToolError::Serialization(format!("Failed to serialize output: {}", e)))?,
            success: true,
            error: None,
        })
    }
}

/// Retrieval parameters
#[derive(Debug, Clone, Deserialize)]
struct RetrievalParams {
    /// User ID
    user_id: Uuid,

    /// Current message/query
    message: String,

    /// Cognitive load (0.0-1.0)
    cognitive_load: f32,

    /// Session state (optional)
    session_state: Option<SessionState>,

    /// Maximum results (optional)
    limit: Option<u64>,
}

impl RetrievalParams {
    /// Parse parameters from tool input
    fn parse(input: &ToolInput) -> Result<Self, ToolError> {
        let user_id = input
            .params
            .get("user_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("missing user_id"))?
            .parse::<Uuid>()
            .map_err(|_| ToolError::InvalidInput("invalid user_id format"))?;

        let message = input
            .params
            .get("message")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidInput("missing message"))?
            .to_string();

        let cognitive_load = input
            .params
            .get("cognitive_load")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ToolError::InvalidInput("missing cognitive_load"))? as f32;

        let session_state = input
            .params
            .get("session_state")
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str::<SessionState>(s).ok());

        let limit = input.params.get("limit").and_then(|v| v.as_u64());

        Ok(Self {
            user_id,
            message,
            cognitive_load,
            session_state,
            limit,
        })
    }
}

/// Retrieval output
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RetrievalOutput {
    /// Formatted context text
    context: String,
    /// Number of knowledge points retrieved
    knowledge_count: usize,
    /// Cognitive load used for retrieval
    cognitive_load: f32,
    /// When retrieval was performed
    retrieved_at: chrono::DateTime<chrono::Utc>,
}

/// Tool input
#[derive(Debug, Clone)]
pub struct ToolInput {
    /// Parameters
    pub params: serde_json::Value,
}

/// Tool output
#[derive(Debug, Clone)]
pub struct ToolOutput {
    /// Tool name
    pub tool_name: String,
    /// Output data
    pub data: serde_json::Value,
    /// Success flag
    pub success: bool,
    /// Error message (if any)
    pub error: Option<String>,
}

/// Tool errors
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(&'static str),

    /// Execution error
    #[error("Execution error: {0}")]
    Execution(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retrieval_params_missing_user_id() {
        let input = ToolInput {
            params: serde_json::json!({
                "message": "test",
                "cognitive_load": 0.5,
            }),
        };

        let result = RetrievalParams::parse(&input);
        assert!(result.is_err());
    }

    #[test]
    fn test_retrieval_output_serialization() {
        let output = RetrievalOutput {
            context: "Test context".to_string(),
            knowledge_count: 3,
            cognitive_load: 0.5,
            retrieved_at: Utc::now(),
        };

        let json = serde_json::to_value(&output);
        assert!(json.is_ok());

        let value = json.unwrap();
        assert_eq!(value["context"], "Test context");
        assert_eq!(value["knowledge_count"], 3);
    }
}
