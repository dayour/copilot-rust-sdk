---
id: client
title: "API: client"
sidebar_label: client
---

# Module `client`

Connection management and the factory for sessions. Re-exported at the crate root as `copilot_sdk::{Client, ClientBuilder, LifecycleHandler}`.

## `LifecycleHandler`

```rust
pub type LifecycleHandler = Arc<dyn Fn(&SessionLifecycleEvent) + Send + Sync>;
```

Handler for client-level lifecycle events (session created, deleted, updated, foreground, background).

## `Client`

The Copilot client. Manages the connection to the CLI and creates/manages sessions.

### Constructors and lifecycle

```rust
pub fn new(options: ClientOptions) -> Result<Self>;
pub fn builder() -> ClientBuilder;
pub async fn start(&self) -> Result<()>;
pub async fn stop(&self) -> Vec<StopError>;
pub async fn force_stop(&self);
pub async fn state(&self) -> ConnectionState;
```

- `new` - construct from explicit [`ClientOptions`](/docs/api/types#clientoptions).
- `builder` - fluent configuration (recommended).
- `start` - spawn/connect the CLI and negotiate the protocol.
- `stop` - graceful shutdown; returns non-fatal [`StopError`](/docs/api/types#stoperror)s instead of failing.
- `force_stop` - immediate, non-graceful teardown.
- `state` - current [`ConnectionState`](/docs/api/types#connectionstate).

### Sessions

```rust
pub async fn create_session(&self, config: SessionConfig) -> Result<Arc<Session>>;
pub async fn resume_session(&self, session_id: &str, config: ResumeSessionConfig) -> Result<Arc<Session>>;
pub async fn list_sessions(&self) -> Result<Vec<SessionMetadata>>;
pub async fn delete_session(&self, session_id: &str) -> Result<()>;
pub async fn get_last_session_id(&self) -> Result<Option<String>>;
pub async fn get_session(&self, session_id: &str) -> Option<Arc<Session>>;
```

### Server and status

```rust
pub async fn ping(&self, message: Option<String>) -> Result<PingResponse>;
pub async fn get_status(&self) -> Result<GetStatusResponse>;
pub async fn get_auth_status(&self) -> Result<GetAuthStatusResponse>;
pub async fn list_models(&self) -> Result<Vec<ModelInfo>>;
pub async fn tools_list(&self, model_id: Option<&str>) -> Result<ToolsListResult>;
pub async fn get_quota(&self) -> Result<QuotaResult>;
pub async fn clear_models_cache(&self);
pub async fn get_foreground_session_id(&self) -> Result<GetForegroundSessionResponse>;
pub async fn set_foreground_session_id(&self, session_id: &str) -> Result<SetForegroundSessionResponse>;
pub async fn negotiated_protocol_version(&self) -> Option<u32>;
```

### Lifecycle event handling

```rust
pub async fn on<F>(&self, handler: F) -> impl FnOnce()
where
    F: Fn(&SessionLifecycleEvent) + Send + Sync + 'static;
```

Registers a client-level lifecycle handler and returns an unsubscribe closure.

## `ClientBuilder`

Fluent builder for a [`Client`](#client). Every setter consumes and returns `Self`.

### Builder methods

```rust
pub fn new() -> Self;
pub fn cli_path(self, path: impl Into<PathBuf>) -> Self;
pub fn cli_args<I, S>(self, args: I) -> Self where I: IntoIterator<Item = S>, S: Into<String>;
pub fn cli_arg(self, arg: impl Into<String>) -> Self;
pub fn use_stdio(self, use_stdio: bool) -> Self;
pub fn cli_url(self, url: impl Into<String>) -> Self;
pub fn port(self, port: u16) -> Self;
pub fn auto_start(self, auto_start: bool) -> Self;
pub fn auto_restart(self, auto_restart: bool) -> Self;
pub fn log_level(self, level: LogLevel) -> Self;
pub fn cwd(self, dir: impl Into<PathBuf>) -> Self;
pub fn env(self, key: impl Into<String>, value: impl Into<String>) -> Self;
pub fn github_token(self, token: impl Into<String>) -> Self;
pub fn use_logged_in_user(self, value: bool) -> Self;
pub fn deny_tool(self, tool_spec: impl Into<String>) -> Self;
pub fn deny_tools<I, S>(self, tool_specs: I) -> Self where I: IntoIterator<Item = S>, S: Into<String>;
pub fn allow_tool(self, tool_spec: impl Into<String>) -> Self;
pub fn allow_tools<I, S>(self, tool_specs: I) -> Self where I: IntoIterator<Item = S>, S: Into<String>;
pub fn allow_all_tools(self, allow: bool) -> Self;
pub fn telemetry(self, config: TelemetryConfig) -> Self;
pub fn on_list_models<F, Fut>(self, handler: F) -> Self
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = Result<Vec<ModelInfo>, CopilotError>> + Send + 'static;
pub fn build(self) -> Result<Client>;
```

See [Client lifecycle](/docs/core-concepts/client-lifecycle) for usage and option semantics.

## Related

- [Session](/docs/api/session)
- [Types](/docs/api/types)
