# Rig Agent Integration Design

**Date:** 2025-02-26
**Status:** Approved
**Author:** Claude

---

## Overview

Replace the mock `NightMindAgent` implementation with actual Rig-powered OpenAI integration while maintaining the existing public API.

---

## Architecture

```
WebSocket → NightMindAgent.prompt() → rig::Agent → OpenAI API → Response
```

**Key Principle:** Use Rig's ecosystem directly, no extra abstraction layers.

---

## Component Details

### NightMindAgent Structure

```rust
pub struct NightMindAgent {
    /// The actual Rig agent - store it, don't rebuild it
    agent: rig::Agent<rig::openai::CompletionModel>,
    /// Config for rebuilding with different session states
    config: AgentConfig,
}
```

### AgentBuilder Implementation

```rust
impl AgentBuilder {
    pub fn build(self) -> Result<NightMindAgent> {
        // Build OpenAI client
        let client = rig::openai::Client::builder()
            .with_api_key(&self.config.api_key)
            .build()?;

        // Build the agent using Rig's builder
        let agent = client
            .completion_model_builder(&self.config.model)
            .preamble(&self.config.system_prompt)
            .temperature(self.config.temperature)
            .build();

        Ok(NightMindAgent { agent, config: self.config })
    }
}
```

### Method Delegation

```rust
impl NightMindAgent {
    pub async fn prompt(&self, message: &str) -> Result<String> {
        self.agent.prompt(message).await
            .map_err(|e| NightMindError::AiService(e.to_string()))
    }

    pub async fn prompt_stream(&self, message: &str) -> Result<impl Stream<Item = String>> {
        // TODO: Implement streaming
        self.prompt(message).await?;
        // Return stream wrapper
    }
}
```

---

## Configuration

### src/config/settings.rs

```rust
pub struct AiConfig {
    pub api_key: String,
    pub model: String,           // e.g., "gpt-4o-mini"
    pub temperature: f32,
    pub max_tokens: u32,
    pub provider: String,        // "openai" for now
}
```

### Environment Variables

```bash
OPENAI_API_KEY=sk-...
AI_MODEL=gpt-4o-mini
AI_TEMPERATURE=0.7
AI_MAX_TOKENS=2048
```

### Cargo.toml

```toml
rig-core = { version = "0.31", features = ["openai", "derive"] }
```

---

## Error Handling

### src/error.rs

```rust
pub enum NightMindError {
    AiService(String),           // LLM API errors
    AgentBuild(String),          // Agent construction failures
    // ... existing variants ...
}
```

---

## Testing Strategy

### Unit Tests

- Test builder with fake API key
- Test temperature clamping
- Test config defaults

### Integration Tests

`tests/agent_integration.rs`:

```rust
#[tokio::test]
#[ignore]  // Run with: cargo test -- --ignored
async fn test_real_openai_chat() {
    let settings = Settings::load().unwrap();
    let agent = AgentBuilder::from_settings(&settings)
        .unwrap()
        .build()
        .unwrap();

    let response = agent.prompt("Say hello").await.unwrap();
    assert!(!response.is_empty());
}
```

---

## Implementation Steps

1. **Update Cargo.toml** - Add rig-core with openai feature
2. **Modify src/core/agent/builder.rs** - Replace mock with Rig calls
3. **Update src/core/agent/mod.rs** - Re-export useful Rig types
4. **Verify WebSocket handler** - Should work without changes
5. **Add integration test** - Verify real OpenAI calls
6. **Clean up** - Remove unused dependencies

---

## Files to Modify

| File | Changes |
|------|---------|
| `Cargo.toml` | Add rig-core features |
| `src/core/agent/builder.rs` | Replace mock with Rig |
| `src/core/agent/mod.rs` | Re-exports |
| `src/config/settings.rs` | Verify AiConfig has needed fields |
| `src/error.rs` | Add AgentBuild variant |
| `tests/agent_integration.rs` | NEW: Integration test |

---

## Future Work (Out of Scope for This Phase)

- Streaming responses
- Tool system (ContentTransformer, FatigueDetector)
- RAG with vector store
- Multi-provider support (Claude, Gemini)
- Fallback/retry logic
