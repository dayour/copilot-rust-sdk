---
id: sessions
title: Sessions
sidebar_label: Sessions
---

# Sessions

A [`Session`](/docs/api/session) represents a single conversation with the agent. It maintains conversation state, streams events, runs tools, and exposes the bulk of the SDK's functionality. Sessions are returned as `Arc<Session>` so they are cheap to clone and share across tasks.

## Creating

```rust
use copilot_sdk::SessionConfig;

let session = client.create_session(SessionConfig {
    model: Some("gpt-4.1".into()),
    streaming: true,
    client_name: Some("my-app".into()),
    ..Default::default()
}).await?;
```

[`SessionConfig`](/docs/api/types#sessionconfig) is large and every field has a sensible default. Highlights:

| Field | Purpose |
|-------|---------|
| `model` | Initial model id. |
| `streaming` | Emit incremental delta events. |
| `tools` | Tools to register at creation. |
| `system_message` | Append/replace the system prompt. |
| `mcp_servers` | MCP servers to connect. |
| `custom_agents` | Custom agent definitions. |
| `provider` | BYOK provider configuration. |
| `infinite_sessions` | Enable automatic compaction. |
| `hooks` | Lifecycle hook handlers. |
| `request_permission` | Route tool approvals to your handler. |
| `request_user_input` | Route user-input questions to your handler. |
| `reasoning_effort` | Reasoning effort for reasoning models. |
| `client_name` | A label for your application. |

## Resuming

Pick up a previous session by id with [`resume_session`](/docs/api/client) and a [`ResumeSessionConfig`](/docs/api/types#resumesessionconfig):

```rust
use copilot_sdk::ResumeSessionConfig;

let last = client.get_last_session_id().await?;
if let Some(id) = last {
    let session = client.resume_session(&id, ResumeSessionConfig::default()).await?;
}
```

You can re-register tools, providers, MCP servers, hooks, and agents on resume, exactly as on creation.

## Listing and deleting

```rust
let sessions = client.list_sessions().await?;
for meta in &sessions {
    println!("{} ({})", meta.session_id, meta.summary.clone().unwrap_or_default());
}

client.delete_session("old-session-id").await?;
```

[`SessionMetadata`](/docs/api/types#sessionmetadata) includes the id, start/modified times, a summary, and whether the session is remote.

## Identity

```rust
let id = session.session_id();              // &str
let workspace = session.workspace_path();   // Option<&str> (infinite sessions)
```

You can also retrieve a previously created session from the client by id with `client.get_session(id)`.

## What a session can do

The `Session` API surface is broad. Each area has a dedicated guide:

| Capability | Methods | Guide |
|------------|---------|-------|
| Messaging | `send`, `send_and_collect`, `send_and_wait`, `abort`, `get_messages` | [Messaging](/docs/guides/messaging) |
| Events | `subscribe`, `on`, `off`, `dispatch_event` | [Events](/docs/core-concepts/events) |
| Models | `get_model`, `set_model` | [Models](/docs/guides/models) |
| Modes | `get_mode`, `set_mode` | [Modes](/docs/guides/modes) |
| Plans | `read_plan`, `update_plan`, `delete_plan` | [Plans](/docs/guides/plans) |
| Agents | `list_agents`, `get_current_agent`, `select_agent`, `deselect_agent` | [Agents](/docs/guides/agents) |
| Tools | `register_tool`, `register_tool_with_handler`, `invoke_tool`, ... | [Tools](/docs/guides/tools) |
| Permissions | `register_permission_handler` | [Permissions](/docs/guides/permissions) |
| User input | `register_user_input_handler` | [User Input](/docs/guides/user-input) |
| Hooks | `register_hooks` | [Hooks](/docs/guides/hooks) |
| Compaction | `compact` | [Infinite Sessions](/docs/guides/infinite-sessions) |
| Shell | `shell_exec`, `shell_kill` | [Shell](/docs/guides/shell) |
| Workspace | `workspace_list_files`, `workspace_read_file`, `workspace_create_file` | [Workspace](/docs/guides/workspace) |
| Fleet | `start_fleet` | [Fleet](/docs/guides/fleet) |
| Logging | `log` | [Logging](/docs/guides/logging) |

## Lifetime and cleanup

Destroy a session when you are done with it to release its resources on the CLI side:

```rust
session.destroy().await?;
```

If you do not destroy sessions explicitly, [`client.stop()`](/docs/core-concepts/client-lifecycle#stopping) cleans up outstanding sessions for you. Calling methods on a destroyed session yields [`CopilotError::SessionDestroyed`](/docs/api/error).

## Next

- [Events](/docs/core-concepts/events)
- [Messaging](/docs/guides/messaging)
