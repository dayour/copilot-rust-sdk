---
id: types
title: "API: types"
sidebar_label: types
---

# Module `types`

Configuration, request/response, and value types. All are re-exported at the crate root. Each type below has its own anchor; field types are shown as Rust definitions.

## Constants

```rust
pub const SDK_PROTOCOL_VERSION: u32; // = 3, the max protocol version supported
// MIN_PROTOCOL_VERSION (internal) = 2, the minimum supported
```

## Enums

### `ConnectionState`

```rust
pub enum ConnectionState { Disconnected, Connecting, Connected, Error }
```

### `SystemMessageMode`

```rust
pub enum SystemMessageMode { Append, Replace }
```

### `AttachmentType`

```rust
pub enum AttachmentType { File, Directory, Selection }
```

### `LogLevel`

```rust
pub enum LogLevel { None, Debug, Info, Warn, Error, All }
```

The client-level log level (CLI verbosity). Distinct from [`SessionLogLevel`](#sessionloglevel).

### `SessionMode`

```rust
pub enum SessionMode { Interactive, Plan, Autopilot }
```

### `SessionLogLevel`

```rust
pub enum SessionLogLevel { Error, Info, Warning }
```

### `ShellSignal`

```rust
pub enum ShellSignal { SIGINT, SIGKILL, SIGTERM }
```

## Client configuration

### `ClientOptions`

```rust
pub struct ClientOptions {
    pub cli_path: Option<PathBuf>,
    pub cli_args: Option<Vec<String>>,
    pub cwd: Option<PathBuf>,
    pub port: u16,
    pub use_stdio: bool,
    pub cli_url: Option<String>,
    pub log_level: LogLevel,
    pub auto_start: bool,
    pub auto_restart: bool,
    pub environment: Option<HashMap<String, String>>,
    pub github_token: Option<String>,
    pub use_logged_in_user: Option<bool>,
    pub deny_tools: Option<Vec<String>>,
    pub allow_tools: Option<Vec<String>>,
    pub allow_all_tools: bool,
    pub telemetry: Option<TelemetryConfig>,
    // on_list_models: optional async model-list callback (BYOK)
}
```

### `TelemetryConfig`

```rust
pub struct TelemetryConfig {
    pub otlp_endpoint: Option<String>,
    pub file_path: Option<String>,
    pub exporter_type: Option<String>,
    pub source_name: Option<String>,
    pub capture_content: Option<bool>,
}
```

See [Telemetry](/docs/guides/telemetry).

## Session configuration

### `SessionConfig`

```rust
pub struct SessionConfig {
    pub session_id: Option<String>,
    pub model: Option<String>,
    pub config_dir: Option<PathBuf>,
    pub tools: Vec<Tool>,
    pub system_message: Option<SystemMessageConfig>,
    pub available_tools: Option<Vec<String>>,
    pub excluded_tools: Option<Vec<String>>,
    pub provider: Option<ProviderConfig>,
    pub streaming: bool,
    pub mcp_servers: Option<HashMap<String, serde_json::Value>>,
    pub custom_agents: Option<Vec<CustomAgentConfig>>,
    pub skill_directories: Option<Vec<String>>,
    pub disabled_skills: Option<Vec<String>>,
    pub request_permission: Option<bool>,
    pub infinite_sessions: Option<InfiniteSessionConfig>,
    pub request_user_input: Option<bool>,
    pub reasoning_effort: Option<String>,
    pub working_directory: Option<String>,
    pub client_name: Option<String>,
    pub agent: Option<String>,
    pub hooks: Option<SessionHooks>,
    pub auto_byok_from_env: bool,
}
```

### `ResumeSessionConfig`

```rust
pub struct ResumeSessionConfig {
    pub model: Option<String>,
    pub tools: Vec<Tool>,
    pub provider: Option<ProviderConfig>,
    pub streaming: bool,
    pub mcp_servers: Option<HashMap<String, serde_json::Value>>,
    pub custom_agents: Option<Vec<CustomAgentConfig>>,
    pub skill_directories: Option<Vec<String>>,
    pub disabled_skills: Option<Vec<String>>,
    pub request_permission: Option<bool>,
    pub request_user_input: Option<bool>,
    pub reasoning_effort: Option<String>,
    pub working_directory: Option<String>,
    pub client_name: Option<String>,
    pub agent: Option<String>,
    pub disable_resume: bool,
    pub infinite_sessions: Option<InfiniteSessionConfig>,
    pub hooks: Option<SessionHooks>,
    pub auto_byok_from_env: bool,
}
```

### `SystemMessageConfig`

```rust
pub struct SystemMessageConfig {
    pub mode: Option<SystemMessageMode>,
    pub content: Option<String>,
}
```

### `InfiniteSessionConfig`

```rust
pub struct InfiniteSessionConfig {
    pub enabled: Option<bool>,
    pub background_compaction_threshold: Option<f64>,
    pub buffer_exhaustion_threshold: Option<f64>,
}

impl InfiniteSessionConfig {
    pub fn enabled() -> Self;
    pub fn with_thresholds(background: f64, exhaustion: f64) -> Self;
}
```

### `MessageOptions`

```rust
pub struct MessageOptions {
    pub prompt: String,
    pub attachments: Option<Vec<UserMessageAttachment>>,
    pub mode: Option<String>,
}
```

A `&str` or `String` converts into `MessageOptions` via `Into`, so `session.send("hi")` works.

## Providers (BYOK)

### `ProviderConfig`

```rust
pub struct ProviderConfig {
    pub base_url: String,
    pub provider_type: Option<String>,
    pub wire_api: Option<String>,
    pub api_key: Option<String>,
    pub bearer_token: Option<String>,
    pub azure: Option<AzureOptions>,
}
```

### `AzureOptions`

```rust
pub struct AzureOptions {
    pub api_version: Option<String>,
}
```

See [BYOK](/docs/guides/byok).

## MCP servers

### `McpLocalServerConfig`

```rust
pub struct McpLocalServerConfig {
    pub tools: Vec<String>,
    pub command: String,
    pub args: Vec<String>,
    pub server_type: Option<String>,
    pub timeout: Option<i32>,
    pub env: Option<HashMap<String, String>>,
    pub cwd: Option<String>,
}
```

### `McpRemoteServerConfig`

```rust
pub struct McpRemoteServerConfig {
    pub tools: Vec<String>,
    pub url: String,
    pub server_type: String,
    pub timeout: Option<i32>,
    pub headers: Option<HashMap<String, String>>,
}
```

### `McpServerConfig`

```rust
pub enum McpServerConfig {
    Local(McpLocalServerConfig),
    Remote(McpRemoteServerConfig),
}
```

See [MCP Servers](/docs/guides/mcp).

## Custom agents

### `CustomAgentConfig`

```rust
pub struct CustomAgentConfig {
    pub name: String,
    pub prompt: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub tools: Option<Vec<String>>,
    pub mcp_servers: Option<HashMap<String, serde_json::Value>>,
    pub infer: Option<bool>,
}
```

### `AgentInfo`

```rust
pub struct AgentInfo {
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
}
```

## Tools

### `Tool`

```rust
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters_schema: serde_json::Value,
    pub overrides_built_in_tool: bool,
    pub skip_permission: bool,
}

impl Tool {
    pub fn new(name: impl Into<String>) -> Self;
    pub fn description(self, desc: impl Into<String>) -> Self;
    pub fn schema(self, schema: serde_json::Value) -> Self;
    pub fn parameter(self, name: &str, param_type: &str, description: &str, required: bool) -> Self;
    pub fn overrides_built_in_tool(self, value: bool) -> Self;
    pub fn skip_permission(self, value: bool) -> Self;
    // With the `schemars` feature:
    pub fn typed_schema<T: schemars::JsonSchema>() -> Self;
}
```

### `ToolResultObject`

```rust
pub struct ToolResultObject {
    pub text_result_for_llm: String,
    pub binary_results_for_llm: Option<Vec<ToolBinaryResult>>,
    pub result_type: String,
    pub error: Option<String>,
    pub session_log: Option<String>,
    pub tool_telemetry: Option<HashMap<String, serde_json::Value>>,
}

impl ToolResultObject {
    pub fn text(result: impl Into<String>) -> Self;
    pub fn error(message: impl Into<String>) -> Self;
}

pub type ToolResult = ToolResultObject;
```

### `ToolBinaryResult`

```rust
pub struct ToolBinaryResult {
    pub data: String,        // base64
    pub mime_type: String,
    pub result_type: String,
    pub description: Option<String>,
}
```

### `ToolInvocation`

```rust
pub struct ToolInvocation {
    pub session_id: String,
    pub tool_call_id: String,
    pub tool_name: String,
    pub arguments: Option<serde_json::Value>,
}
```

### `ToolInfo`

```rust
pub struct ToolInfo {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<serde_json::Value>,
}
```

### `ToolsListResult`

```rust
pub struct ToolsListResult {
    pub tools: Vec<ToolInfo>,
}
```

See [Tools](/docs/guides/tools).

## Permissions

### `PermissionRequest`

```rust
pub struct PermissionRequest {
    pub kind: String,
    pub tool_call_id: Option<String>,
    pub extension_data: HashMap<String, serde_json::Value>,
}
```

### `PermissionRequestResult`

```rust
pub struct PermissionRequestResult {
    pub kind: String,
    pub rules: Option<Vec<serde_json::Value>>,
}

impl PermissionRequestResult {
    pub fn approved() -> Self;
    pub fn denied() -> Self;
    pub fn is_approved(&self) -> bool;
    pub fn is_denied(&self) -> bool;
}
```

See [Permissions](/docs/guides/permissions).

## User input

### `UserInputRequest`

```rust
pub struct UserInputRequest {
    pub question: String,
    pub choices: Option<Vec<String>>,
    pub allow_freeform: Option<bool>,
}
```

### `UserInputResponse`

```rust
pub struct UserInputResponse {
    pub answer: String,
    pub was_freeform: Option<bool>,
}
```

### `UserInputInvocation`

```rust
pub struct UserInputInvocation {
    pub session_id: String,
}
```

See [User Input](/docs/guides/user-input).

## Hooks

### `SessionHooks`

```rust
pub struct SessionHooks {
    pub on_pre_tool_use: Option<PreToolUseHandler>,
    pub on_post_tool_use: Option<PostToolUseHandler>,
    pub on_user_prompt_submitted: Option<UserPromptSubmittedHandler>,
    pub on_session_start: Option<SessionStartHandler>,
    pub on_session_end: Option<SessionEndHandler>,
    pub on_error_occurred: Option<ErrorOccurredHandler>,
}
```

### Hook input and output types

```rust
pub struct PreToolUseHookInput { pub timestamp: i64, pub cwd: String, pub tool_name: String, pub tool_args: serde_json::Value }
pub struct PreToolUseHookOutput {
    pub permission_decision: Option<String>,
    pub permission_decision_reason: Option<String>,
    pub modified_args: Option<serde_json::Value>,
    pub additional_context: Option<String>,
    pub suppress_output: Option<bool>,
}
pub struct PostToolUseHookInput { pub timestamp: i64, pub cwd: String, pub tool_name: String, pub tool_args: serde_json::Value, pub tool_result: serde_json::Value }
pub struct PostToolUseHookOutput { pub modified_result: Option<serde_json::Value>, pub additional_context: Option<String>, pub suppress_output: Option<bool> }
pub struct UserPromptSubmittedHookInput { pub timestamp: i64, pub cwd: String, pub prompt: String }
pub struct UserPromptSubmittedHookOutput { pub modified_prompt: Option<String>, pub additional_context: Option<String>, pub suppress_output: Option<bool> }
pub struct SessionStartHookInput { pub timestamp: i64, pub cwd: String, pub source: String, pub initial_prompt: Option<String> }
pub struct SessionStartHookOutput { pub additional_context: Option<String>, pub modified_config: Option<serde_json::Value> }
pub struct SessionEndHookInput { pub timestamp: i64, pub cwd: String, pub reason: String, pub final_message: Option<String>, pub error: Option<String> }
pub struct SessionEndHookOutput { pub suppress_output: Option<bool>, pub cleanup_actions: Option<Vec<String>>, pub session_summary: Option<String> }
pub struct ErrorOccurredHookInput { pub timestamp: i64, pub cwd: String, pub error: String, pub error_context: String, pub recoverable: bool }
pub struct ErrorOccurredHookOutput { pub suppress_output: Option<bool>, pub error_handling: Option<String>, pub retry_count: Option<i32>, pub user_notification: Option<String> }
```

### Hook handler aliases

```rust
pub type PreToolUseHandler = Arc<dyn Fn(&PreToolUseHookInput) -> PreToolUseHookOutput + Send + Sync>;
pub type PostToolUseHandler = Arc<dyn Fn(&PostToolUseHookInput) -> PostToolUseHookOutput + Send + Sync>;
pub type UserPromptSubmittedHandler = Arc<dyn Fn(&UserPromptSubmittedHookInput) -> UserPromptSubmittedHookOutput + Send + Sync>;
pub type SessionStartHandler = Arc<dyn Fn(&SessionStartHookInput) -> SessionStartHookOutput + Send + Sync>;
pub type SessionEndHandler = Arc<dyn Fn(&SessionEndHookInput) -> SessionEndHookOutput + Send + Sync>;
pub type ErrorOccurredHandler = Arc<dyn Fn(&ErrorOccurredHookInput) -> ErrorOccurredHookOutput + Send + Sync>;
```

See [Hooks](/docs/guides/hooks).

## Attachments

### `UserMessageAttachment`

```rust
pub struct UserMessageAttachment {
    pub attachment_type: AttachmentType,
    pub path: String,
    pub display_name: String,
}
```

### `SelectionPosition`

```rust
pub struct SelectionPosition { pub line: f64, pub character: f64 }
```

### `SelectionRange`

```rust
pub struct SelectionRange { pub start: SelectionPosition, pub end: SelectionPosition }
```

### `SelectionAttachment`

```rust
pub struct SelectionAttachment {
    pub file_path: String,
    pub display_name: String,
    pub text: String,
    pub selection: SelectionRange,
}
```

See [Attachments](/docs/guides/attachments).

## Models

### `ModelInfo`

```rust
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub capabilities: ModelCapabilities,
    pub policy: Option<ModelPolicy>,
    pub billing: Option<ModelBilling>,
    pub supported_reasoning_efforts: Option<Vec<String>>,
    pub default_reasoning_effort: Option<String>,
}
```

### `ModelCapabilities`

```rust
pub struct ModelCapabilities { pub supports: ModelSupports, pub limits: ModelLimits }
pub struct ModelSupports { pub vision: bool, pub reasoning_effort: bool }
pub struct ModelLimits { pub max_prompt_tokens: Option<u32>, pub max_context_window_tokens: u32, pub vision: Option<ModelVisionLimits> }
pub struct ModelVisionLimits { pub supported_media_types: Vec<String>, pub max_prompt_images: u32, pub max_prompt_image_size: u64 }
pub struct ModelPolicy { pub state: String, pub terms: String }
pub struct ModelBilling { pub multiplier: f64 }
```

### `SetModelOptions`

```rust
pub struct SetModelOptions { pub reasoning_effort: Option<String> }
```

## Sessions, plans, logging, fleet, shell, workspace

### `SessionMetadata`

```rust
pub struct SessionMetadata {
    pub session_id: String,
    pub start_time: Option<String>,
    pub modified_time: Option<String>,
    pub summary: Option<String>,
    pub is_remote: bool,
}
```

### `PlanData`

```rust
pub struct PlanData { pub content: Option<String>, pub title: Option<String> }
```

### `LogOptions`

```rust
pub struct LogOptions { pub level: Option<SessionLogLevel>, pub ephemeral: Option<bool> }
```

### `LogResult`

```rust
pub struct LogResult { pub event_id: String }
```

### `FleetStartOptions`

```rust
pub struct FleetStartOptions { pub prompt: Option<String> }
```

### `ShellExecOptions`

```rust
pub struct ShellExecOptions { pub command: String, pub cwd: Option<String>, pub env: Option<HashMap<String, String>> }
```

### `ShellExecResult`

```rust
pub struct ShellExecResult { pub process_id: String }
```

### `WorkspaceFile`

```rust
pub struct WorkspaceFile { pub path: String, pub size: Option<u64>, pub modified_at: Option<String> }
```

## Responses and status

### `PingResponse`

```rust
pub struct PingResponse { pub message: String, pub timestamp: i64, pub protocol_version: Option<u32> }
```

### `GetStatusResponse`

```rust
pub struct GetStatusResponse { pub version: String, pub protocol_version: u32 }
```

### `GetAuthStatusResponse`

```rust
pub struct GetAuthStatusResponse {
    pub is_authenticated: bool,
    pub auth_type: Option<String>,
    pub host: Option<String>,
    pub login: Option<String>,
    pub status_message: Option<String>,
}
```

### `GetForegroundSessionResponse`

```rust
pub struct GetForegroundSessionResponse { pub session_id: Option<String>, pub workspace_path: Option<String> }
```

### `SetForegroundSessionResponse`

```rust
pub struct SetForegroundSessionResponse { pub success: bool, pub error: Option<String> }
```

### `StopError`

```rust
pub struct StopError { pub message: String, pub source: Option<String> }
```

Collected (not thrown) by [`Client::stop`](/docs/api/client#constructors-and-lifecycle) so a noisy shutdown never masks your result.

## Quota

### `QuotaSnapshot`

```rust
pub struct QuotaSnapshot {
    pub quota_type: String,
    pub limit: Option<u64>,
    pub used: Option<u64>,
    pub remaining: Option<u64>,
    pub resets_at: Option<String>,
}
```

### `QuotaResult`

```rust
pub struct QuotaResult { pub quotas: Vec<QuotaSnapshot> }
```

## Session lifecycle (client-level)

### `session_lifecycle_event_types`

```rust
pub mod session_lifecycle_event_types {
    pub const CREATED: &str = "session.created";
    pub const DELETED: &str = "session.deleted";
    pub const UPDATED: &str = "session.updated";
    pub const FOREGROUND: &str = "session.foreground";
    pub const BACKGROUND: &str = "session.background";
}
```

### `SessionLifecycleEvent`

```rust
pub struct SessionLifecycleEvent {
    pub event_type: String,
    pub session_id: String,
    pub metadata: Option<SessionLifecycleEventMetadata>,
}

pub struct SessionLifecycleEventMetadata {
    pub start_time: Option<String>,
    pub modified_time: Option<String>,
    pub summary: Option<String>,
}
```

## Related

- [Events](/docs/api/events)
- [Client](/docs/api/client)
- [Session](/docs/api/session)
