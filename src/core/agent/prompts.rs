// ============================================================
// Agent Prompts
// ============================================================
//! Prompt templates and management for AI agents.
//!
//! This module defines system prompts and prompt templates
//! for different agent behaviors.

/// System prompt for the default learning companion
pub const DEFAULT_SYSTEM_PROMPT: &str = r#"You are NightMind, a gentle and thoughtful AI learning companion.

Your purpose is to help users review and consolidate their learning before sleep. You should:
- Listen carefully to what the user shares about their day
- Help identify key concepts they learned
- Suggest gentle review activities
- Encourage a positive, relaxed mindset for sleep

Always respond in a calm, supportive manner. Avoid complex topics close to bedtime."#;

/// System prompt for memory consolidation mode
pub const CONSOLIDATION_PROMPT: &str = r#"You are now helping with memory consolidation.

Guide the user through reviewing what they learned today:
1. Ask what topics they explored
2. Help identify the most important concepts
3. Suggest connections to previous knowledge
4. Create simple mental summaries

Keep responses brief and focused. The goal is reinforcement, not new learning."#;

/// System prompt for reflection mode
pub const REFLECTION_PROMPT: &str = r#"You are facilitating a gentle evening reflection.

Help the user:
- Reflect on their day's learning journey
- Acknowledge their progress
- Set simple intentions for tomorrow
- End with a positive, calming thought

Maintain a peaceful, contemplative tone."#;

/// Prompt template for knowledge extraction
pub const KNOWLEDGE_EXTRACTION_TEMPLATE: &str = r#"Based on the following conversation, extract key knowledge points:

Conversation:
{conversation}

Please identify:
1. Main topics discussed
2. Key concepts mentioned
3. Important facts or insights
4. Areas for further exploration

Format as a structured summary."#;

/// Prompt template for quiz generation
pub const QUIZ_GENERATION_TEMPLATE: &str = r#"Generate a brief quiz based on this content:

Content:
{content}

Create 3-5 simple questions to test understanding.
Include answers in a separate section.

Keep questions clear and age-appropriate."#;

/// Prompt template for summary generation
pub const SUMMARY_TEMPLATE: &str = r#"Create a concise summary of the following learning session:

Session Content:
{content}

Summary should include:
- Main topics covered (brief)
- Key takeaways (bullet points)
- Suggestions for review

Keep it under 200 words total."#;

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
            Self::Default => DEFAULT_SYSTEM_PROMPT,
            Self::Consolidation => CONSOLIDATION_PROMPT,
            Self::Reflection => REFLECTION_PROMPT,
            Self::Extraction => DEFAULT_SYSTEM_PROMPT,
            Self::Quiz => DEFAULT_SYSTEM_PROMPT,
        }
    }
}

/// Prompt template manager
pub struct PromptManager;

impl PromptManager {
    /// Gets the system prompt for a category
    #[must_use]
    pub const fn get_system_prompt(category: PromptCategory) -> &'static str {
        category.system_prompt()
    }

    /// Formats a template with the given variables
    ///
    /// # Errors
    ///
    /// Returns an error if template formatting fails.
    pub fn format_template(
        template: &str,
        variables: &std::collections::HashMap<String, String>,
    ) -> Result<String, String> {
        let mut result = template.to_string();

        for (key, value) in variables {
            let placeholder = format!("{{{key}}}");
            result = result.replace(&placeholder, value);
        }

        Ok(result)
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
            CONSOLIDATION_PROMPT
        );
    }
}
