---
id: session
title: "API: session"
sidebar_label: session
---

# Module `session`

A single conversation with the agent. Re-exported at the crate root as `copilot_sdk::{Session, EventSubscription, RegisteredTool, EventHandler, PermissionHandler, ToolHandler, UserInputHandler, InvokeFuture}`.

## Type aliases

```rust
pub type EventHandler = Arc<dyn Fn(&SessionEvent) + Send + Sync>;
pub type PermissionHandler = Arc<dyn Fn(&PermissionRequest) -> PermissionRequestResult + Send + Sync>;
pub type ToolHandler = Arc<dyn Fn(&str, &Value) -> ToolResultObject + Send + Sync>;
pub type UserInputHandler = Arc<dyn Fn(&UserInputRequest, &UserInputInvocation) -> UserInputResponse + Send + Sync>;
pub type InvokeFuture = Pin<Box<dyn Future<Output = Result<Value>> + Send>>;
```

## `EventSubscription`

A subscription to session events, backed by a broadcast receiver.

```rust
pub struct EventSubscription {
    pub receiver: broadcast::Receiver<SessionEvent>,
}

impl EventSubscription {
    pub async fn recv(&mut self)
        -> std::result::Result<SessionEvent, broadcast::error::RecvError>;
}
```

## `RegisteredTool`

A tool registered with the session, plus its optional handler.

```rust
pub struct RegisteredTool {
    pub tool: Tool,
    pub handler: Option<ToolHandler>,
}
```

## `Session`

### Construction and identity

```rust
pub fn new<F>(session_id: String, workspace_path: Option<String>, invoke_fn: F) -> Self
where
    F: Fn(&str, Option<Value>) -> InvokeFuture + Send + Sync + 'static;
pub fn session_id(&self) -> &str;
pub fn workspace_path(&self) -> Option<&str>;
```

### Event handling

```rust
pub fn subscribe(&self) -> EventSubscription;
pub async fn on<F>(&self, handler: F) -> impl FnOnce()
where
    F: Fn(&SessionEvent) + Send + Sync + 'static;
pub async fn off(&self, handler_id: u64);
pub async fn dispatch_event(&self, event: SessionEvent);
```

### Messaging

```rust
pub async fn send(&self, options: impl Into<MessageOptions>) -> Result<String>;
pub async fn abort(&self) -> Result<()>;
pub async fn get_messages(&self) -> Result<Vec<SessionEvent>>;
```

### Wait helpers

```rust
pub async fn wait_for_idle(&self, timeout: Option<Duration>) -> Result<Option<SessionEvent>>;
pub async fn send_and_wait(&self, options: impl Into<MessageOptions>, timeout: Option<Duration>) -> Result<Option<SessionEvent>>;
pub async fn send_and_collect(&self, options: impl Into<MessageOptions>, timeout: Option<Duration>) -> Result<String>;
```

### Tool management

```rust
pub async fn register_tool(&self, tool: Tool);
pub async fn register_tool_with_handler(&self, tool: Tool, handler: Option<ToolHandler>);
pub async fn register_tools(&self, tools: Vec<Tool>);
pub async fn get_tool(&self, name: &str) -> Option<Tool>;
pub async fn get_tools(&self) -> Vec<Tool>;
pub async fn invoke_tool(&self, name: &str, arguments: &Value) -> Result<ToolResultObject>;
```

### Permission handling

```rust
pub async fn register_permission_handler<F>(&self, handler: F)
where
    F: Fn(&PermissionRequest) -> PermissionRequestResult + Send + Sync + 'static;
pub async fn handle_permission_request(&self, request: &PermissionRequest) -> PermissionRequestResult;
```

### User input handling

```rust
pub async fn register_user_input_handler<F>(&self, handler: F)
where
    F: Fn(&UserInputRequest, &UserInputInvocation) -> UserInputResponse + Send + Sync + 'static;
pub async fn handle_user_input_request(&self, request: &UserInputRequest) -> Result<UserInputResponse>;
pub async fn has_user_input_handler(&self) -> bool;
```

### Hooks

```rust
pub async fn register_hooks(&self, hooks: SessionHooks);
pub async fn has_hooks(&self) -> bool;
pub async fn handle_hooks_invoke(&self, hook_type: &str, input: &Value) -> Result<Value>;
```

### Model management

```rust
pub async fn get_model(&self) -> Result<String>;
pub async fn set_model(&self, model: &str, options: Option<SetModelOptions>) -> Result<()>;
```

### Mode management

```rust
pub async fn get_mode(&self) -> Result<SessionMode>;
pub async fn set_mode(&self, mode: SessionMode) -> Result<()>;
```

### Plan management

```rust
pub async fn read_plan(&self) -> Result<Option<PlanData>>;
pub async fn update_plan(&self, plan: &PlanData) -> Result<()>;
pub async fn delete_plan(&self) -> Result<()>;
```

### Agent management

```rust
pub async fn list_agents(&self) -> Result<Vec<AgentInfo>>;
pub async fn get_current_agent(&self) -> Result<Option<AgentInfo>>;
pub async fn select_agent(&self, name: &str) -> Result<()>;
pub async fn deselect_agent(&self) -> Result<()>;
```

### Session logging

```rust
pub async fn log(&self, message: &str, options: Option<LogOptions>) -> Result<LogResult>;
```

### Compaction

```rust
pub async fn compact(&self) -> Result<()>;
```

### Fleet management

```rust
pub async fn start_fleet(&self, options: Option<FleetStartOptions>) -> Result<()>;
```

### Shell operations

```rust
pub async fn shell_exec(&self, options: ShellExecOptions) -> Result<ShellExecResult>;
pub async fn shell_kill(&self, process_id: &str, signal: ShellSignal) -> Result<()>;
```

### Workspace operations

```rust
pub async fn workspace_list_files(&self) -> Result<Vec<WorkspaceFile>>;
pub async fn workspace_read_file(&self, path: &str) -> Result<String>;
pub async fn workspace_create_file(&self, path: &str, content: &str) -> Result<()>;
```

### Lifecycle

```rust
pub async fn destroy(&self) -> Result<()>;
```

## Related

- [Sessions concept](/docs/core-concepts/sessions)
- [Events](/docs/api/events)
- [Types](/docs/api/types)
