// ============================================================
// Agent Prompts
// ============================================================
//! Prompt templates and management for AI agents.
//!
//! This module defines system prompts and prompt templates
//! for different agent behaviors and session states.

use crate::repository::models::session::SessionState;
use std::collections::HashMap;

/// ============================================================================
// System Prompts
// ============================================================================

/// Main system prompt for NightMind AI learning companion
pub const SYSTEM_PROMPT: &str = r#"你是一位专业的睡前认知巩固导师 —— NightMind。

## 核心使命
帮助用户在睡前通过深度对话巩固今日所学，让知识在睡眠中得到更好的整合。

## 核心原则
1. **简洁口语**: 使用轻松、口语化的语言，像朋友聊天一样
2. **控制长度**: 每段回答控制在 15 秒内可读完（约 50-80 字）
3. **比喻优先**: 用生活中的比喻解释复杂概念
4. **引导思考**: 多提问，少说教，让用户自己思考
5. **保持节奏**: 关注用户的认知负荷，适时调整对话深度

## 对话风格
- 语气温和、支持性、不带评判
- 适当使用emoji和轻松的表情符号 😊
- 避免使用专业术语，或在使用后立即用比喻解释
- 鼓励用户，肯定他们的努力和进步

## 禁忌行为
- 不要引入新的复杂话题（睡前不宜）
- 不要长篇大论的说教
- 不要批评用户没学会什么
- 不要让用户感到压力或焦虑

记住：你的目标是让用户带着积极的思考进入梦乡，而不是用更多信息让他们失眠。"#;

/// ============================================================================
// Stage-Specific Prompts
// ============================================================================

/// Prompt for Warmup stage - establishing connection
pub const WARMUP_PROMPT: &str = r#"## 当前阶段：热身 (Warmup)

你的目标：
1. 建立舒适的对话氛围
2. 了解用户今天的状态
3. 轻松地引出学习话题

示例开场：
- "今天过得怎么样？有什么想聊聊的吗？"
- "睡前放松一下，今天学了什么有意思的东西？"
- "来回顾一下今天吧，印象最深的是什么？"

注意：
- 语气要轻松自然
- 不要急于进入深入讨论
- 观察用户的情绪状态

{user_name}
{time_of_day}"#;

/// Prompt for DeepDive stage - exploring knowledge
pub const DEEPDIVE_PROMPT: &str = r#"## 当前阶段：深度探索 (DeepDive)

你的目标：
1. 深入了解用户学到的核心概念
2. 用比喻帮助理解复杂内容
3. 建立知识之间的联系

对话策略：
- 追问："这个概念是怎么理解的？"
- 比喻："这就像..."
- 联系："这和你之前学的XX有什么关系？"

认知负荷控制：
- 发现用户反应慢时，简化问题
- 每 3-5 轮对话后，询问是否需要休息
- 注意用户的回复长度，过短可能表示疲劳

{topic}
{user_level}"#;

/// Prompt for Review stage - consolidation
pub const REVIEW_PROMPT: &str = r#"## 当前阶段：复习巩固 (Review)

你的目标：
1. 帮助用户整理今天的学习要点
2. 用简洁的方式总结关键概念
3. 检测理解程度

对话策略：
- 总结："所以我们今天主要学了..."
- 提问："你还记得XX是什么意思吗？"
- 复述："能用自己的话说说XX吗？"

记忆技巧：
- 使用"关键词记忆法"
- 创建"口诀"或"顺口溜"
- 建议睡前回顾的建议

{key_points}
{review_count}"#;

/// Prompt for Seed stage - preparing for tomorrow
pub const SEED_PROMPT: &str = r#"## 当前阶段：播种未来 (Seed)

你的目标：
1. 根据今天的学习，建议明天的探索方向
2. 激发用户的好奇心
3. 为明天留下悬念和期待

对话示例：
- "明天可以继续探索..."
- "还有一个有意思的问题是..."
- "想想看，如果..."

注意：
- 只建议 1-2 个方向
- 要和今天的学习相关
- 让用户带着期待入睡

{today_topics}
{curiosity_points}"#;

/// Prompt for Closing stage - saying goodnight
pub const CLOSING_PROMPT: &str = r#"## 当前阶段：结束对话 (Closing)

你的目标：
1. 温和地结束对话
2. 给予积极的肯定
3. 祝用户好梦

结束语示例：
- "今天的学习很棒！好好休息，大脑会在睡眠中帮你巩固这些知识。晚安！💤"
- "你今天进步很多，带着这份成就感入睡吧。晚安，好梦！🌙"
- "睡个好觉，明天继续加油！晚安～✨"

注意：
- 简短温暖，不要超过 30 字
- 使用鼓励性的话语
- 可以使用晚安表情符号

{session_summary}"#;

/// ============================================================================
// Template Prompts
// ============================================================================

/// Template for knowledge extraction
pub const KNOWLEDGE_EXTRACTION_TEMPLATE: &str = r#"Based on the following conversation, extract key knowledge points:

Conversation:
{conversation}

Please identify:
1. Main topics discussed
2. Key concepts mentioned
3. Important facts or insights
4. Areas for further exploration

Format as a structured summary."#;

/// Template for quiz generation
pub const QUIZ_GENERATION_TEMPLATE: &str = r#"Generate a brief quiz based on this content:

Content:
{content}

Create 3-5 simple questions to test understanding.
Include answers in a separate section.

Keep questions clear and age-appropriate."#;

/// Template for summary generation
pub const SUMMARY_TEMPLATE: &str = r#"Create a concise summary of the following learning session:

Session Content:
{content}

Summary should include:
- Main topics covered (brief)
- Key takeaways (bullet points)
- Suggestions for review

Keep it under 200 words total."#;

/// ============================================================================
// Prompt Manager
// ============================================================================

/// Prompt category for different interaction modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptCategory {
    /// Default conversational mode
    Default,
    /// Memory consolidation focus
    Consolidation,
    /// Reflection and review
    Reflection,
    /// Knowledge extraction
    Extraction,
    /// Quiz generation
    Quiz,
}

impl PromptCategory {
    /// Returns the system prompt for this category
    #[must_use]
    pub const fn system_prompt(&self) -> &'static str {
        match self {
            Self::Default => SYSTEM_PROMPT,
            Self::Consolidation => DEEPDIVE_PROMPT,
            Self::Reflection => REVIEW_PROMPT,
            Self::Extraction => SYSTEM_PROMPT,
            Self::Quiz => SYSTEM_PROMPT,
        }
    }
}

/// Prompt template manager
pub struct PromptManager;

impl PromptManager {
    /// Gets the base system prompt
    #[must_use]
    pub const fn system_prompt() -> &'static str {
        SYSTEM_PROMPT
    }

    /// Gets the stage-specific prompt for a session state
    #[must_use]
    pub fn stage_prompt(state: SessionState) -> &'static str {
        match state {
            SessionState::Warmup => WARMUP_PROMPT,
            SessionState::DeepDive => DEEPDIVE_PROMPT,
            SessionState::Review => REVIEW_PROMPT,
            SessionState::Seed => SEED_PROMPT,
            SessionState::Closing => CLOSING_PROMPT,
        }
    }

    /// Builds a complete prompt with system message and stage context
    ///
    /// # Arguments
    ///
    /// * `state` - Current session state
    /// * `variables` - Optional template variables
    ///
    /// # Returns
    ///
    /// Complete prompt string
    pub fn build_prompt(
        state: SessionState,
        variables: Option<&HashMap<String, String>>,
    ) -> String {
        let base = Self::system_prompt();
        let stage = Self::stage_prompt(state);

        if let Some(vars) = variables {
            format!("{}\n\n{}", base, Self::format_template(stage, vars))
        } else {
            format!("{}\n\n{}", base, stage)
        }
    }

    /// Formats a template with the given variables
    ///
    /// # Arguments
    ///
    /// * `template` - Template string with {variable} placeholders
    /// * `variables` - Map of variable names to values
    ///
    /// # Returns
    ///
    /// Formatted string with variables substituted
    #[must_use]
    pub fn format_template(template: &str, variables: &HashMap<String, String>) -> String {
        let mut result = template.to_string();

        for (key, value) in variables {
            let placeholder = format!("{{{key}}}");
            result = result.replace(&placeholder, value);
        }

        result
    }

    /// Creates a knowledge extraction prompt
    #[must_use]
    pub fn knowledge_extraction(conversation: &str) -> String {
        KNOWLEDGE_EXTRACTION_TEMPLATE.replace("{conversation}", conversation)
    }

    /// Creates a quiz generation prompt
    #[must_use]
    pub fn quiz_generation(content: &str) -> String {
        QUIZ_GENERATION_TEMPLATE.replace("{content}", content)
    }

    /// Creates a summary prompt
    #[must_use]
    pub fn summary(content: &str) -> String {
        SUMMARY_TEMPLATE.replace("{content}", content)
    }

    /// Creates a warmup prompt with user context
    #[must_use]
    pub fn warmup(user_name: Option<&str>, time_of_day: &str) -> String {
        let mut vars = HashMap::new();
        vars.insert("user_name".to_string(), user_name.unwrap_or("朋友").to_string());
        vars.insert("time_of_day".to_string(), time_of_day.to_string());
        Self::format_template(WARMUP_PROMPT, &vars)
    }

    /// Creates a deep dive prompt with topic
    #[must_use]
    pub fn deep_dive(topic: &str, user_level: &str) -> String {
        let mut vars = HashMap::new();
        vars.insert("topic".to_string(), topic.to_string());
        vars.insert("user_level".to_string(), user_level.to_string());
        Self::format_template(DEEPDIVE_PROMPT, &vars)
    }

    /// Creates a review prompt with key points
    #[must_use]
    pub fn review(key_points: &[String], review_count: usize) -> String {
        let mut vars = HashMap::new();
        vars.insert("key_points".to_string(), key_points.join("\n"));
        vars.insert("review_count".to_string(), review_count.to_string());
        Self::format_template(REVIEW_PROMPT, &vars)
    }

    /// Creates a seed prompt with today's topics
    #[must_use]
    pub fn seed(today_topics: &[String], curiosity_points: usize) -> String {
        let mut vars = HashMap::new();
        vars.insert("today_topics".to_string(), today_topics.join(", "));
        vars.insert("curiosity_points".to_string(), curiosity_points.to_string());
        Self::format_template(SEED_PROMPT, &vars)
    }

    /// Creates a closing prompt with session summary
    #[must_use]
    pub fn closing(session_summary: &str) -> String {
        let mut vars = HashMap::new();
        vars.insert("session_summary".to_string(), session_summary.to_string());
        Self::format_template(CLOSING_PROMPT, &vars)
    }
}

/// ============================================================================
// Personality Configuration
// ============================================================================

/// Personality traits for the AI agent
#[derive(Debug, Clone)]
pub struct PersonalityConfig {
    /// Friendliness level (0.0 - 1.0)
    pub friendliness: f32,
    /// Enthusiasm level (0.0 - 1.0)
    pub enthusiasm: f32,
    /// Formalness level (0.0 = casual, 1.0 = formal)
    pub formalness: f32,
    /// Emoji usage frequency (0.0 - 1.0)
    pub emoji_frequency: f32,
    /// Response brevity (0.0 = verbose, 1.0 = brief)
    pub brevity: f32,
}

impl Default for PersonalityConfig {
    fn default() -> Self {
        Self {
            friendliness: 0.8,
            enthusiasm: 0.6,
            formalness: 0.2,
            emoji_frequency: 0.3,
            brevity: 0.7,
        }
    }
}

impl PersonalityConfig {
    /// Creates a gentle personality for bedtime
    #[must_use]
    pub fn bedtime() -> Self {
        Self {
            friendliness: 0.9,
            enthusiasm: 0.4,
            formalness: 0.1,
            emoji_frequency: 0.2,
            brevity: 0.8,
        }
    }

    /// Creates an energetic personality for active learning
    #[must_use]
    pub fn energetic() -> Self {
        Self {
            friendliness: 0.7,
            enthusiasm: 0.9,
            formalness: 0.2,
            emoji_frequency: 0.5,
            brevity: 0.5,
        }
    }

    /// Gets a description of the personality
    #[must_use]
    pub fn description(&self) -> &str {
        match (self.enthusiasm, self.formalness) {
            (0.0..=0.4, 0.0..=0.3) => "温和轻松",
            (0.0..=0.4, _) => "温和正式",
            (0.4..=0.7, 0.0..=0.3) => "友好亲切",
            (0.4..=0.7, _) => "友好专业",
            (_, 0.0..=0.3) => "热情活泼",
            _ => "专业严谨",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_manager_extraction() {
        let prompt = PromptManager::knowledge_extraction("Test conversation");
        assert!(prompt.contains("Test conversation"));
    }

    #[test]
    fn test_prompt_manager_quiz() {
        let prompt = PromptManager::quiz_generation("Test content");
        assert!(prompt.contains("Test content"));
    }

    #[test]
    fn test_prompt_category() {
        assert_eq!(
            PromptCategory::Consolidation.system_prompt(),
            DEEPDIVE_PROMPT
        );
    }

    #[test]
    fn test_stage_prompt() {
        assert_eq!(
            PromptManager::stage_prompt(SessionState::Warmup),
            WARMUP_PROMPT
        );
        assert_eq!(
            PromptManager::stage_prompt(SessionState::DeepDive),
            DEEPDIVE_PROMPT
        );
        assert_eq!(
            PromptManager::stage_prompt(SessionState::Review),
            REVIEW_PROMPT
        );
        assert_eq!(
            PromptManager::stage_prompt(SessionState::Seed),
            SEED_PROMPT
        );
        assert_eq!(
            PromptManager::stage_prompt(SessionState::Closing),
            CLOSING_PROMPT
        );
    }

    #[test]
    fn test_format_template() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("age".to_string(), "25".to_string());

        let template = "Hello {name}, you are {age} years old.";
        let result = PromptManager::format_template(template, &vars);

        assert_eq!(result, "Hello Alice, you are 25 years old.");
    }

    #[test]
    fn test_warmup_prompt() {
        let prompt = PromptManager::warmup(Some("张三"), "晚上");
        assert!(prompt.contains("张三"));
        assert!(prompt.contains("晚上"));
    }

    #[test]
    fn test_deep_dive_prompt() {
        let prompt = PromptManager::deep_dive("Rust编程", "初级");
        assert!(prompt.contains("Rust编程"));
        assert!(prompt.contains("初级"));
    }

    #[test]
    fn test_review_prompt() {
        let key_points = vec
!["变量".to_string(), "函数".to_string()]
;
        let prompt = PromptManager::review(&key_points, 2);
        assert!(prompt.contains("变量"));
        assert!(prompt.contains("函数"));
        assert!(prompt.contains("2"));
    }

    #[test]
    fn test_personality_default() {
        let config = PersonalityConfig::default();
        assert_eq!(config.friendliness, 0.8);
        assert_eq!(config.description(), "友好亲切");
    }

    #[test]
    fn test_personality_bedtime() {
        let config = PersonalityConfig::bedtime();
        assert!(config.friendliness > 0.8);
        assert!(config.enthusiasm < 0.5);
        assert_eq!(config.description(), "温和轻松");
    }

    #[test]
    fn test_personality_energetic() {
        let config = PersonalityConfig::energetic();
        assert!(config.enthusiasm > 0.8);
        assert_eq!(config.description(), "热情活泼");
    }
}
