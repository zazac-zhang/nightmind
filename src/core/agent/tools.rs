// ============================================================
// Agent Tools
// ============================================================
//! Tool definitions for AI agent capabilities.
//!
//! This module provides tool abstractions for extending agent
/// functionality with external capabilities.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Trait defining agent tool capabilities
#[async_trait::async_trait]
pub trait Tool: Send + Sync {
    /// Returns the tool's name
    fn name(&self) -> &str;

    /// Returns the tool's description
    fn description(&self) -> &str;

    /// Executes the tool with the given parameters
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult, ToolError>;
}

/// Parameters for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParams {
    /// Tool name
    pub tool_name: String,
    /// Parameters as JSON value
    pub params: serde_json::Value,
}

impl ToolParams {
    /// Creates new tool parameters
    #[must_use]
    pub fn new(tool_name: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            tool_name: tool_name.into(),
            params,
        }
    }
}

/// Result from tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Tool name that generated the result
    pub tool_name: String,
    /// Execution result data
    pub data: serde_json::Value,
    /// Whether execution was successful
    pub success: bool,
    /// Error message if execution failed
    pub error: Option<String>,
}

impl ToolResult {
    /// Creates a successful tool result
    #[must_use]
    pub fn success(tool_name: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            tool_name: tool_name.into(),
            data,
            success: true,
            error: None,
        }
    }

    /// Creates a failed tool result
    #[must_use]
    pub fn failure(tool_name: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            data: serde_json::Value::Null,
            success: false,
            error: Some(error.into()),
        }
    }
}

/// Errors that can occur during tool execution
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    /// Tool not found
    #[error("Tool not found: {0}")]
    NotFound(String),

    /// Invalid parameters
    #[error("Invalid parameters: {0}")]
    InvalidParams(String),

    /// Execution error
    #[error("Execution error: {0}")]
    Execution(String),
}

/// Knowledge retrieval tool
pub struct KnowledgeTool {
    /// Tool ID
    pub id: Uuid,
}

impl KnowledgeTool {
    /// Creates a new knowledge tool
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
        }
    }
}

impl Default for KnowledgeTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Tool for KnowledgeTool {
    fn name(&self) -> &str {
        "knowledge_retrieval"
    }

    fn description(&self) -> &str {
        "Retrieves relevant knowledge from the user's knowledge base"
    }

    async fn execute(&self, _params: &ToolParams) -> Result<ToolResult, ToolError> {
        // Placeholder implementation
        Ok(ToolResult::success(
            self.name(),
            serde_json::json!({ "results": [] }),
        ))
    }
}

/// Memory consolidation tool
pub struct MemoryTool {
    /// Tool ID
    pub id: Uuid,
}

impl MemoryTool {
    /// Creates a new memory tool
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
        }
    }
}

impl Default for MemoryTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Tool for MemoryTool {
    fn name(&self) -> &str {
        "memory_consolidation"
    }

    fn description(&self) -> &str {
        "Consolidates and organizes learning material for better retention"
    }

    async fn execute(&self, _params: &ToolParams) -> Result<ToolResult, ToolError> {
        // Placeholder implementation
        Ok(ToolResult::success(
            self.name(),
            serde_json::json!({ "consolidated": true }),
        ))
    }
}

/// Tool registry for managing available tools
pub struct ToolRegistry {
    /// Registered tool names
    tool_names: Vec<String>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    /// Creates a new tool registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            tool_names: vec![
                KnowledgeTool::new().name().to_string(),
                MemoryTool::new().name().to_string(),
            ],
        }
    }

    /// Lists all registered tool names
    #[must_use]
    pub fn list(&self) -> Vec<String> {
        self.tool_names.clone()
    }

    /// Checks if a tool exists by name
    #[must_use]
    pub fn has_tool(&self, name: &str) -> bool {
        self.tool_names.iter().any(|n| n == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry() {
        let registry = ToolRegistry::new();
        let tools = registry.list();

        assert!(tools.contains(&"knowledge_retrieval".to_string()));
        assert!(tools.contains(&"memory_consolidation".to_string()));
    }

    #[test]
    fn test_tool_registry_has_tool() {
        let registry = ToolRegistry::new();

        assert!(registry.has_tool("knowledge_retrieval"));
        assert!(!registry.has_tool("nonexistent"));
    }

    #[tokio::test]
    async fn test_knowledge_tool_execution() {
        let tool = KnowledgeTool::new();
        let params = ToolParams::new("knowledge_retrieval", serde_json::json!({}));

        let result = tool.execute(&params).await.unwrap();
        assert!(result.success);
    }
}
