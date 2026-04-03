// ============================================================
// Content Transformer
// ============================================================
//! Content transformation and processing.
//!
//! This module provides utilities for transforming and processing
//! various types of content within the system.

use crate::core::agent::{AgentBuilder, NightMindAgent};
use crate::error::{NightMindError, Result};
use crate::config::Settings;
use crate::core::content::cache::TransformCache;
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

/// Type alias for transform results
pub type TransformResult<T> = std::result::Result<T, TransformError>;

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
    ) -> TransformResult<TransformedContent>;
}

/// Markdown transformer for converting between text and markdown
pub struct MarkdownTransformer;

impl ContentTransformer for MarkdownTransformer {
    fn transform(
        &self,
        content: &str,
        from_type: ContentType,
        to_type: ContentType,
    ) -> TransformResult<TransformedContent> {
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
    ) -> TransformResult<TransformedContent> {
        match (from_type, to_type) {
            (ContentType::Json, ContentType::Text) => {
                // Pretty print JSON
                let value: serde_json::Value = serde_json::from_str(content)
                    .map_err(|e| TransformError::Parse(e.to_string()))?;
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

/// ============================================================================
// Semantic Content Transformation (Core Product Feature)
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

/// Specific code pattern types for precise transformation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodePattern {
    /// React Hooks (useState, useEffect, etc.)
    ReactHook,
    /// Context API (React Context, context package)
    Context,
    /// Async/Await pattern
    AsyncAwait,
    /// Promise chain
    Promise,
    /// Recursive function
    Recursion,
    /// Tree structure
    Tree,
    /// Graph algorithms
    Graph,
    /// Closure
    Closure,
    /// Error handling (try-catch, Result, Option)
    ErrorHandling,
    /// Generics
    Generics,
    /// Pattern matching (match, switch)
    PatternMatching,
    /// Decorator
    Decorator,
    /// Class definition
    Class,
    /// Loop (for, while)
    Loop,
    /// Function definition
    Function,
    /// Unknown pattern
    Unknown,
}

/// Detected content pattern
#[derive(Debug, Clone)]
pub struct ContentPattern {
    /// The semantic type
    pub semantic_type: SemanticType,
    /// The matched content
    pub content: String,
    /// Position in original text (start, end)
    pub position: (usize, usize),
}

/// Voice-friendly transformation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceFriendlyResult {
    /// Transformed content
    pub content: String,
    /// Whether transformation was successful
    pub success: bool,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Estimated reading time in seconds
    pub reading_time_seconds: u32,
    /// Warnings if content is too long or complex
    pub warnings: Vec<String>,
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

        // Detect code-like patterns (function definitions, etc.)
        if content.contains("fn ") || content.contains("function ") || content.contains("def ") {
            patterns.push(ContentPattern {
                semantic_type: SemanticType::Code,
                content: "function definition detected".to_string(),
                position: (0, 0),
            });
        }

        // Detect mathematical formulas
        if content.contains("\\frac") || content.contains("\\sum") || content.contains("∫")
            || content.contains("∑") || content.contains("π") {
            patterns.push(ContentPattern {
                semantic_type: SemanticType::Formula,
                content: "mathematical formula detected".to_string(),
                position: (0, 0),
            });
        }

        // Detect lists
        let lines: Vec<&str> = content.lines().collect();
        let mut bullet_count = 0;
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("- ") || trimmed.starts_with("* ")
                || trimmed.starts_with("• ") || trimmed.chars().next().map_or(false, |c| c.is_ascii_digit() && trimmed.len() > 1 && trimmed.chars().nth(1) == Some('.')) {
                bullet_count += 1;
            }
        }
        if bullet_count >= 3 {
            patterns.push(ContentPattern {
                semantic_type: SemanticType::List,
                content: format!("list with {} items", bullet_count),
                position: (0, 0),
            });
        }

        patterns
    }

    /// Detects specific code pattern type for better transformations
    #[must_use]
    pub fn detect_code_pattern(content: &str) -> CodePattern {
        // Check in priority order for more specific patterns first

        // 1. Decorator (check first before function detection)
        if content.contains("@") || content.contains("decorator") {
            return CodePattern::Decorator;
        }

        // 2. React Hooks
        if content.contains("useState") || content.contains("useEffect")
            || content.contains("useContext") || content.contains("useCallback") {
            return CodePattern::ReactHook;
        }

        // 3. Context API
        if content.contains("Context") || content.contains("context")
            || content.contains("useContext") || content.contains("withContext") {
            return CodePattern::Context;
        }

        // 4. Async/Await
        if content.contains("async ") || content.contains("await ")
            || content.contains("AsyncFunction") || content.contains("async def") {
            return CodePattern::AsyncAwait;
        }

        // 5. Promise
        if content.contains("Promise") || content.contains(".then")
            || content.contains(".catch") || content.contains("async/await") {
            return CodePattern::Promise;
        }

        // 6. Recursion
        // Only detect if we have a function that calls itself
        // Check for more explicit recursion indicators
        if content.contains("recursive") || content.contains("factorial")
            || content.contains("fibonacci") || (content.contains("fn ") && content.contains("return") && content.len() < 200) {
            // Additional check: function name appears in parentheses (calling itself)
            if let Some(pos) = content.find("fn ").or_else(|| content.find("def ")) {
                let after_fn = &content[pos..];
                if let Some(paren_pos) = after_fn.find('(') {
                    let func_name = after_fn[3..paren_pos].trim();
                    if !func_name.is_empty() && content.contains(&format!("{}(", func_name)) {
                        return CodePattern::Recursion;
                    }
                }
            }
        }

        // 7. Tree structures
        if content.contains("tree") || content.contains("Tree")
            || content.contains(".left") || content.contains(".right")
            || content.contains("TreeNode") || content.contains("children") && content.contains("parent") {
            return CodePattern::Tree;
        }

        // 8. Graph algorithms
        if content.contains("graph") || content.contains("Graph")
            || content.contains("adjacency") || content.contains("vertex")
            || content.contains("edge") || content.contains("Dijkstra")
            || content.contains("BFS") || content.contains("DFS") {
            return CodePattern::Graph;
        }

        // 9. Closure
        if content.contains("closure") || content.contains("Closure")
            || (content.contains("fn ") && content.contains("move") && content.contains("|"))
            || (content.contains("function") && content.contains("return") && content.contains("function")) {
            return CodePattern::Closure;
        }

        // 10. Error handling
        if content.contains("try {") || content.contains("try:")
            || content.contains("catch") || content.contains("except")
            || content.contains("Result<") || content.contains("Option<") {
            return CodePattern::ErrorHandling;
        }

        // 11. Generics
        if content.contains("<") && content.contains(">")
            && (content.contains("<T>") || content.contains("<T,")
            || content.contains("typename") || content.contains("generic")) {
            return CodePattern::Generics;
        }

        // 12. Pattern matching
        if content.contains("match ") || content.contains("switch ")
            || content.contains("case ") || content.contains("Pattern matching") {
            return CodePattern::PatternMatching;
        }

        // 13. Class
        if content.contains("class ") {
            return CodePattern::Class;
        }

        // 14. Loop
        if content.contains("for ") || content.contains("while ") || content.contains("loop") {
            return CodePattern::Loop;
        }

        // 15. Function (default)
        if content.contains("fn ") || content.contains("def ") || content.contains("function ") {
            return CodePattern::Function;
        }

        CodePattern::Unknown
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
        let lines: Vec<&str> = content.lines().collect();
        let mut bullet_count = 0;
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.starts_with("- ") || trimmed.starts_with("* ")
                || trimmed.starts_with("• ") {
                bullet_count += 1;
            }
        }
        bullet_count >= 3
    }
}

/// Voice-friendly content transformer
pub struct VoiceFriendlyTransformer {
    /// Maximum reading time per segment (seconds)
    max_reading_time: u32,
    /// Average reading speed (chars per second)
    reading_speed: u32,
}

impl VoiceFriendlyTransformer {
    /// Creates a new voice-friendly transformer
    #[must_use]
    pub fn new() -> Self {
        Self {
            max_reading_time: 15,
            reading_speed: 10, // ~10 chars per second for comfortable speech
        }
    }

    /// Transforms content to be voice-friendly (rule-based only)
    ///
    /// # Arguments
    ///
    /// * `content` - Input content
    ///
    /// # Returns
    ///
    /// Voice-friendly transformation result
    pub async fn transform(&self, content: &str) -> VoiceFriendlyResult {
        let patterns = PatternDetector::detect_patterns(content);

        // If no patterns detected, content is already voice-friendly
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

        // Transform using rule-based approach only
        let mut transformed = content.to_string();
        let mut warnings = Vec::new();
        let mut total_confidence = 0.0;

        for pattern in &patterns {
            match pattern.semantic_type {
                SemanticType::Code => {
                    let result = Self::transform_code_rules(&pattern.content);
                    transformed = transformed.replace(&pattern.content, &result);
                    total_confidence += 0.6;
                }
                SemanticType::Formula => {
                    let result = Self::transform_formula_rules(&pattern.content);
                    transformed = transformed.replace(&pattern.content, &result);
                    total_confidence += 0.7;
                }
                SemanticType::List => {
                    let result = Self::transform_list_rules(&pattern.content);
                    transformed = transformed.replace(&pattern.content, &result);
                    total_confidence += 0.65;
                }
                _ => {
                    warnings.push(format!("Unsupported pattern type: {:?}", pattern.semantic_type));
                }
            }
        }

        let avg_confidence: f32 = if patterns.is_empty() {
            1.0
        } else {
            (total_confidence / patterns.len() as f64) as f32
        };

        let reading_time = self.estimate_reading_time(&transformed);

        VoiceFriendlyResult {
            content: transformed,
            success: true,
            confidence: avg_confidence,
            reading_time_seconds: reading_time,
            warnings,
        }
    }

    /// Transforms content to be voice-friendly with AI assistance and caching
    ///
    /// # Arguments
    ///
    /// * `content` - Input content
    /// * `agent` - AI agent for intelligent transformations
    /// * `cache` - Optional cache for storing/retrieving transformations
    ///
    /// # Returns
    ///
    /// Voice-friendly transformation result
    pub async fn transform_with_agent_and_cache(
        &self,
        content: &str,
        agent: &NightMindAgent,
        cache: Option<&TransformCache>,
    ) -> VoiceFriendlyResult {
        // Try cache first if available
        if let Some(cache) = cache {
            if let Ok(Some(cached_result)) = cache.get(content).await {
                tracing::debug!("Using cached transformation for content");
                return cached_result;
            }
        }

        // Perform transformation
        let result = self.transform_with_agent(content, agent).await;

        // Cache the result if cache is available
        if let Some(cache) = cache {
            if let Err(e) = cache.set(content, result.clone()).await {
                tracing::warn!("Failed to cache transformation result: {}", e);
            }
        }

        result
    }

    /// Transforms content to be voice-friendly with AI assistance
    ///
    /// # Arguments
    ///
    /// * `content` - Input content
    /// * `agent` - AI agent for intelligent transformations
    ///
    /// # Returns
    ///
    /// Voice-friendly transformation result
    pub async fn transform_with_agent(
        &self,
        content: &str,
        agent: &NightMindAgent,
    ) -> VoiceFriendlyResult {
        let patterns = PatternDetector::detect_patterns(content);

        // If no patterns detected, content is already voice-friendly
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

        // Transform based on detected patterns
        let mut transformed = content.to_string();
        let mut warnings = Vec::new();
        let mut total_confidence = 0.0;

        for pattern in &patterns {
            match pattern.semantic_type {
                SemanticType::Code => {
                    // Use AI to transform code
                    match self.transform_code_with_ai(&pattern.content, agent).await {
                        Ok(result) => {
                            transformed = transformed.replace(&pattern.content, &result.content);
                            total_confidence += result.confidence;
                        }
                        Err(_) => {
                            // Fallback to rule-based transformation
                            let result = Self::transform_code_rules(&pattern.content);
                            transformed = transformed.replace(&pattern.content, &result);
                            total_confidence += 0.6;
                        }
                    }
                }
                SemanticType::Formula => {
                    match self.transform_formula_with_ai(&pattern.content, agent).await {
                        Ok(result) => {
                            transformed = transformed.replace(&pattern.content, &result.content);
                            total_confidence += result.confidence;
                        }
                        Err(_) => {
                            let result = Self::transform_formula_rules(&pattern.content);
                            transformed = transformed.replace(&pattern.content, &result);
                            total_confidence += 0.7;
                        }
                    }
                }
                SemanticType::List => {
                    match self.transform_list_with_ai(content, agent).await {
                        Ok(result) => {
                            transformed = result.content;
                            total_confidence += result.confidence;
                        }
                        Err(_) => {
                            let result = Self::transform_list_rules(content);
                            transformed = result;
                            total_confidence += 0.65;
                        }
                    }
                }
                _ => {
                    warnings.push(format!("Unsupported pattern type: {:?}", pattern.semantic_type));
                }
            }
        }

        let reading_time = self.estimate_reading_time(&transformed);

        // Check reading time
        if reading_time > self.max_reading_time {
            warnings.push(format!(
                "Content too long: {}s exceeds recommended {}s",
                reading_time, self.max_reading_time
            ));
        }

        VoiceFriendlyResult {
            content: transformed,
            success: true,
            confidence: (total_confidence / patterns.len() as f32).min(1.0),
            reading_time_seconds: reading_time,
            warnings,
        }
    }

    /// Transforms code using AI
    async fn transform_code_with_ai(
        &self,
        code: &str,
        agent: &NightMindAgent,
    ) -> std::result::Result<VoiceFriendlyResult, NightMindError> {
        let prompt = format!(
            "将以下代码转换为适合语音朗读的比喻性解释。\
            不要朗读代码本身，而是解释它的设计意图和核心逻辑，\
            使用生活中简单的比喻（就像...）。控制在50字以内：\n\n{}",
            code
        );

        let response = agent.prompt(&prompt).await?;

        let reading_time = self.estimate_reading_time(&response);

        Ok(VoiceFriendlyResult {
            content: response,
            success: true,
            confidence: 0.9,
            reading_time_seconds: reading_time,
            warnings: Vec::new(),
        })
    }

    /// Transforms formula using AI
    async fn transform_formula_with_ai(
        &self,
        formula: &str,
        agent: &NightMindAgent,
    ) -> std::result::Result<VoiceFriendlyResult, NightMindError> {
        let prompt = format!(
            "将以下数学公式转换为通俗的口语解释。\
            不要朗读公式本身，而是解释它的含义和用途，\
            使用生活中的例子。控制在50字以内：\n\n{}",
            formula
        );

        let response = agent.prompt(&prompt).await?;

        let reading_time = self.estimate_reading_time(&response);

        Ok(VoiceFriendlyResult {
            content: response,
            success: true,
            confidence: 0.85,
            reading_time_seconds: reading_time,
            warnings: Vec::new(),
        })
    }

    /// Transforms list using AI
    async fn transform_list_with_ai(
        &self,
        list: &str,
        agent: &NightMindAgent,
    ) -> std::result::Result<VoiceFriendlyResult, NightMindError> {
        let prompt = format!(
            "将以下列表转换为连贯的故事或流程描述。\
            不要逐项朗读，而是把它们编织成一个有逻辑的整体，\
            使用生命周期或流程的比喻。控制在80字以内：\n\n{}",
            list
        );

        let response = agent.prompt(&prompt).await?;

        let reading_time = self.estimate_reading_time(&response);

        Ok(VoiceFriendlyResult {
            content: response,
            success: true,
            confidence: 0.88,
            reading_time_seconds: reading_time,
            warnings: Vec::new(),
        })
    }

    /// Rule-based code transformation (fallback) - supports 15 code patterns
    fn transform_code_rules(code: &str) -> String {
        // Detect specific pattern first
        let pattern = PatternDetector::detect_code_pattern(code);

        match pattern {
            CodePattern::ReactHook => {
                if code.contains("useState") {
                    "这就像给卡片加上一个可变的标签，每次标签改变时，界面就会自动更新显示。这就是React的状态管理。".to_string()
                } else if code.contains("useEffect") {
                    "这就像设定了一个闹钟，当特定条件满足时，自动执行某些操作。比如数据加载完成后自动显示。".to_string()
                } else {
                    "这就像给组件添加超能力，让它能在特定时机自动做事情，而不需要手动触发。这就是React钩子的作用。".to_string()
                }
            }
            CodePattern::Context => {
                "这就像一个全球广播系统，无论你在哪里，都能收到最新的消息。不用一层层传递，直接就能获取共享数据。".to_string()
            }
            CodePattern::AsyncAwait => {
                "这就像在餐厅点餐，点完后不用站在窗口等，可以坐下刷手机，好了会被叫到。异步就是不用傻等，可以做其他事。".to_string()
            }
            CodePattern::Promise => {
                "这就像点外卖时的订单号，你拿到号后就知道餐会做好，到时候要么给你送到，要么告诉你出了问题。这就是承诺机制。".to_string()
            }
            CodePattern::Recursion => {
                "这就像俄罗斯套娃，打开一个里面有另一个，再打开还有，直到最小的那个。函数自己调用自己，不断深入直到找到答案。".to_string()
            }
            CodePattern::Tree => {
                "这就像家谱树，从祖先开始，一代代分支下去。每个节点可以有多个孩子，但只有一个父母。查找时就像顺着族谱找人。".to_string()
            }
            CodePattern::Graph => {
                "这就像社交媒体的好友网络，每个人都可以连接很多人，而且可以互相访问。比树更复杂，因为连接关系更自由。".to_string()
            }
            CodePattern::Closure => {
                "这就像一个随身背包，函数带着它诞生时的环境，即使在别的地方执行，也能用到创建时的变量。就像带着家乡的水在异乡饮用。".to_string()
            }
            CodePattern::ErrorHandling => {
                "这就像开车系安全带，正常时感觉不到，但一旦出事故就能保护你。程序出错时不会崩溃，而是优雅地处理问题。".to_string()
            }
            CodePattern::Generics => {
                "这就像万能插座，不管是什么电器都能插。不需要为每种电器单独造插座，一个模板适配所有类型。这就是泛型的威力。".to_string()
            }
            CodePattern::PatternMatching => {
                "这就像分拣中心的传送带，根据包裹的形状、大小，自动分流到不同的处理线。比用一堆if判断更清晰高效。".to_string()
            }
            CodePattern::Decorator => {
                "就像给礼物加包装纸，不改变礼物本身，但增加了额外功能。这就是装饰器的作用，在不修改原代码的情况下增强功能。".to_string()
            }
            CodePattern::Class => {
                "这就像一个产品的设计图纸，定义了它有什么属性和能做什么。根据图纸可以造出很多产品实例，每个都有相同的结构。".to_string()
            }
            CodePattern::Loop => {
                "这就像流水线重复做同一件事，对每个产品都执行相同操作。比如安检时，每个人都要过同样的检查流程。".to_string()
            }
            CodePattern::Function => {
                "这就像一个加工流程，输入原料，按步骤处理，输出成品。有了这个流程，每次需要时就不用重新写步骤，直接调用就行。".to_string()
            }
            CodePattern::Unknown => {
                "这段代码定义了程序的执行逻辑，就像菜谱的步骤说明，告诉计算机一步步该做什么。".to_string()
            }
        }
    }

    /// Rule-based formula transformation (fallback)
    fn transform_formula_rules(formula: &str) -> String {
        if formula.contains("\\frac") {
            "这是分数公式，用分子的值除以分母的值。就像把蛋糕分给几个人。".to_string()
        } else if formula.contains("∑") || formula.contains("\\sum") {
            "这是求和公式，把一堆数加起来得到总数。就像统计购物车的总金额。".to_string()
        } else if formula.contains("∫") {
            "这是积分公式，计算曲线下的面积。就像计算不均匀田地的总面积。".to_string()
        } else if formula.contains("π") {
            "这是圆周率，圆的周长和直径的比例。约等于3.14。".to_string()
        } else {
            "这是一个数学公式，描述了数量之间的精确关系。".to_string()
        }
    }

    /// Rule-based list transformation (fallback)
    fn transform_list_rules(_list: &str) -> String {
        "这就像一个完整的故事流程，从开始到结束，环环相扣，形成整体。".to_string()
    }

    /// Estimates reading time in seconds
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

        // Check reading time
        if reading_time > 15 {
            issues.push("内容过长，建议分段".to_string());
            score = score.saturating_sub(20);
        }

        // Check for code
        if PatternDetector::has_code(content) {
            issues.push("包含代码，需要转换".to_string());
            score = score.saturating_sub(30);
        }

        // Check for formulas
        if PatternDetector::has_formula(content) {
            issues.push("包含公式，需要转换".to_string());
            score = score.saturating_sub(25);
        }

        // Check for long lists
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

impl Default for VoiceFriendlyTransformer {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation result for voice-friendly content
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether content is voice-friendly
    pub is_voice_friendly: bool,
    /// Score (0-100, higher is better)
    pub score: u8,
    /// Estimated reading time in seconds
    pub reading_time_seconds: u32,
    /// Issues found (empty if none)
    pub issues: Vec<String>,
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

    // Tests for semantic transformation

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
    fn test_code_transformation_decorator() {
        let code = "@app.route('/')\ndef home(): return 'Hello'";
        let result = VoiceFriendlyTransformer::transform_code_rules(code);

        // Decorator pattern should be detected
        println!("Decorator test result: {}", result);
        assert!(result.contains("装饰器") || result.contains("包装") || result.contains("礼物"));
    }

    #[test]
    fn test_formula_transformation_fraction() {
        let formula = "\\frac{1}{2}";
        let result = VoiceFriendlyTransformer::transform_formula_rules(formula);

        assert!(result.contains("分数"));
        assert!(result.contains("除"));
    }

    #[test]
    fn test_formula_transformation_sum() {
        let formula = "\\sum_{i=1}^{n} x_i";
        let result = VoiceFriendlyTransformer::transform_formula_rules(formula);

        assert!(result.contains("求和") || result.contains("总数"));
    }

    #[test]
    fn test_reading_time_estimation() {
        let transformer = VoiceFriendlyTransformer::new();
        let content = "This is a test sentence with some words.";

        let time = transformer.estimate_reading_time(content);
        assert!(time > 0);
        assert!(time < 15); // Should be readable in 15 seconds
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

    #[test]
    fn test_detect_patterns_mixed() {
        let content = "Here's some code:\n```\nfn test() {}\n```\nAnd a list:\n- One\n- Two\n- Three";

        let patterns = PatternDetector::detect_patterns(content);
        assert!(!patterns.is_empty());

        // Should detect code and list
        let has_code = patterns.iter().any(|p| p.semantic_type == SemanticType::Code);
        let has_list = patterns.iter().any(|p| p.semantic_type == SemanticType::List);
        assert!(has_code);
        assert!(has_list);
    }

    // Tests for 15 code patterns

    #[test]
    fn test_detect_react_hook() {
        let code = "const [count, setCount] = useState(0);";
        let pattern = PatternDetector::detect_code_pattern(code);
        assert_eq!(pattern, CodePattern::ReactHook);
    }

    #[test]
    fn test_transform_react_hook_usestate() {
        let code = "useState(0)";
        let result = VoiceFriendlyTransformer::transform_code_rules(code);
        assert!(result.contains("状态") || result.contains("标签") || result.contains("更新"));
    }

    #[test]
    fn test_detect_context() {
        let code = "const ThemeContext = React.createContext();";
        let pattern = PatternDetector::detect_code_pattern(code);
        assert_eq!(pattern, CodePattern::Context);
    }

    #[test]
    fn test_transform_context() {
        let code = "ThemeContext";
        let result = VoiceFriendlyTransformer::transform_code_rules(code);
        assert!(result.contains("广播") || result.contains("传递") || result.contains("共享"));
    }

    #[test]
    fn test_detect_async_await() {
        let code = "async function fetchData() { await response.json(); }";
        let pattern = PatternDetector::detect_code_pattern(code);
        assert_eq!(pattern, CodePattern::AsyncAwait);
    }

    #[test]
    fn test_transform_async_await() {
        let code = "await response.json()";
        let result = VoiceFriendlyTransformer::transform_code_rules(code);
        assert!(result.contains("点餐") || result.contains("等") || result.contains("异步"));
    }

    #[test]
    fn test_detect_promise() {
        let code = "fetch(url).then(response => response.json())";
        let pattern = PatternDetector::detect_code_pattern(code);
        assert_eq!(pattern, CodePattern::Promise);
    }

    #[test]
    fn test_transform_promise() {
        let code = "Promise.then";
        let result = VoiceFriendlyTransformer::transform_code_rules(code);
        assert!(result.contains("承诺") || result.contains("订单") || result.contains("外卖"));
    }

    #[test]
    fn test_detect_closure() {
        let code = "fn adder(x: i32) -> impl Fn(i32) -> i32 { move |y| x + y }";
        let pattern = PatternDetector::detect_code_pattern(code);
        assert_eq!(pattern, CodePattern::Closure);
    }

    #[test]
    fn test_transform_closure() {
        let code = "closure function";
        let result = VoiceFriendlyTransformer::transform_code_rules(code);
        assert!(result.contains("背包") || result.contains("环境") || result.contains("闭包"));
    }

    #[test]
    fn test_detect_error_handling() {
        let code = "try { riskyOperation(); } catch (e) { handleError(e); }";
        let pattern = PatternDetector::detect_code_pattern(code);
        assert_eq!(pattern, CodePattern::ErrorHandling);
    }

    #[test]
    fn test_transform_error_handling() {
        let code = "try { } catch";
        let result = VoiceFriendlyTransformer::transform_code_rules(code);
        assert!(result.contains("安全带") || result.contains("错误") || result.contains("保护"));
    }

    #[test]
    fn test_detect_generics() {
        let code = "fn identity<T>(x: T) -> T { x }";
        let pattern = PatternDetector::detect_code_pattern(code);
        assert_eq!(pattern, CodePattern::Generics);
    }

    #[test]
    fn test_transform_generics() {
        let code = "struct Container<T> { value: T }";
        let result = VoiceFriendlyTransformer::transform_code_rules(code);
        assert!(result.contains("万能") || result.contains("泛型") || result.contains("适配"));
    }

    #[test]
    fn test_detect_pattern_matching() {
        let code = "match value { Some(x) => println!(\"{}\", x), None => () }";
        let pattern = PatternDetector::detect_code_pattern(code);
        assert_eq!(pattern, CodePattern::PatternMatching);
    }

    #[test]
    fn test_transform_pattern_matching() {
        let code = "match value";
        let result = VoiceFriendlyTransformer::transform_code_rules(code);
        assert!(result.contains("分拣") || result.contains("匹配") || result.contains("模式"));
    }

    #[test]
    fn test_detect_tree() {
        let code = "struct TreeNode { left: Option<Box<TreeNode>>, right: Option<Box<TreeNode>> }";
        let pattern = PatternDetector::detect_code_pattern(code);
        assert_eq!(pattern, CodePattern::Tree);
    }

    #[test]
    fn test_transform_tree() {
        let code = "tree.left.right";
        let result = VoiceFriendlyTransformer::transform_code_rules(code);
        assert!(result.contains("家谱") || result.contains("树") || result.contains("分支"));
    }

    #[test]
    fn test_detect_graph() {
        let code = "struct Graph { vertices: Vec<Node>, edges: Vec<Edge> }";
        let pattern = PatternDetector::detect_code_pattern(code);
        assert_eq!(pattern, CodePattern::Graph);
    }

    #[test]
    fn test_transform_graph() {
        let code = "Dijkstra algorithm on graph";
        let result = VoiceFriendlyTransformer::transform_code_rules(code);
        assert!(result.contains("社交") || result.contains("网络") || result.contains("图"));
    }

    #[test]
    fn test_detect_decorator() {
        let code = "@app.route('/')\ndef home(): pass";
        let pattern = PatternDetector::detect_code_pattern(code);
        assert_eq!(pattern, CodePattern::Decorator);
    }

    #[test]
    fn test_detect_class() {
        let code = "class User: def __init__(self, name): self.name = name";
        let pattern = PatternDetector::detect_code_pattern(code);
        assert_eq!(pattern, CodePattern::Class);
    }

    #[test]
    fn test_detect_loop() {
        let code = "for i in 0..10 { println!(\"{}\"); }";
        let pattern = PatternDetector::detect_code_pattern(code);
        assert_eq!(pattern, CodePattern::Loop);
    }

    #[test]
    fn test_detect_function() {
        let code = "fn calculate(x: i32, y: i32) -> i32 { x + y }";
        let pattern = PatternDetector::detect_code_pattern(code);
        assert_eq!(pattern, CodePattern::Function);
    }

    #[test]
    fn test_detect_unknown_pattern() {
        let code = "just some random text without specific patterns";
        let pattern = PatternDetector::detect_code_pattern(code);
        assert_eq!(pattern, CodePattern::Unknown);
    }
}
