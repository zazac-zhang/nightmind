// ============================================================
// Agent Builder
// ============================================================
//! Builder pattern for constructing AI agents.
//!
//! This module provides a fluent API for configuring and building
//! AI agents with various capabilities.

use crate::config::Settings;
use crate::core::agent::prompts::{PersonalityConfig, PromptManager};
use crate::core::content::transformer::{VoiceFriendlyTransformer, ValidationResult};
use crate::error::{NightMindError, Result};
use crate::repository::models::session::SessionState;
use rig::client::{CompletionClient, ProviderClient};
use rig::completion::Prompt;
use rig::providers::openai;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// ============================================================================
// Agent Configuration
// ============================================================================

/// Configuration for agent personality and behavior
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Agent name
    pub name: String,
    /// System prompt defining agent behavior
    pub system_prompt: String,
    /// AI model to use
    pub model: String,
    /// Temperature setting (0.0 - 1.0)
    pub temperature: f32,
    /// Maximum tokens per response
    pub max_tokens: u32,
    /// Agent identifier
    pub agent_id: Uuid,
    /// Personality configuration
    pub personality: PersonalityConfig,
    /// Current session state
    pub session_state: SessionState,
    /// API key for the LLM provider
    pub api_key: String,
    /// Enable automatic content transformation for voice-friendly output
    pub enable_content_transform: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: "NightMind Agent".to_string(),
            system_prompt: PromptManager::system_prompt().to_string(),
            model: "gpt-4o-mini".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            agent_id: Uuid::new_v4(),
            personality: PersonalityConfig::default(),
            session_state: SessionState::Warmup,
            api_key: String::new(),
            enable_content_transform: true,
        }
    }
}

/// ============================================================================
// Agent Builder
// ============================================================================

/// Builder for creating configured agents
pub struct AgentBuilder {
    config: AgentConfig,
}

impl Default for AgentBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentBuilder {
    /// Creates a new agent builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: AgentConfig::default(),
        }
    }

    /// Creates a builder from application settings
    ///
    /// # Errors
    ///
    /// Returns an error if settings are invalid
    pub fn from_settings(settings: &Settings) -> Result<Self> {
        Ok(Self {
            config: AgentConfig {
                name: "NightMind Agent".to_string(),
                system_prompt: PromptManager::system_prompt().to_string(),
                model: settings.ai.model.clone(),
                temperature: settings.ai.temperature,
                max_tokens: settings.ai.max_tokens,
                agent_id: Uuid::new_v4(),
                personality: PersonalityConfig::bedtime(),
                session_state: SessionState::Warmup,
                api_key: settings.ai.api_key.clone(),
                enable_content_transform: true,
            },
        })
    }

    /// Sets the agent name
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    /// Sets the system prompt
    #[must_use]
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.config.system_prompt = prompt.into();
        self
    }

    /// Sets the model
    #[must_use]
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.config.model = model.into();
        self
    }

    /// Sets the temperature
    #[must_use]
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.config.temperature = temperature.clamp(0.0, 1.0);
        self
    }

    /// Sets the maximum tokens
    #[must_use]
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.config.max_tokens = max_tokens;
        self
    }

    /// Sets the agent ID
    #[must_use]
    pub fn with_agent_id(mut self, id: Uuid) -> Self {
        self.config.agent_id = id;
        self
    }

    /// Sets the personality configuration
    #[must_use]
    pub fn with_personality(mut self, personality: PersonalityConfig) -> Self {
        self.config.personality = personality;
        self
    }

    /// Sets the session state
    #[must_use]
    pub fn with_session_state(mut self, state: SessionState) -> Self {
        self.config.session_state = state;
        self
    }

    /// Sets the API key
    #[must_use]
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.config.api_key = api_key.into();
        self
    }

    /// Sets whether content transformation is enabled
    #[must_use]
    pub fn with_content_transform(mut self, enabled: bool) -> Self {
        self.config.enable_content_transform = enabled;
        self
    }

    /// Builds and returns the agent configuration
    #[must_use]
    pub fn build_config(self) -> AgentConfig {
        self.config
    }

    /// Builds the actual NightMindAgent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent cannot be initialized
    pub fn build(self) -> Result<NightMindAgent> {
        // Validate API key
        if self.config.api_key.is_empty() {
            return Err(NightMindError::AgentBuild(
                "API key is required".to_string()
            ));
        }

        // Set API key as environment variable for Rig
        std::env::set_var("OPENAI_API_KEY", &self.config.api_key);

        // Create OpenAI client
        let client = openai::Client::from_env();

        Ok(NightMindAgent {
            client,
            config: self.config,
        })
    }
}

/// ============================================================================
// NightMind Agent
// ============================================================================

/// NightMind AI agent for learning companion functionality
///
/// This agent wraps Rig's OpenAI client for easy interactions.
pub struct NightMindAgent {
    /// OpenAI client
    client: openai::Client,
    /// Configuration for building agents
    config: AgentConfig,
}

impl NightMindAgent {
    /// Creates a new NightMind agent with the given configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid
    pub fn new(config: AgentConfig) -> Result<Self> {
        if config.api_key.is_empty() {
            return Err(NightMindError::AgentBuild(
                "API key is required".to_string()
            ));
        }

        // Set API key as environment variable for Rig
        std::env::set_var("OPENAI_API_KEY", &config.api_key);

        // Create OpenAI client
        let client = openai::Client::from_env();

        Ok(Self {
            client,
            config,
        })
    }

    /// Creates an agent from application settings
    ///
    /// # Errors
    ///
    /// Returns an error if settings are invalid or agent cannot be created
    pub fn from_settings(settings: &Settings) -> Result<Self> {
        let builder = AgentBuilder::from_settings(settings)?;
        builder.build()
    }

    /// Returns a reference to the agent's configuration
    #[must_use]
    pub const fn config(&self) -> &AgentConfig {
        &self.config
    }

    /// Gets the agent ID
    #[must_use]
    pub const fn agent_id(&self) -> Uuid {
        self.config.agent_id
    }

    /// Updates the session state and adjusts prompt accordingly
    pub fn update_session_state(&mut self, state: SessionState) {
        self.config.session_state = state;
    }

    /// Gets the current full prompt including stage-specific context
    #[must_use]
    pub fn current_prompt(&self) -> String {
        PromptManager::build_prompt(self.config.session_state, None)
    }

    /// Sends a prompt and gets a response
    ///
    /// # Arguments
    ///
    /// * `message` - User message to send
    ///
    /// # Returns
    ///
    /// The agent's response
    ///
    /// # Errors
    ///
    /// Returns an error if the API request fails
    pub async fn prompt(&self, message: &str) -> Result<String> {
        tracing::info!("Agent prompt: {}", message);
        tracing::info!("Using model: {}", self.config.model);
        tracing::info!("Session state: {:?}", self.config.session_state);

        // Build agent with current config
        let agent = self.client
            .agent(&self.config.model)
            .preamble(&self.config.system_prompt)
            .temperature(f64::from(self.config.temperature))
            .build();

        // Use Rig agent to get response
        let response = agent
            .prompt(message)
            .await
            .map_err(|e| NightMindError::AiService(e.to_string()))?;

        Ok(response)
    }

    /// Sends a prompt with automatic content transformation for voice-friendly output
    ///
    /// This method checks if the agent's response contains code, formulas, or lists
    /// that are not suitable for voice reading, and automatically transforms them
    /// into voice-friendly metaphors and explanations.
    ///
    /// # Arguments
    ///
    /// * `message` - User message to send
    ///
    /// # Returns
    ///
    /// The agent's response, transformed if needed for voice output
    ///
    /// # Errors
    ///
    /// Returns an error if the API request or transformation fails
    pub async fn prompt_with_transform(&self, message: &str) -> Result<String> {
        // First, get the raw response from the agent
        let raw_response = self.prompt(message).await?;

        // Check if content transformation is enabled
        if !self.config.enable_content_transform {
            return Ok(raw_response);
        }

        // Check if transformation is needed
        let validation = VoiceFriendlyTransformer::validate_voice_friendly(&raw_response);

        if validation.is_voice_friendly {
            // Content is already voice-friendly, return as-is
            tracing::debug!("Content is voice-friendly (score: {}), no transformation needed", validation.score);
            return Ok(raw_response);
        }

        // Content needs transformation
        tracing::info!(
            "Content needs transformation (score: {}, issues: {:?})",
            validation.score,
            validation.issues
        );

        // Create transformer and transform with AI
        let transformer = VoiceFriendlyTransformer::new();
        let result = transformer.transform_with_agent(&raw_response, self).await;

        if !result.warnings.is_empty() {
            tracing::warn!("Transformation warnings: {:?}", result.warnings);
        }

        Ok(result.content)
    }

    /// Sends a prompt with custom context variables
    ///
    /// # Arguments
    ///
    /// * `message` - User message
    /// * `context_vars` - Additional context for the prompt
    ///
    /// # Returns
    ///
    /// The agent's response
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails
    pub async fn prompt_with_context(
        &self,
        message: &str,
        context_vars: &HashMap<String, String>,
    ) -> Result<String> {
        // Build prompt with context
        let full_prompt = PromptManager::build_prompt(
            self.config.session_state,
            Some(context_vars)
        );

        tracing::info!("Agent prompt with context: {}", full_prompt);
        tracing::info!("User message: {}", message);

        // Build agent with context-aware preamble
        let agent = self.client
            .agent(&self.config.model)
            .preamble(&full_prompt)
            .temperature(f64::from(self.config.temperature))
            .build();

        // Get response
        let response = agent
            .prompt(message)
            .await
            .map_err(|e| NightMindError::AiService(e.to_string()))?;

        Ok(response)
    }

    /// Streams a response
    ///
    /// # Arguments
    ///
    /// * `message` - User message
    ///
    /// # Returns
    ///
    /// A stream of response chunks
    ///
    /// # Errors
    ///
    /// Returns an error if streaming fails
    pub async fn prompt_stream(
        &self,
        message: &str,
    ) -> Result<tokio_stream::wrappers::ReceiverStream<String>> {
        // Get the full response
        let response = self.prompt(message).await?;

        // Split into chunks for streaming effect
        let chunks: Vec<String> = response
            .chars()
            .collect::<Vec<char>>()
            .chunks(10)
            .map(|c| c.iter().collect())
            .collect();

        let (tx, rx) = tokio::sync::mpsc::channel(chunks.len());

        tokio::spawn(async move {
            for chunk in chunks {
                if tx.send(chunk).await.is_err() {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            }
        });

        Ok(tokio_stream::wrappers::ReceiverStream::new(rx))
    }

    /// Creates a chat session with conversation history support
    ///
    /// # Arguments
    ///
    /// * `history` - Previous conversation messages
    /// * `new_message` - New user message
    ///
    /// # Returns
    ///
    /// The agent's response considering conversation history
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails
    pub async fn chat_with_history(
        &self,
        history: &[(Role, String)],
        new_message: &str,
    ) -> Result<String> {
        // Build chat prompt with history
        let mut conversation = String::new();
        for (role, msg) in history {
            conversation.push_str(&format!("{}: {}\n", role.as_str(), msg));
        }
        conversation.push_str(&format!("user: {}", new_message));

        self.prompt(&conversation).await
    }
}

/// Message role for conversation history
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    /// System message
    System,
    /// User message
    User,
    /// Assistant message
    Assistant,
}

impl Role {
    /// Returns the string representation of the role
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}

/// ============================================================================
// Agent Manager
// ============================================================================

/// Manager for multiple agent instances
///
/// Handles creation and management of agent instances for different sessions.
pub struct AgentManager {
    /// Application settings
    settings: Settings,
    /// Cached agent instances
    agents: HashMap<Uuid, Arc<NightMindAgent>>,
}

impl AgentManager {
    /// Creates a new agent manager
    ///
    /// # Errors
    ///
    /// Returns an error if settings cannot be loaded
    pub async fn new() -> Result<Self> {
        let settings = Settings::load()
            .map_err(|e| NightMindError::Config(e.to_string()))?;
        Ok(Self {
            settings,
            agents: HashMap::new(),
        })
    }

    /// Creates a new agent instance
    ///
    /// # Errors
    ///
    /// Returns an error if agent cannot be created
    pub fn create_agent(&mut self) -> Result<Arc<NightMindAgent>> {
        let builder = AgentBuilder::from_settings(&self.settings)?;
        let agent = Arc::new(builder.build()?);
        let id = agent.agent_id();
        self.agents.insert(id, Arc::clone(&agent));
        Ok(agent)
    }

    /// Gets an existing agent by ID
    #[must_use]
    pub fn get_agent(&self, id: Uuid) -> Option<Arc<NightMindAgent>> {
        self.agents.get(&id).cloned()
    }

    /// Removes an agent from the manager
    pub fn remove_agent(&mut self, id: Uuid) -> Option<Arc<NightMindAgent>> {
        self.agents.remove(&id)
    }

    /// Returns the number of active agents
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.agents.len()
    }

    /// Clears all cached agents
    pub fn clear(&mut self) {
        self.agents.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_builder() {
        let config = AgentBuilder::new()
            .with_name("Test Agent")
            .with_temperature(0.5)
            .with_max_tokens(1024)
            .build_config();

        assert_eq!(config.name, "Test Agent");
        assert_eq!(config.temperature, 0.5);
        assert_eq!(config.max_tokens, 1024);
    }

    #[test]
    fn test_temperature_clamping() {
        let config = AgentBuilder::new()
            .with_temperature(1.5)
            .build_config();

        assert_eq!(config.temperature, 1.0);
    }

    #[test]
    fn test_role_as_str() {
        assert_eq!(Role::System.as_str(), "system");
        assert_eq!(Role::User.as_str(), "user");
        assert_eq!(Role::Assistant.as_str(), "assistant");
    }

    #[test]
    fn test_current_prompt() {
        let config = AgentBuilder::new()
            .with_session_state(SessionState::Review)
            .with_api_key("test-key")
            .build_config();

        let agent = NightMindAgent::new(config);

        assert!(agent.is_ok());
        let agent = agent.unwrap();
        let prompt = agent.current_prompt();
        assert!(prompt.contains("NightMind"));
    }

    #[test]
    fn test_agent_config_default() {
        let config = AgentConfig::default();
        assert_eq!(config.name, "NightMind Agent");
        assert!(!config.system_prompt.is_empty());
        assert_eq!(config.session_state, SessionState::Warmup);
    }

    #[test]
    fn test_agent_without_api_key() {
        let config = AgentConfig::default();
        let agent = NightMindAgent::new(config);
        assert!(agent.is_err());
    }

    #[tokio::test]
    #[ignore]  // Run with: cargo test -- --ignored
    async fn test_prompt_response() {
        let config = AgentBuilder::new()
            .with_api_key("test-key")
            .build_config();

        let agent = NightMindAgent::new(config).unwrap();
        let response = agent.prompt("你好").await;
        assert!(response.is_ok());
        assert!(response.unwrap().contains("你好"));
    }

    #[test]
    fn test_content_transform_enabled_by_default() {
        let config = AgentConfig::default();
        assert!(config.enable_content_transform);
    }

    #[test]
    fn test_content_transform_can_be_disabled() {
        let config = AgentBuilder::new()
            .with_content_transform(false)
            .build_config();

        assert!(!config.enable_content_transform);
    }

    #[test]
    fn test_voice_friendly_validation() {
        // Test that validation correctly identifies code content
        let code_content = "function compute() { return x + y; }";
        let validation = VoiceFriendlyTransformer::validate_voice_friendly(code_content);

        assert!(!validation.is_voice_friendly);
        assert!(validation.score <= 70);
        assert!(!validation.issues.is_empty());
    }

    #[test]
    fn test_voice_friendly_validation_passes() {
        // Test that validation passes for simple text
        let simple_content = "This is simple conversational text.";
        let validation = VoiceFriendlyTransformer::validate_voice_friendly(simple_content);

        assert!(validation.is_voice_friendly);
        assert!(validation.score > 70);
        assert!(validation.issues.is_empty());
    }

    #[tokio::test]
    #[ignore]  // Run with: cargo test -- --ignored (requires OPENAI_API_KEY)
    async fn test_prompt_with_transform() {
        let config = AgentBuilder::new()
            .with_api_key("test-key")
            .with_content_transform(true)
            .build_config();

        let agent = NightMindAgent::new(config).unwrap();

        // Test with a prompt that might return code
        let response = agent.prompt_with_transform("解释一下Python的装饰器模式").await;
        assert!(response.is_ok());

        let content = response.unwrap();
        // The response should be transformed to be voice-friendly
        // (This is a basic check - in a real test with API key, we'd verify the transformation)
        assert!(!content.is_empty());
    }
}
