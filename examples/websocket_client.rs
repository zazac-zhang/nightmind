// ============================================================
// WebSocket Client Example
// ============================================================
//! Example WebSocket client for NightMind.

use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures::{SinkExt, StreamExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("NightMind WebSocket Client Example");
    println!("==================================");
    println!();
    println!("This example demonstrates WebSocket client usage.");
    println!("TODO: Implement full WebSocket client example.");
    println!();
    println!("To connect to the server:");
    println!("  1. Start the NightMind server: cargo run");
    println!("  2. Connect to ws://localhost:8080/api/ws");
    println!();
    println!("WebSocket message format:");
    println!("  {{\"type\":\"textInput\",\"data\":{{\"text\":\"Hello\",\"session_id\":\"...\"}}}}");

    Ok(())
}
