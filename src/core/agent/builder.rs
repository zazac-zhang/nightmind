// ============================================================
// Agent Builder
// ============================================================
//! Builder pattern for constructing AI agents.
//!
//! This module provides a fluent API for configuring and building
//! AI agents with various capabilities.

use uuid::Uuid;

/// Configuration for agent personality and behavior
#[derive(Debug, Clone)]
pub struct AgentConfig {
    /// Agent name
    pub name: String,
    /// System prompt defining agent behavior
    pub system_prompt: String,
    /// Temperature setting (0.0 - 1.0)
    pub temperature: f32,
    /// Maximum tokens per response
    pub max_tokens: u32,
    /// Agent identifier
    pub agent_id: Uuid,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            name: "NightMind Agent".to_string(),
            system_prompt: "You are a helpful AI learning companion.".to_string(),
            temperature: 0.7,
            max_tokens: 2048,
            agent_id: Uuid::new_v4(),
        }
    }
}

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

    /// Builds and returns the agent configuration
    #[must_use]
    pub fn build(self) -> AgentConfig {
        self.config
    }
}

/// Constructed AI agent
pub struct Agent {
    /// Agent configuration
    pub config: AgentConfig,
}

impl Agent {
    /// Creates a new agent from the given configuration
    #[must_use]
    pub fn new(config: AgentConfig) -> Self {
        Self { config }
    }

    /// Returns a reference to the agent's configuration
    #[must_use]
    pub const fn config(&self) -> &AgentConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_builder() {
        let agent = AgentBuilder::new()
            .with_name("Test Agent")
            .with_temperature(0.5)
            .with_max_tokens(1024)
            .build();

        assert_eq!(agent.name, "Test Agent");
        assert_eq!(agent.temperature, 0.5);
        assert_eq!(agent.max_tokens, 1024);
    }

    #[test]
    fn test_temperature_clamping() {
        let agent = AgentBuilder::new()
            .with_temperature(1.5)
            .build();

        assert_eq!(agent.temperature, 1.0);
    }
}
