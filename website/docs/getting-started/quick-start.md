---
id: quick-start
title: Quick Start
sidebar_label: Quick Start
---

# Quick Start

This page gets you from zero to a working request as fast as possible. For a deeper walkthrough of the lifecycle and events, see [Your First Session](/docs/getting-started/your-first-session).

## 1. Add dependencies

```toml title="Cargo.toml"
[dependencies]
copilot-sdk = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## 2. Send a message and collect the reply

The simplest pattern uses [`send_and_collect`](/docs/api/session#wait-helpers), which sends a prompt and returns the assistant's full text response as a `String`.

```rust title="src/main.rs"
use copilot_sdk::{Client, SessionConfig};

#[tokio::main]
async fn main() -> copilot_sdk::Result<()> {
    // Build and start the client (spawns and connects to the CLI).
    let client = Client::builder().build()?;
    client.start().await?;

    // Create a session with default configuration.
    let session = client.create_session(SessionConfig::default()).await?;

    // Send a prompt and wait for the complete response.
    let response = session.send_and_collect("What is the capital of France?", None).await?;
    println!("{response}");

    // Shut down cleanly.
    client.stop().await;
    Ok(())
}
```

```bash
cargo run
```

## 3. Stream the response as it arrives

For token-by-token output, subscribe to the session's [event stream](/docs/core-concepts/events) and react to events as they arrive:

```rust
use copilot_sdk::{Client, SessionConfig, SessionEventData};

#[tokio::main]
async fn main() -> copilot_sdk::Result<()> {
    let client = Client::builder().build()?;
    client.start().await?;

    let session = client.create_session(SessionConfig::default()).await?;
    let mut events = session.subscribe();

    session.send("Write a haiku about Rust.").await?;

    while let Ok(event) = events.recv().await {
        match &event.data {
            SessionEventData::AssistantMessageDelta(delta) => {
                print!("{}", delta.delta_content);
            }
            SessionEventData::SessionIdle(_) => break,
            _ => {}
        }
    }

    client.stop().await;
    Ok(())
}
```

## 4. Configure the session

[`SessionConfig`](/docs/api/types#sessionconfig) controls model, streaming, client name, tools, and much more:

```rust
let session = client.create_session(SessionConfig {
    model: Some("gpt-4.1".into()),
    streaming: true,
    client_name: Some("my-app".into()),
    ..Default::default()
}).await?;
```

## Common next steps

| I want to... | See |
|--------------|-----|
| Understand the event loop | [Your First Session](/docs/getting-started/your-first-session) |
| Register a custom tool | [Tools](/docs/guides/tools) |
| Switch models or reasoning effort | [Models](/docs/guides/models) |
| Use plan or autopilot mode | [Modes](/docs/guides/modes) |
| Approve or deny tool calls | [Permissions](/docs/guides/permissions) |
| Use my own provider key | [BYOK](/docs/guides/byok) |
| Connect an MCP server | [MCP Servers](/docs/guides/mcp) |
