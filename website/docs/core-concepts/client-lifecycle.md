---
id: client-lifecycle
title: Client Lifecycle
sidebar_label: Client Lifecycle
---

# Client Lifecycle

The [`Client`](/docs/api/client) owns the connection to the Copilot CLI and is the factory for sessions. This page covers building, starting, inspecting, and stopping it.

## Building a client

Use [`ClientBuilder`](/docs/api/client#clientbuilder) (via `Client::builder()`) for fluent configuration. Every setter returns `Self`.

```rust
use copilot_sdk::{Client, LogLevel};

let client = Client::builder()
    .cli_path("/usr/local/bin/copilot")   // override CLI discovery
    .cwd("/path/to/workspace")            // working directory for the CLI
    .log_level(LogLevel::Info)
    .env("MY_VAR", "value")               // extra environment variables
    .auto_restart(true)                   // restart after a fatal failure
    .build()?;
```

You can also construct directly from [`ClientOptions`](/docs/api/types#clientoptions) with `Client::new(options)` if you prefer building the options struct yourself.

### Builder options reference

| Method | Purpose |
|--------|---------|
| `cli_path(path)` | Explicit path to the CLI executable/script. |
| `cli_args(iter)` / `cli_arg(arg)` | Extra arguments passed to the CLI. |
| `use_stdio(bool)` | Use stdio transport (the default). |
| `cli_url(url)` / `port(u16)` | TCP transport target. |
| `auto_start(bool)` | Connect lazily on first use. |
| `auto_restart(bool)` | Restart the connection after a fatal failure. |
| `log_level(LogLevel)` | CLI log verbosity (`None`..`All`). |
| `cwd(dir)` | Working directory for the CLI process. |
| `env(key, value)` | Add an environment variable. |
| `github_token(token)` | Authenticate with a token. |
| `use_logged_in_user(bool)` | Use the CLI's logged-in user. |
| `deny_tool(spec)` / `deny_tools(iter)` | Block tools from running. |
| `allow_tool(spec)` / `allow_tools(iter)` | Auto-approve specific tools. |
| `allow_all_tools(bool)` | Auto-approve every tool (use with care). |
| `telemetry(config)` | OpenTelemetry export config. |
| `on_list_models(handler)` | Custom model list provider for BYOK. |

See [Permissions](/docs/guides/permissions) for tool allow/deny semantics, [Telemetry](/docs/guides/telemetry) for tracing, and [BYOK](/docs/guides/byok) for `on_list_models`.

## Starting

```rust
client.start().await?;
```

Starting spawns the CLI (stdio mode) or connects to it (TCP mode), brings up the JSON-RPC read loop, and negotiates the protocol version. Until `start` completes, RPC calls will fail with [`CopilotError::NotConnected`](/docs/api/error). After a successful start you can read the negotiated version:

```rust
if let Some(v) = client.negotiated_protocol_version().await {
    println!("negotiated protocol v{v}");
}
```

## Connection state

```rust
use copilot_sdk::ConnectionState;

match client.state().await {
    ConnectionState::Disconnected => { /* not started */ }
    ConnectionState::Connecting => { /* in progress */ }
    ConnectionState::Connected => { /* ready */ }
    ConnectionState::Error => { /* failed */ }
}
```

## Server queries

Once connected, the client exposes several read-only RPCs:

```rust
let ping   = client.ping(Some("hello".into())).await?; // health check
let status = client.get_status().await?;               // version + protocol
let auth   = client.get_auth_status().await?;          // auth state
let models = client.list_models().await?;              // available models
let tools  = client.tools_list(None).await?;           // available tools
let quota  = client.get_quota().await?;                // account quota
```

The models list is cached; call [`clear_models_cache`](/docs/api/client) to force a refresh.

## Foreground session control

Some workflows track a single "foreground" session that the CLI treats specially:

```rust
let current = client.get_foreground_session_id().await?;
client.set_foreground_session_id(&session.session_id()).await?;
```

## Lifecycle events

Register a handler to observe session lifecycle changes (created, deleted, updated, foreground, background):

```rust
let unsubscribe = client.on(|event| {
    println!("lifecycle: {} for {}", event.event_type, event.session_id);
}).await;

// later...
unsubscribe();
```

The constants in [`session_lifecycle_event_types`](/docs/api/types) name each `event_type`.

## Stopping

```rust
// Graceful: returns non-fatal shutdown errors rather than failing.
let errors = client.stop().await;

// Immediate: no graceful drain.
client.force_stop().await;
```

`stop` destroys outstanding sessions, shuts down the JSON-RPC loop, and terminates the CLI process. Prefer `stop`; reserve `force_stop` for crash/abort paths.

## Auto-start and auto-restart

- **`auto_start`** lets the client connect lazily the first time it is used instead of requiring an explicit `start`.
- **`auto_restart`** makes the client attempt to re-establish the connection after a fatal failure (for example, the CLI process exiting unexpectedly).

## Next

- [Sessions](/docs/core-concepts/sessions)
- [Error handling](/docs/core-concepts/error-handling)
