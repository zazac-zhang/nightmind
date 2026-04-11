// ============================================================
// Content Transformer
// ============================================================
//! Content transformation for voice-friendly output.
//!
//! LLM-first approach: semantic understanding replaces rule-based
//! pattern detection. Code, formulas, and lists are converted to
//! metaphorical explanations suitable for speech.

use crate::core::agent::NightMindAgent;
use crate::error::Result;
use serde::{Deserialize, Serialize};

// ============================================================================
// Content Pattern Detection (simplified)
// ============================================================================

/// Semantic content type detected in text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SemanticType {
    /// Plain conversational text
    Plain,
    /// Code blocks or programming syntax
    Code,
    /// Mathematical formulas or equations
    Formula,
    /// Lists or bullet points
    List,
    /// Technical jargon
    Technical,
}

/// Detected content pattern
#[derive(Debug, Clone)]
pub struct ContentPattern {
    pub semantic_type: SemanticType,
    pub content: String,
    pub position: (usize, usize),
}

/// Content pattern detector
pub struct PatternDetector;

impl PatternDetector {
    /// Detects patterns in content that need transformation
    pub fn detect_patterns(content: &str) -> Vec<ContentPattern> {
        let mut patterns = Vec::new();

        // Detect code blocks
        if let Some(pos) = content.find("```") {
            if let Some(end) = content[pos + 3..].find("```") {
                patterns.push(ContentPattern {
                    semantic_type: SemanticType::Code,
                    content: content[pos..pos + end + 3].to_string(),
                    position: (pos, pos + end + 3),
                });
            }
        }

        // Detect inline code patterns
        if content.contains("fn ") || content.contains("function ")
            || content.contains("def ") || content.contains("class ")
        {
            patterns.push(ContentPattern {
                semantic_type: SemanticType::Code,
                content: "code pattern detected".to_string(),
                position: (0, 0),
            });
        }

        // Detect mathematical formulas
        if content.contains("\\frac") || content.contains("\\sum") || content.contains("∫")
            || content.contains("∑") || content.contains("π")
        {
            patterns.push(ContentPattern {
                semantic_type: SemanticType::Formula,
                content: "mathematical formula detected".to_string(),
                position: (0, 0),
            });
        }

        // Detect lists (3+ bullet items)
        let lines: Vec<&str> = content.lines().collect();
        let mut bullet_count = 0;
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("- ") || trimmed.starts_with("* ")
                || trimmed.starts_with("• ")
                || (trimmed.chars().next().map_or(false, |c| c.is_ascii_digit())
                    && trimmed.len() > 1
                    && trimmed.chars().nth(1) == Some('.'))
            {
                bullet_count += 1;
            }
        }
        if bullet_count >= 3 {
            patterns.push(ContentPattern {
                semantic_type: SemanticType::List,
                content: format!("list with {bullet_count} items"),
                position: (0, 0),
            });
        }

        patterns
    }

    /// Checks if content contains code patterns
    #[must_use]
    pub fn has_code(content: &str) -> bool {
        content.contains("```")
            || content.contains("fn ")
            || content.contains("function ")
            || content.contains("def ")
            || content.contains("class ")
    }

    /// Checks if content contains mathematical formulas
    #[must_use]
    pub fn has_formula(content: &str) -> bool {
        content.contains("\\frac")
            || content.contains("\\sum")
            || content.contains("∫")
            || content.contains("∑")
            || content.contains("π")
            || content.contains("√")
    }

    /// Checks if content is a list
    #[must_use]
    pub fn is_list(content: &str) -> bool {
        content.lines().filter(|line| {
            let t = line.trim();
            t.starts_with("- ") || t.starts_with("* ") || t.starts_with("• ")
        }).count() >= 3
    }
}

// ============================================================================
// Voice-Friendly Transformation
// ============================================================================

/// Voice-friendly transformation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceFriendlyResult {
    pub content: String,
    pub success: bool,
    pub confidence: f32,
    pub reading_time_seconds: u32,
    pub warnings: Vec<String>,
}

/// Validation result for voice-friendly content
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_voice_friendly: bool,
    pub score: u8,
    pub reading_time_seconds: u32,
    pub issues: Vec<String>,
}

/// Voice-friendly content transformer
pub struct VoiceFriendlyTransformer {
    max_reading_time: u32,
    reading_speed: u32,
}

impl Default for VoiceFriendlyTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl VoiceFriendlyTransformer {
    #[must_use]
    pub fn new() -> Self {
        Self {
            max_reading_time: 15,
            reading_speed: 10, // ~10 chars per second for comfortable speech
        }
    }

    /// Simple sync transformation (strips markdown, no LLM)
    /// Used as a lightweight check or when LLM is unavailable.
    pub async fn transform(&self, content: &str) -> VoiceFriendlyResult {
        let patterns = PatternDetector::detect_patterns(content);

        if patterns.is_empty() {
            let reading_time = self.estimate_reading_time(content);
            return VoiceFriendlyResult {
                content: content.to_string(),
                success: true,
                confidence: 1.0,
                reading_time_seconds: reading_time,
                warnings: Vec::new(),
            };
        }

        // Basic fallback: strip code block markers and summarize
        let transformed = content
            .replace("```", "")
            .replace("**", "")
            .replace("*", "")
            .replace("#", "");

        let reading_time = self.estimate_reading_time(&transformed);
        let mut warnings = Vec::new();
        if reading_time > self.max_reading_time {
            warnings.push(format!(
                "Content too long: {reading_time}s exceeds recommended {}s",
                self.max_reading_time
            ));
        }

        VoiceFriendlyResult {
            content: transformed,
            success: true,
            confidence: 0.5, // Low confidence for rule-based fallback
            reading_time_seconds: reading_time,
            warnings,
        }
    }

    /// LLM-driven transformation (primary method)
    /// Converts code, formulas, and lists to voice-friendly metaphors.
    pub async fn transform_with_agent(
        &self,
        content: &str,
        agent: &NightMindAgent,
    ) -> VoiceFriendlyResult {
        let patterns = PatternDetector::detect_patterns(content);

        if patterns.is_empty() {
            let reading_time = self.estimate_reading_time(content);
            return VoiceFriendlyResult {
                content: content.to_string(),
                success: true,
                confidence: 1.0,
                reading_time_seconds: reading_time,
                warnings: Vec::new(),
            };
        }

        // Build a unified prompt based on detected pattern types
        let type_names: Vec<&str> = patterns
            .iter()
            .map(|p| match p.semantic_type {
                SemanticType::Code => "代码",
                SemanticType::Formula => "数学公式",
                SemanticType::List => "列表",
                SemanticType::Technical => "技术术语",
                SemanticType::Plain => "普通文本",
            })
            .collect();

        let type_list = type_names.join("、");

        let prompt = format!(
            "以下内容包含{type_list}，不适合直接朗读。请将其转换为通俗易懂的比喻性解释：\n\
            - 代码：解释设计意图和核心逻辑，用生活中的比喻（就像...）\n\
            - 公式：解释含义和用途，用简单的例子\n\
            - 列表：编织成连贯的故事或流程，不要逐项朗读\n\
            - 技术术语：用日常语言解释概念\n\n\
            控制在80字以内，保持口语化风格。\n\n\
            原始内容：\n{content}"
        );

        match agent.prompt(&prompt).await {
            Ok(transformed) => {
                let reading_time = self.estimate_reading_time(&transformed);
                let mut warnings = Vec::new();
                if reading_time > self.max_reading_time {
                    warnings.push(format!("Content too long: {reading_time}s"));
                }
                VoiceFriendlyResult {
                    content: transformed,
                    success: true,
                    confidence: 0.9,
                    reading_time_seconds: reading_time,
                    warnings,
                }
            }
            Err(e) => {
                tracing::warn!("LLM transformation failed: {e}, using fallback");
                self.transform(content).await
            }
        }
    }

    /// Cache-aware version: checks cache before LLM, stores after
    pub async fn transform_with_agent_and_cache(
        &self,
        content: &str,
        agent: &NightMindAgent,
        cache: Option<&crate::core::content::cache::TransformCache>,
    ) -> VoiceFriendlyResult {
        if let Some(cache) = cache {
            if let Ok(Some(cached)) = cache.get(content).await {
                tracing::debug!("Using cached transformation");
                return cached;
            }
        }

        let result = self.transform_with_agent(content, agent).await;

        if let Some(cache) = cache {
            if let Err(e) = cache.set(content, result.clone()).await {
                tracing::warn!("Failed to cache transformation: {e}");
            }
        }

        result
    }

    /// Estimates reading time in seconds
    #[must_use]
    fn estimate_reading_time(&self, content: &str) -> u32 {
        let char_count = content.chars().count() as u32;
        (char_count / self.reading_speed).max(1)
    }

    /// Validates if content is voice-friendly
    #[must_use]
    pub fn validate_voice_friendly(content: &str) -> ValidationResult {
        let reading_time = Self::new().estimate_reading_time(content);
        let mut issues = Vec::new();
        let mut score = 100u8;

        if reading_time > 15 {
            issues.push("内容过长，建议分段".to_string());
            score = score.saturating_sub(20);
        }

        if PatternDetector::has_code(content) {
            issues.push("包含代码，需要转换".to_string());
            score = score.saturating_sub(30);
        }

        if PatternDetector::has_formula(content) {
            issues.push("包含公式，需要转换".to_string());
            score = score.saturating_sub(25);
        }

        if PatternDetector::is_list(content) {
            issues.push("包含列表，建议转换为故事".to_string());
            score = score.saturating_sub(15);
        }

        ValidationResult {
            is_voice_friendly: score > 70,
            score,
            reading_time_seconds: reading_time,
            issues,
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_detector_code() {
        let content = "```rust\nfn hello() { println!(\"Hi\"); }\n```";
        assert!(PatternDetector::has_code(content));
    }

    #[test]
    fn test_pattern_detector_formula() {
        let content = "The formula is \\frac{a}{b}";
        assert!(PatternDetector::has_formula(content));
    }

    #[test]
    fn test_pattern_detector_list() {
        let content = "- First item\n- Second item\n- Third item\n- Fourth item";
        assert!(PatternDetector::is_list(content));
    }

    #[test]
    fn test_detect_patterns_mixed() {
        let content = "Here's some code:\n```\nfn test() {}\n```\nAnd a list:\n- One\n- Two\n- Three";

        let patterns = PatternDetector::detect_patterns(content);
        assert!(!patterns.is_empty());

        let has_code = patterns.iter().any(|p| p.semantic_type == SemanticType::Code);
        let has_list = patterns.iter().any(|p| p.semantic_type == SemanticType::List);
        assert!(has_code);
        assert!(has_list);
    }

    #[test]
    fn test_reading_time_estimation() {
        let transformer = VoiceFriendlyTransformer::new();
        let content = "This is a test sentence with some words.";

        let time = transformer.estimate_reading_time(content);
        assert!(time > 0);
        assert!(time < 15);
    }

    #[test]
    fn test_validate_voice_friendly() {
        let good_content = "This is simple conversational text.";
        let result = VoiceFriendlyTransformer::validate_voice_friendly(good_content);

        assert!(result.is_voice_friendly);
        assert!(result.score > 70);
        assert!(result.issues.is_empty());
    }

    #[test]
    fn test_validate_not_voice_friendly_code() {
        let code_content = "function compute() { return x + y; }";
        let result = VoiceFriendlyTransformer::validate_voice_friendly(code_content);

        assert!(!result.is_voice_friendly);
        assert!(result.score <= 70);
        assert!(!result.issues.is_empty());
    }

    #[tokio::test]
    async fn test_transform_no_patterns() {
        let transformer = VoiceFriendlyTransformer::new();
        let content = "Simple text without any special patterns.";
        let result = transformer.transform(content).await;

        assert!(result.success);
        assert_eq!(result.confidence, 1.0);
        assert!(result.warnings.is_empty());
    }

    #[tokio::test]
    async fn test_transform_strips_markdown() {
        let transformer = VoiceFriendlyTransformer::new();
        let content = "```code```";
        let result = transformer.transform(content).await;

        assert!(result.success);
        assert!(!result.content.contains("```"));
    }
}
