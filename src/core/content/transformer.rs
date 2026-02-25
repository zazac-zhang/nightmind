// ============================================================
// Content Transformer
// ============================================================
//! Content transformation and processing.
//!
//! This module provides utilities for transforming and processing
/// various types of content within the system.

use serde::{Deserialize, Serialize};

/// Content type enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    /// Plain text content
    Text,
    /// Markdown formatted content
    Markdown,
    /// HTML content
    Html,
    /// JSON data
    Json,
    /// Structured data
    Structured,
}

/// Content transformation error
#[derive(Debug, thiserror::Error)]
pub enum TransformError {
    /// Unsupported content type
    #[error("Unsupported content type: {0}")]
    UnsupportedType(String),

    /// Parse error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Transformed content result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformedContent {
    /// The transformed content
    pub content: String,
    /// Content type after transformation
    pub content_type: ContentType,
    /// Metadata about the transformation
    pub metadata: std::collections::HashMap<String, String>,
}

impl TransformedContent {
    /// Creates new transformed content
    #[must_use]
    pub fn new(content: impl Into<String>, content_type: ContentType) -> Self {
        Self {
            content: content.into(),
            content_type,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Adds metadata to the transformed content
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Content transformer trait
pub trait ContentTransformer: Send + Sync {
    /// Transforms content from one format to another
    ///
    /// # Arguments
    ///
    /// * `content` - Input content
    /// * `from_type` - Source content type
    /// * `to_type` - Target content type
    ///
    /// # Returns
    ///
    /// Transformed content or an error
    fn transform(
        &self,
        content: &str,
        from_type: ContentType,
        to_type: ContentType,
    ) -> Result<TransformedContent, TransformError>;
}

/// Markdown transformer for converting between text and markdown
pub struct MarkdownTransformer;

impl ContentTransformer for MarkdownTransformer {
    fn transform(
        &self,
        content: &str,
        from_type: ContentType,
        to_type: ContentType,
    ) -> Result<TransformedContent, TransformError> {
        match (from_type, to_type) {
            (ContentType::Text, ContentType::Markdown) => {
                // Convert plain text to markdown (add formatting)
                let lines: Vec<String> = content
                    .lines()
                    .map(|line| format!("{}\n", line))
                    .collect();
                Ok(TransformedContent::new(lines.join(""), ContentType::Markdown))
            }
            (ContentType::Markdown, ContentType::Text) => {
                // Strip markdown formatting
                let text = content
                    .replace("**", "")
                    .replace("*", "")
                    .replace("#", "")
                    .replace("```", "");
                Ok(TransformedContent::new(text, ContentType::Text))
            }
            _ => Err(TransformError::UnsupportedType(format!(
                "{from_type:?} to {to_type:?}"
            ))),
        }
    }
}

/// JSON transformer for serializing/deserializing structured data
pub struct JsonTransformer;

impl ContentTransformer for JsonTransformer {
    fn transform(
        &self,
        content: &str,
        from_type: ContentType,
        to_type: ContentType,
    ) -> Result<TransformedContent, TransformError> {
        match (from_type, to_type) {
            (ContentType::Json, ContentType::Text) => {
                // Pretty print JSON
                let value: serde_json::Value =
                    serde_json::from_str(content).map_err(|e| TransformError::Parse(e.to_string()))?;
                let pretty = serde_json::to_string_pretty(&value)
                    .map_err(|e| TransformError::Serialization(e.to_string()))?;
                Ok(TransformedContent::new(pretty, ContentType::Text))
            }
            (ContentType::Structured, ContentType::Json) => {
                // Serialize structured data to JSON
                let json = serde_json::to_string_pretty(&content)
                    .map_err(|e| TransformError::Serialization(e.to_string()))?;
                Ok(TransformedContent::new(json, ContentType::Json))
            }
            _ => Err(TransformError::UnsupportedType(format!(
                "{from_type:?} to {to_type:?}"
            ))),
        }
    }
}

/// Content summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSummary {
    /// Word count
    pub word_count: usize,
    /// Character count
    pub char_count: usize,
    /// Line count
    pub line_count: usize,
    /// Paragraph count
    pub paragraph_count: usize,
    /// Detected language (optional)
    pub language: Option<String>,
}

/// Summarizes text content
#[must_use]
pub fn summarize_content(content: &str) -> ContentSummary {
    let char_count = content.chars().count();
    let lines: Vec<&str> = content.lines().collect();
    let line_count = lines.len();

    let word_count = content
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .count();

    let paragraph_count = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .count()
        .max(1);

    ContentSummary {
        word_count,
        char_count,
        line_count,
        paragraph_count,
        language: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_transformer() {
        let transformer = MarkdownTransformer;

        let result = transformer.transform("Hello **world**", ContentType::Markdown, ContentType::Text);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().content, "Hello world");
    }

    #[test]
    fn test_content_summary() {
        let content = "Hello world!\n\nThis is a test.";
        let summary = summarize_content(content);

        assert_eq!(summary.word_count, 6);
        assert!(summary.line_count > 0);
    }

    #[test]
    fn test_transformed_content_with_metadata() {
        let content = TransformedContent::new("test", ContentType::Text)
            .with_metadata("source", "test.txt")
            .with_metadata("length", "4");

        assert_eq!(content.metadata.get("source"), Some(&"test.txt".to_string()));
        assert_eq!(content.metadata.get("length"), Some(&"4".to_string()));
    }
}
