---
id: examples
title: Examples
sidebar_label: Examples
---

# Examples

The repository ships 30+ runnable examples under [`examples/`](https://github.com/dayour/copilot-rust-sdk/tree/main/examples). Run any of them with:

```bash
cargo run --example <name>
```

Most examples need an installed, authenticated Copilot CLI (see [Requirements](/docs/getting-started/requirements)).

## Basics

| Example | Command | What it shows |
|---------|---------|---------------|
| Basic chat | `cargo run --example basic_chat` | Send a prompt and print the reply. |
| Streaming | `cargo run --example streaming` | Consume `AssistantMessageDelta` events token by token. |
| Multi-turn | `cargo run --example multi_turn` | A back-and-forth conversation across turns. |
| System prompt | `cargo run --example system_prompt` | Append or replace the system message. |

See [Messaging](/docs/guides/messaging) and [Events](/docs/core-concepts/events).

## Tools

| Example | Command | What it shows |
|---------|---------|---------------|
| Tool usage | `cargo run --example tool_usage` | Register a custom tool and a handler. |
| Fluent tools | `cargo run --example fluent_tools` | Build tools with the fluent `Tool` API. |
| Multiple tools | `cargo run --example multi_tools` | Register and route between several tools. |
| Tool progress | `cargo run --example tool_progress` | Stream progress and partial results from a tool. |

See [Tools](/docs/guides/tools).

## Permissions

| Example | Command | What it shows |
|---------|---------|---------------|
| Permission callback | `cargo run --example permission_callback` | Approve/deny via a permission handler. |
| Deny/allow tools | `cargo run --example deny_allow_tools` | Client-level allow/deny lists. |
| YOLO | `cargo run --example yolo` | Auto-approve everything with `allow_all_tools`. |

See [Permissions](/docs/guides/permissions).

## Models and modes

| Example | Command | What it shows |
|---------|---------|---------------|
| Set model | `cargo run --example set_model` | Switch models mid-session. |
| Reasoning effort | `cargo run --example reasoning_effort` | Control reasoning effort. |
| List models | `cargo run --example list_models` | Enumerate available models. |
| Mode switching | `cargo run --example mode_switching` | Interactive, plan, and autopilot modes. |

See [Models](/docs/guides/models) and [Modes](/docs/guides/modes).

## Plans and agents

| Example | Command | What it shows |
|---------|---------|---------------|
| Plan ops | `cargo run --example plan_ops` | Read, update, and delete the plan. |
| Agent management | `cargo run --example agent_management` | List, select, and deselect agents. |
| Custom agents | `cargo run --example custom_agents` | Define custom agent personas. |

See [Plans](/docs/guides/plans) and [Agents](/docs/guides/agents).

## Advanced

| Example | Command | What it shows |
|---------|---------|---------------|
| Hooks | `cargo run --example hooks` | Intercept lifecycle with the six hooks. |
| User input | `cargo run --example user_input` | Answer interactive questions from the agent. |
| Attachments | `cargo run --example attachments` | Attach files, directories, and selections. |
| Compaction events | `cargo run --example compaction_events` | Infinite sessions and compaction. |
| Shell exec | `cargo run --example shell_exec` | Run and kill shell processes. |
| MCP servers | `cargo run --example mcp_servers` | Connect local/remote MCP servers. |
| BYOK | `cargo run --example byok` | Use your own provider key and model list. |
| Telemetry | `cargo run --example telemetry` | Configure OpenTelemetry export. |
| Resume with tools | `cargo run --example resume_with_tools` | Resume a session and re-register tools. |

See the matching [guides](/docs/guides/messaging) for each feature.

## Demos and tooling

| Example | Command | What it shows |
|---------|---------|---------------|
| Debug pipe | `cargo run --example debug_pipe` | Inspect raw `Content-Length`-framed traffic. |
| Escape room | `cargo run --example escape_room` | A playful multi-step agent scenario. |
| rscopilot CLI | `cargo run --example rscopilot-cli` | A small interactive CLI built on the SDK. |

See [Transport and protocol](/docs/core-concepts/transport-and-protocol) for the `debug_pipe` background.

## Running tips

- Use `--release` for faster execution: `cargo run --release --example streaming`.
- Set `COPILOT_CLI_PATH` if the CLI is not on your `PATH`.
- Raise `RUST_LOG` (the SDK uses [`tracing`](https://docs.rs/tracing)) for diagnostics, for example `RUST_LOG=copilot_sdk=debug`.
