# Rig Agent Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Integrate Rig framework to replace mock NightMindAgent with real OpenAI-powered AI agent.

**Architecture:** Use Rig's OpenAI client directly (no wrapper abstractions). Refactor existing NightMindAgent struct in place, keeping public API but replacing internal implementation with Rig-powered agent. The WebSocket handler and other callers require no changes.

**Tech Stack:** Rust, Rig 0.31 (with openai feature), OpenAI API, Tokio async runtime

**Design Document:** `docs/plans/2025-02-26-rig-agent-integration-design.md`

---

## Prerequisites

**Before starting:**
1. Ensure `OPENAI_API_KEY` is set in `.env` file
2. Run `make docker-up` to start PostgreSQL and Redis
3. Run `cargo build` to verify current state compiles

---

## Task 1: Update Cargo.toml Dependencies

**Files:**
- Modify: `Cargo.toml:63`

**Step 1: Add rig-core features**

Find the rig-core dependency line and add the `openai` and `derive` features:

```toml
rig-core = { version = "0.31", features = ["openai", "derive"] }
```

**Step 2: Verify changes**

Run: `cargo check`

Expected: No errors (dependency changes only)

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "feat(rig): add openai and derive features to rig-core"
```

---

## Task 2: Add AgentBuild Error Variant

**Files:**
- Modify: `src/error.rs`

**Step 1: Read current error definitions**

Run: `cat src/error.rs`

Observe: The existing `NightMindError` enum structure

**Step 2: Add AgentBuild variant**

Find the `NightMindError` enum and add the new variant:

```rust
pub enum NightMindError {
    // ... existing variants ...
    AgentBuild(String),  // NEW: Agent construction failures
    // ... existing variants ...
}
```

**Step 3: Verify compilation**

Run: `cargo check`

Expected: No errors (we added an unused variant, which is fine)

**Step 4: Commit**

```bash
git add src/error.rs
git commit -m "feat(error): add AgentBuild error variant"
```

---

## Task 3: Update NightMindAgent Internal Structure

**Files:**
- Modify: `src/core/agent/builder.rs:200-206`

**Step 1: Read current NightMindAgent struct**

Run: `cat src/core/agent/builder.rs | grep -A 10 "pub struct NightMindAgent"`

Observe: Current struct uses `reqwest::Client`

**Step 2: Replace the struct definition**

Find the `NightMindAgent` struct (around line 198) and replace:

```rust
/// NightMind AI agent for learning companion functionality
///
/// This agent wraps Rig's Agent for OpenAI integration.
pub struct NightMindAgent {
    /// The actual Rig agent
    agent: rig::Agent<rig::openai::CompletionModel>,
    /// Configuration for rebuilding with different states
    config: AgentConfig,
}
```

**Step 3: Verify compilation error**

Run: `cargo check`

Expected: Error about `reqwest::Client` no longer being used, and `agent` field type mismatch

**Step 4: Commit**

```bash
git add src/core/agent/builder.rs
git commit -m "refactor(agent): update NightMindAgent to use Rig agent"
```

---

## Task 4: Add Rig Imports

**Files:**
- Modify: `src/core/agent/builder.rs:1-15`

**Step 1: Read current imports**

Run: `head -20 src/core/agent/builder.rs`

Observe: Current import statements

**Step 2: Add Rig imports**

Add after the existing imports:

```rust
use rig::agent::Agent;
use rig::openai::{Client, CompletionModelBuilder};
```

**Step 3: Verify compilation**

Run: `cargo check`

Expected: May still have errors (we haven't fixed the implementation yet)

**Step 4: Commit**

```bash
git add src/core/agent/builder.rs
git commit -m "chore(agent): add Rig imports"
```

---

## Task 5: Update AgentBuilder::build() Method

**Files:**
- Modify: `src/core/agent/builder.rs:176-191`

**Step 1: Read current build method**

Run: `sed -n '176,191p' src/core/agent/builder.rs`

Observe: Current placeholder implementation

**Step 2: Replace the build() method**

Replace the entire `build()` method:

```rust
    /// Builds the actual NightMindAgent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent cannot be initialized
    pub fn build(self) -> Result<NightMindAgent> {
        // Create OpenAI client
        let client = Client::builder()
            .with_api_key(&self.config.api_key)
            .build()
            .map_err(|e| NightMindError::AgentBuild(e.to_string()))?;

        // Build the agent using Rig's builder pattern
        let agent = client
            .completion_model_builder(&self.config.model)
            .preamble(&self.config.system_prompt)
            .temperature(self.config.temperature)
            .build();

        Ok(NightMindAgent {
            agent,
            config: self.config,
        })
    }
```

**Step 3: Verify compilation**

Run: `cargo check`

Expected: Errors about the `prompt()` method implementation (next task)

**Step 4: Commit**

```bash
git add src/core/agent/builder.rs
git commit -m "feat(agent): implement AgentBuilder::build() with Rig"
```

---

## Task 6: Update NightMindAgent::prompt() Method

**Files:**
- Modify: `src/core/agent/builder.rs:270-286`

**Step 1: Read current prompt method**

Run: `sed -n '270,286p' src/core/agent/builder.rs`

Observe: Current mock implementation

**Step 2: Replace the prompt() method**

Replace the entire `prompt()` method:

```rust
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
        self.agent
            .prompt(message)
            .await
            .map_err(|e| NightMindError::AiService(e.to_string()))
    }
```

**Step 3: Verify compilation**

Run: `cargo check`

Expected: May have errors in `prompt_with_context()` or `chat_with_history()` methods

**Step 4: Commit**

```bash
git add src/core/agent/builder.rs
git commit -m "feat(agent): implement prompt() using Rig agent"
```

---

## Task 7: Update prompt_with_context() Method

**Files:**
- Modify: `src/core/agent/builder.rs:299-318`

**Step 1: Read current method**

Run: `sed -n '299,318p' src/core/agent/builder.rs`

Observe: Current mock implementation

**Step 2: Replace prompt_with_context() method**

Replace the entire method with simplified version (Rig handles context internally):

```rust
    /// Sends a prompt with context variables
    ///
    /// # Arguments
    ///
    /// * `message` - User message
    /// * `context_vars` - Additional context (currently not used, for future RAG)
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
        _context_vars: &HashMap<String, String>,
    ) -> Result<String> {
        // TODO: Integrate RAG context injection
        // For now, delegate to regular prompt
        self.prompt(message).await
    }
```

**Step 3: Verify compilation**

Run: `cargo check`

Expected: May have errors in `chat_with_history()` method

**Step 4: Commit**

```bash
git add src/core/agent/builder.rs
git commit -m "feat(agent): implement prompt_with_context()"
```

---

## Task 8: Update chat_with_history() Method

**Files:**
- Modify: `src/core/agent/builder.rs:372-389`

**Step 1: Read current method**

Run: `sed -n '372,389p' src/core/agent/builder.rs`

Observe: Current mock implementation

**Step 2: Replace chat_with_history() method**

Replace with Rig-based implementation:

```rust
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
        // Build conversation string from history
        let mut conversation = String::new();
        for (role, msg) in history {
            conversation.push_str(&format!("{}: {}\n", role.as_str(), msg));
        }
        conversation.push_str(&format!("user: {}", new_message));

        self.agent
            .prompt(&conversation)
            .await
            .map_err(|e| NightMindError::AiService(e.to_string()))
    }
```

**Step 3: Verify compilation**

Run: `cargo check`

Expected: May have errors in `prompt_stream()` method

**Step 4: Commit**

```bash
git add src/core/agent/builder.rs
git commit -m "feat(agent): implement chat_with_history() with Rig"
```

---

## Task 9: Update prompt_stream() Method

**Files:**
- Modify: `src/core/agent/builder.rs:329-360`

**Step 1: Read current method**

Run: `sed -n '329,360p' src/core/agent/builder.rs`

Observe: Current mock streaming implementation

**Step 2: Replace with placeholder for real streaming**

Replace with a note that streaming will be added later:

```rust
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
    ///
    /// # Note
    ///
    /// Streaming will be implemented in a follow-up task.
    /// For now, this falls back to non-streaming prompt.
    pub async fn prompt_stream(
        &self,
        message: &str,
    ) -> Result<tokio_stream::wrappers::ReceiverStream<String>> {
        // For now, get the full response and chunk it
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
```

**Step 3: Verify compilation**

Run: `cargo check`

Expected: No errors in builder.rs now

**Step 4: Commit**

```bash
git add src/core/agent/builder.rs
git commit -m "feat(agent): implement prompt_stream() (mock streaming)"
```

---

## Task 10: Remove build_simple() Mock Method

**Files:**
- Modify: `src/core/agent/builder.rs:184-191`

**Step 1: Remove build_simple()**

Find and remove the `build_simple()` method (it's no longer needed with real Rig integration):

```rust
    /// Builds a simple agent without API key validation
    #[must_use]
    pub fn build_simple(self) -> NightMindAgent {
        // DELETE THIS METHOD - no longer needed
    }
```

**Step 2: Update WebSocket handler call**

The websocket.rs file calls `build_simple()`. We need to check if it still exists:

Run: `grep -n "build_simple" src/api/websocket.rs`

If found, update it to use `build()` instead:

```rust
// In src/api/websocket.rs, around line 51
let agent = AgentBuilder::from_settings(&state.settings)
    .await?
    .build()?;  // Changed from build_simple()
```

**Step 3: Verify compilation**

Run: `cargo check`

Expected: No errors

**Step 4: Commit**

```bash
git add src/core/agent/builder.rs src/api/websocket.rs
git commit -m "refactor(agent): remove build_simple() mock method"
```

---

## Task 11: Update Module Exports

**Files:**
- Modify: `src/core/agent/mod.rs`

**Step 1: Read current exports**

Run: `cat src/core/agent/mod.rs`

**Step 2: Add Rig re-exports**

Add re-exports for commonly used Rig types:

```rust
// ============================================================
// Agent System Module
// ============================================================
//! AI agent system for learning companion functionality.
//!
//! This module provides the agent builder, prompts, and tools
//! for the NightMind AI learning companion.

pub mod builder;
pub mod prompts;
pub mod tools;

// Re-export Rig types for convenience
pub use rig::{
    agent::Agent,
    completion::PromptError,
};

// Re-export common types
pub use builder::{
    AgentBuilder, AgentConfig, AgentManager, NightMindAgent,
    Role,
};
pub use prompts::{
    PersonalityConfig, PromptCategory, PromptManager,
    SYSTEM_PROMPT,
    WARMUP_PROMPT, DEEPDIVE_PROMPT, REVIEW_PROMPT, SEED_PROMPT, CLOSING_PROMPT,
};
```

**Step 3: Verify compilation**

Run: `cargo check`

Expected: No errors

**Step 4: Commit**

```bash
git add src/core/agent/mod.rs
git commit -m "chore(agent): re-export Rig types for convenience"
```

---

## Task 12: Create Integration Test

**Files:**
- Create: `tests/agent_integration.rs`

**Step 1: Write the integration test**

Create new test file:

```rust
// ============================================================
// Rig Agent Integration Tests
// ============================================================
//! Integration tests for the Rig-powered NightMind agent.
//!
//! These tests require a valid OPENAI_API_KEY in the environment.

use nightmind::config::Settings;
use nightmind::core::agent::AgentBuilder;

#[tokio::test]
#[ignore]  // Run with: cargo test -- --ignored
async fn test_real_openai_chat() {
    // Load settings (requires .env with OPENAI_API_KEY)
    let settings = Settings::load()
        .expect("Failed to load settings - ensure .env exists");

    // Build the agent
    let agent = AgentBuilder::from_settings(&settings)
        .expect("Failed to create builder")
        .build()
        .expect("Failed to build agent");

    // Test simple prompt
    let response = agent
        .prompt("Say 'Hello, NightMind!' in exactly those words.")
        .await
        .expect("Failed to get response");

    // Verify response
    assert!(!response.is_empty(), "Response should not be empty");
    assert!(
        response.to_lowercase().contains("hello"),
        "Response should contain 'Hello'"
    );

    println!("Agent response: {}", response);
}

#[tokio::test]
#[ignore]
async fn test_agent_with_history() {
    let settings = Settings::load().unwrap();
    let agent = AgentBuilder::from_settings(&settings)
        .unwrap()
        .build()
        .unwrap();

    let history = vec![
        (nightmind::core::agent::Role::User, "My name is Alice.".to_string()),
        (nightmind::core::agent::Role::Assistant, "Nice to meet you, Alice!".to_string()),
    ];

    let response = agent
        .chat_with_history(&history, "What's my name?")
        .await
        .unwrap();

    assert!(response.to_lowercase().contains("alice"));
}

#[tokio::test]
#[ignore]
async fn test_prompt_stream() {
    let settings = Settings::load().unwrap();
    let agent = AgentBuilder::from_settings(&settings)
        .unwrap()
        .build()
        .unwrap();

    let mut stream = agent.prompt_stream("Count from 1 to 3").await.unwrap();

    let mut result = String::new();
    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        result.push_str(&chunk.unwrap());
    }

    assert!(!result.is_empty());
    println!("Streamed response: {}", result);
}
```

**Step 2: Verify test file exists**

Run: `ls -la tests/agent_integration.rs`

Expected: File exists

**Step 3: Verify tests compile**

Run: `cargo test --test agent_integration --no-run`

Expected: No compilation errors

**Step 4: Commit**

```bash
git add tests/agent_integration.rs
git commit -m "test(agent): add Rig agent integration tests"
```

---

## Task 13: Run Integration Tests (Requires API Key)

**Step 1: Ensure .env has valid OPENAI_API_KEY**

Run: `grep OPENAI_API_KEY .env`

Expected: Shows valid API key

**Step 2: Run the integration test**

Run: `cargo test --test agent_integration -- --ignored`

Expected output:
```
test test_real_openai_chat ... ok
test test_agent_with_history ... ok
test test_prompt_stream ... ok
```

**Step 3: If tests pass, verify agent works end-to-end**

Run the server:

```bash
cargo run
```

In another terminal, test WebSocket connection (or use provided frontend if available).

**Step 4: Commit test results (optional)**

```bash
echo "Integration tests passed on $(date)" > docs/test-results/agent-integration-$(date +%Y%m%d).txt
git add docs/test-results/
git commit -m "test(agent): record integration test results"
```

---

## Task 14: Clean Up Unused Code

**Step 1: Find unused reqwest usage**

Run: `grep -r "reqwest" src/ --include="*.rs"`

Expected: May find old references to remove

**Step 2: Remove any unused imports from builder.rs**

Check for these imports that may no longer be needed:
- `reqwest::Client`
- `futures_util` (if only used for mock streaming)

**Step 3: Run cargo clippy to find unused code**

Run: `cargo clippy -- -W unused_imports`

Fix any warnings found.

**Step 4: Commit cleanup**

```bash
git add -A
git commit -m "chore: remove unused imports and code after Rig integration"
```

---

## Task 15: Update Documentation

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Update Agent System section**

Find the Agent System section in CLAUDE.md and update to mention Rig integration is complete.

**Step 2: Add notes about running tests**

Add a note about integration tests requiring OPENAI_API_KEY.

**Step 3: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md with Rig integration notes"
```

---

## Verification Steps

After completing all tasks:

**1. Full test suite**

Run: `cargo test --all`

Expected: All tests pass

**2. Clippy check**

Run: `cargo clippy --all-targets`

Expected: No warnings (or only acceptable ones)

**3. Format check**

Run: `cargo fmt --check`

Expected: No formatting differences

**4. Run the server**

Run: `cargo run`

Expected: Server starts successfully

**5. Test WebSocket endpoint**

Connect to `ws://localhost:8080/api/ws` and send a message.

Expected: Real AI response from OpenAI

---

## Post-Implementation Notes

**What's Working:**
- Real OpenAI chat via Rig
- Non-streaming prompts
- Chat with history
- Configuration-based model selection

**What's Still TODO (future work):**
- True streaming responses (currently mock chunked)
- Tool system integration
- RAG with vector store
- Multi-provider fallback
- Advanced prompt management

**Next Steps:**
1. Implement true streaming with Rig's streaming API
2. Add tool system (ContentTransformer, FatigueDetector)
3. Integrate Qdrant for RAG

---

## Troubleshooting

**"API key invalid" error:**
- Verify `.env` file exists in project root
- Check `OPENAI_API_KEY` is set correctly
- Ensure key has OpenAI API access

**"Model not found" error:**
- Verify `AI_MODEL` in config (default should be `gpt-4o-mini`)
- Check model is available in your OpenAI account

**Compilation errors with Rig:**
- Ensure rig-core version is 0.31 or compatible
- Check that `openai` feature is enabled
- Run `cargo update` if needed

**Integration tests ignored:**
- Run with `cargo test -- --ignored`
- Ensure `.env` has valid API key
