---
id: intro
title: Introduction
sidebar_label: Introduction
description: A Rust SDK for the GitHub Copilot CLI agent runtime.
---

# Copilot SDK for Rust

`copilot-sdk` is a Rust library for driving the **GitHub Copilot CLI agent runtime** programmatically. It speaks the CLI's JSON-RPC protocol over **stdio** (a spawned CLI process) or **TCP** (a spawned or external server), and exposes a fully asynchronous, strongly-typed API built on [Tokio](https://tokio.rs).

:::info Technical preview
This crate is a Rust port of the upstream Copilot SDKs (Go, TypeScript, Python, .NET) and is currently in technical preview. The public API may evolve before a stable `1.0` release.
:::

## What you can build

The SDK turns the Copilot CLI into an embeddable agent engine you can script from Rust:

- Conversational assistants and chat front-ends.
- Autonomous coding agents that read/write files, run shell commands, and manage plans.
- Tool-augmented agents that call your own Rust functions or external [MCP servers](/docs/guides/mcp).
- Fleets of parallel agents working a shared task.
- Custom provider integrations via [BYOK](/docs/guides/byok) (bring your own key).

## Key features

- **Full session lifecycle** - create, resume, list, delete, and foreground control.
- **Streaming events** - 40+ strongly-typed event variants delivered over a broadcast channel.
- **Model management** - list models, switch models and reasoning effort mid-session.
- **Mode switching** - interactive, plan, and autopilot modes.
- **Plan management** - read, update, and delete the session plan.
- **Agent management** - list, select, and deselect custom agents.
- **Custom tools** - register tools with a fluent builder and async handlers.
- **Permission handling** - approve/deny tool calls via callbacks or allow/deny lists.
- **Hook system** - six lifecycle hooks (pre/post tool use, prompt submit, session start/end, error).
- **User input** - answer interactive questions raised by the agent.
- **Infinite sessions** - automatic and manual context-window compaction.
- **Shell operations** - execute and kill processes in the session.
- **Workspace files** - list, read, and create files in the session workspace.
- **Fleet management** - start parallel agent fleets.
- **OpenTelemetry** - configure distributed tracing for the CLI process.
- **MCP servers** - connect local (stdio) and remote (HTTP/SSE) Model Context Protocol servers.

## A 30-second taste

```rust
use copilot_sdk::{Client, SessionConfig};

#[tokio::main]
async fn main() -> copilot_sdk::Result<()> {
    let client = Client::builder().build()?;
    client.start().await?;

    let session = client.create_session(SessionConfig::default()).await?;
    let response = session.send_and_collect("Hello!", None).await?;
    println!("{response}");

    client.stop().await;
    Ok(())
}
```

## Feature parity

This port targets parity with the official SDKs.

| Feature | Status |
|---------|--------|
| Session CRUD (create / resume / list / delete) | Supported |
| Model management (get / switch) | Supported |
| Mode management (interactive / plan / autopilot) | Supported |
| Plan management (read / update / delete) | Supported |
| Agent management (list / select / deselect) | Supported |
| Tool system (register / invoke / permissions) | Supported |
| Hook system (6 lifecycle hooks) | Supported |
| Permission handling | Supported |
| User input handling | Supported |
| Infinite sessions and compaction | Supported |
| Shell operations (exec / kill) | Supported |
| Workspace file operations | Supported |
| Fleet management | Supported |
| Session logging | Supported |
| BYOK (custom providers) | Supported |
| OpenTelemetry configuration | Supported |
| Custom model list callback | Supported |
| MCP server integration | Supported |
| Custom agent configuration | Supported |
| Streaming events (40+ types) | Supported |
| Protocol v2/v3 negotiation | Supported |
| CLI bundling | Planned |

## How the documentation is organized

- **[Getting Started](/docs/getting-started/requirements)** - requirements, installation, and your first session.
- **[Core Concepts](/docs/core-concepts/architecture)** - architecture, the client/session model, events, and the wire protocol.
- **[Guides](/docs/guides/messaging)** - task-focused walkthroughs for each feature area.
- **[API Reference](/docs/api/client)** - module-by-module type and method reference.
- **[Examples](/docs/examples)** - a catalog of the 30+ runnable examples in the repository.

## License

MIT. See the [`LICENSE`](https://github.com/dayour/copilot-rust-sdk/blob/main/LICENSE) file. Upstream SDKs live at [github/copilot-sdk](https://github.com/github/copilot-sdk).
