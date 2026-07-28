# copilot-sdk (Rust)

Rust SDK for interacting with the GitHub Copilot CLI agent runtime (JSON-RPC over stdio or TCP).

This is a Rust port of the upstream SDKs and is currently in technical preview.

## Requirements

- Rust 1.85+ (Edition 2024)
- GitHub Copilot CLI installed and authenticated
- `copilot` available in `PATH`, or set `COPILOT_CLI_PATH` to the CLI executable/script

## Install

Once published, add:

```toml
[dependencies]
copilot-sdk = "3.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

For development from this repository:

```toml
[dependencies]
copilot-sdk = { path = "." }
```

## Quick Start

```rust
use copilot_sdk::{Client, SessionConfig};

#[tokio::main]
async fn main() -> copilot_sdk::Result<()> {
    let client = Client::builder().build()?;
    client.start().await?;

    let session = client.create_session(SessionConfig::default()).await?;
    let response = session.send_and_collect("Hello!", None).await?;
    println!("{}", response);

    client.stop().await;
    Ok(())
}
```

## Features

### Session Management

Full session lifecycle with create, resume, list, delete, and foreground control:

```rust
let session = client.create_session(SessionConfig {
    model: Some("gpt-4.1".into()),
    streaming: true,
    client_name: Some("my-app".into()),
    ..Default::default()
}).await?;
```

### Model Management

Switch models and reasoning effort mid-session:

```rust
let model = session.get_model().await?;
session.set_model("claude-sonnet-4", Some(SetModelOptions {
    reasoning_effort: Some("high".into()),
})).await?;
```

### Mode Switching

Switch between interactive, plan, and autopilot modes:

```rust
session.set_mode(SessionMode::Plan).await?;
session.set_mode(SessionMode::Autopilot).await?;
session.set_mode(SessionMode::Interactive).await?;
```

### Plan Management

Read, update, and delete session plans:

```rust
session.update_plan(&PlanData {
    content: Some("Step 1: Implement\nStep 2: Test".into()),
    title: Some("Implementation Plan".into()),
}).await?;
let plan = session.read_plan().await?;
session.delete_plan().await?;
```

### Agent Management

List, select, and deselect custom agents:

```rust
let agents = session.list_agents().await?;
session.select_agent("code-reviewer").await?;
session.deselect_agent().await?;
```

### Custom Tools

Register tools that the assistant can invoke, with permission control:

```rust
let tool = Tool::new("get_weather")
    .description("Get current weather")
    .parameter("city", "string", "City name", true)
    .skip_permission(true);

session.register_tool_with_handler(tool, Some(handler)).await;
```

### Infinite Sessions

Automatic context window management with manual compaction support:

```rust
let config = SessionConfig {
    infinite_sessions: Some(InfiniteSessionConfig::enabled()),
    ..Default::default()
};
// Trigger manual compaction
session.compact().await?;
```

### Session Logging

Add log entries to sessions:

```rust
session.log("Processing step complete", Some(LogOptions {
    level: Some(SessionLogLevel::Info),
    ephemeral: Some(false),
})).await?;
```

### Shell Operations

Execute shell commands and manage processes:

```rust
let result = session.shell_exec(ShellExecOptions {
    command: "cargo test".into(),
    cwd: Some("/my/project".into()),
    env: None,
}).await?;
session.shell_kill(&result.process_id, ShellSignal::SIGTERM).await?;
```

### Workspace File Operations

List, read, and create files in the session workspace:

```rust
let files = session.workspace_list_files().await?;
let content = session.workspace_read_file("plan.md").await?;
session.workspace_create_file("notes.md", "# Notes").await?;
```

### Fleet Management

Start parallel agent fleets:

```rust
session.start_fleet(Some(FleetStartOptions {
    prompt: Some("Build and test the project".into()),
})).await?;
```

### Rust-native LSP

The crate includes a typed, Content-Length framed LSP 3.17 server with deterministic
Rust declaration symbols:

```rust,no_run
use copilot_sdk::LspServer;

#[tokio::main]
async fn main() -> copilot_sdk::Result<()> {
    let server = LspServer::new();
    server
        .serve(tokio::io::stdin(), tokio::io::stdout())
        .await?;
    Ok(())
}
```

Semantic symbols use canonical strong names and stable IDs prefixed with
`rust-lsp-semantic-v1:`. A Copilot session can provide matching identity and workspace
metadata through `session.lsp_server_config()`.

### Client Utilities

```rust
let status = client.get_status().await?;       // CLI version info
let auth = client.get_auth_status().await?;    // Authentication state
let models = client.list_models().await?;      // Available models
let tools = client.tools_list(None).await?;    // Available tools
let quota = client.get_quota().await?;         // Account quota
```

### OpenTelemetry Integration

Configure distributed tracing for the CLI process:

```rust
let client = Client::builder()
    .telemetry(TelemetryConfig {
        otlp_endpoint: Some("http://localhost:4318".into()),
        exporter_type: Some("otlp-http".into()),
        source_name: Some("my-app".into()),
        capture_content: Some(true),
        file_path: None,
    })
    .build()?;
```

### BYOK (Bring Your Own Key)

Use your own API keys with compatible providers, with custom model listing:

```rust
let client = Client::builder()
    .on_list_models(|| async {
        Ok(vec![ModelInfo { /* ... */ }])
    })
    .build()?;

let config = SessionConfig {
    provider: Some(ProviderConfig {
        base_url: "https://api.openai.com/v1".into(),
        api_key: Some("sk-...".into()),
        ..Default::default()
    }),
    auto_byok_from_env: true,
    ..Default::default()
};
```

### Hooks

Intercept session lifecycle at key points:

```rust
let config = SessionConfig {
    hooks: Some(SessionHooks {
        on_pre_tool_use: Some(Arc::new(|input| {
            println!("Tool: {}", input.tool_name);
            PreToolUseHookOutput::default()
        })),
        ..Default::default()
    }),
    ..Default::default()
};
```

## Examples

```bash
cargo run --example basic_chat          # Simple Q&A
cargo run --example streaming           # Streaming responses
cargo run --example tool_usage          # Custom tools
cargo run --example set_model           # Model switching
cargo run --example mode_switching      # Mode management
cargo run --example plan_ops            # Plan CRUD
cargo run --example agent_management    # Agent operations
cargo run --example telemetry           # OpenTelemetry setup
cargo run --example shell_exec          # Shell commands
cargo run --example hooks               # Session hooks
cargo run --example byok                # Bring Your Own Key
```

## Development

### Setup

Enable pre-commit hooks to catch formatting/linting issues before push:

```bash
git config core.hooksPath .githooks
```

### Commands

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

E2E tests (real Copilot CLI):

```bash
cargo test --features e2e -- --test-threads=1
```

Snapshot conformance tests (optional, against upstream YAML snapshots):

```bash
cargo test --features snapshots --test snapshot_conformance
```

Set `COPILOT_SDK_RUST_SNAPSHOT_DIR` or `UPSTREAM_SNAPSHOTS` to point at `copilot-sdk/test/snapshots` if it cannot be auto-detected.

## Protocol Compatibility

- **SDK Protocol Version**: 3 (minimum: 2)
- **Transport**: stdio (spawned CLI) and TCP (spawned or external server)
- **JSON-RPC**: v2.0 with Content-Length framing
- **LSP**: 3.17 server with full document synchronization and semantic symbols

## Feature Parity

Parity is **measured, not asserted**. The authoritative contract is
`schemas/api.schema.json`, vendored verbatim from the `@github/copilot` npm
package (protocol v3). It declares 196 JSON-RPC methods. `codegen/` turns that
schema into `src/generated/methods.rs`, and `tests/rpc_parity.rs` fails the
build if the crate drifts from it.

### RPC method coverage

| Surface | Description | Covered |
|---------|-------------|---------|
| `server` | Client-to-server, not session-scoped | 38 / 38 |
| `session` | Client-to-server, scoped to one session | 143 / 143 |
| `clientSession` | Reverse RPC the SDK must serve | 15 / 15 |
| **Total** | | **196 / 196 (100%)** |

A method counts as covered only if the crate actually invokes it (outbound) or
serves it (inbound match arm). Declaring a type is not sufficient.

### Coverage by namespace

| Namespace | Methods | Namespace | Methods |
|-----------|--------:|-----------|--------:|
| `session.permissions` | 21 | `session.canvas` | 5 |
| `sessions` | 19 | `session.history` | 5 |
| `session.mcp` | 15 | `session.eventLog` | 4 |
| `sessionFs` | 13 | `session.extensions` | 4 |
| `session.tasks` | 11 | `session.model` | 4 |
| `mcp` | 8 | `canvas` | 3 |
| `session.ui` | 8 | `session.name` | 3 |
| `session.workspaces` | 8 | `session.plan` | 3 |
| `session.commands` | 6 | `session.queue` | 3 |
| `session.metadata` | 6 | `session.remote` | 3 |
| `session.skills` | 6 | `session.tools` | 3 |
| `session.agent` | 5 | others (auth, mode, schedule, shell, skills, account, agentRegistry, connect, models, ping, secrets, fleet, instructions, lsp, options, plugins, telemetry, usage, tools, user) | 23 |

### How this is enforced

`tests/rpc_parity.rs` runs six gates, and CI runs them on every push:

- `schema_registry_is_well_formed` - the generated registry matches the schema.
- `session_surface_names_are_prefixed` - naming invariants hold.
- `every_invoked_method_exists_in_schema` - **catches typo'd wire names**; this
  gate found and fixed 8 live protocol defects (for example
  `model.get_current` should be `model.getCurrent`, and
  `account.get_quota` should be `account.getQuota`).
- `off_schema_exceptions_are_still_needed` - the 11 documented methods the CLI
  accepts but does not declare cannot silently grow.
- `unbound_methods_match_allowlist` - a ratchet. `tests/rpc_parity_allowlist.txt`
  is now empty, so any newly unbound method fails CI.
- `report_parity_coverage` - prints the table above.

CI additionally runs a codegen drift check: it regenerates
`src/generated/methods.rs` from the schema and fails if the committed file
differs.

### Capability parity

| Capability | Status |
|------------|--------|
| Session CRUD (create/resume/list/delete) | yes |
| Model management (get/switch) | yes |
| Mode management (interactive/plan/autopilot) | yes |
| Plan management (read/update/delete) | yes |
| Agent management (list/select/deselect) | yes |
| Tool system (register/invoke/permissions) | yes |
| Hook system (6 lifecycle hooks) | yes |
| Permission handling | yes |
| User input handling | yes |
| Infinite sessions and compaction | yes |
| Shell operations (exec/kill) | yes |
| Workspace file operations | yes |
| Session filesystem provider | yes |
| Canvas subsystem | yes |
| MCP server integration | yes |
| Fleet management | yes |
| Rust-native LSP 3.17 server and semantic symbol IDs | yes |
| Session logging | yes |
| BYOK (custom providers) | yes |
| OpenTelemetry configuration | yes |
| Custom model list callback | yes |
| Custom agent configuration | yes |
| Streaming events (40+ types) | yes |
| Protocol v2/v3 negotiation | yes |


## License

MIT License - see [LICENSE](LICENSE).

## Related

- Upstream SDKs: https://github.com/github/copilot-sdk
