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
    println!("Response with history: {}", response);
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
        result.push_str(&chunk);
    }

    assert!(!result.is_empty());
    println!("Streamed response: {}", result);
}

#[tokio::test]
#[ignore]
async fn test_agent_with_context() {
    let settings = Settings::load().unwrap();
    let agent = AgentBuilder::from_settings(&settings)
        .unwrap()
        .build()
        .unwrap();

    let mut context = std::collections::HashMap::new();
    context.insert("user_name".to_string(), "Bob".to_string());
    context.insert("time_of_day".to_string(), "morning".to_string());

    let response = agent
        .prompt_with_context("Hello!", &context)
        .await
        .unwrap();

    assert!(!response.is_empty());
    println!("Response with context: {}", response);
}

#[tokio::test]
#[ignore]
async fn test_chinese_prompt() {
    let settings = Settings::load().unwrap();
    let agent = AgentBuilder::from_settings(&settings)
        .unwrap()
        .build()
        .unwrap();

    let response = agent
        .prompt("用中文说：你好，世界！")
        .await
        .expect("Failed to get Chinese response");

    assert!(!response.is_empty());
    assert!(
        response.contains("你好") || response.contains("世界"),
        "Response should contain Chinese greeting"
    );

    println!("Chinese response: {}", response);
}
