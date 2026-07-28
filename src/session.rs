// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Session management for the Copilot SDK.
//!
//! A session represents a conversation with the Copilot CLI.

use crate::canvas::CanvasHandler;
use crate::error::{CopilotError, Result};
use crate::events::{SessionEvent, SessionEventData};
use crate::lsp::LspServerConfig;
use crate::session_fs::SessionFsProvider;
use crate::types::{
    AgentInfo, AutoModeSwitchRequest, AutoModeSwitchResponse, CommandContext, ElicitationContext,
    ElicitationMode, ElicitationResult, ErrorOccurredHookInput, ExitPlanModeRequest,
    ExitPlanModeResult, FleetStartOptions, LogOptions, LogResult, LspInitializeOptions,
    MessageOptions, PermissionRequest, PermissionRequestResult, PlanData, PostToolUseHookInput,
    PreToolUseHookInput, SectionTransformFn, SessionCapabilities, SessionEndHookInput,
    SessionHooks, SessionMode, SessionStartHookInput, SessionUpdateOptions, SetModelOptions,
    ShellExecOptions, ShellExecResult, ShellSignal, Tool, ToolResultObject, UiCapabilities,
    UiInputOptions, UserInputInvocation, UserInputRequest, UserInputResponse,
    UserPromptSubmittedHookInput, WorkspaceFile,
};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};

// =============================================================================
// Event Handler Types
// =============================================================================

/// Handler for session events.
pub type EventHandler = Arc<dyn Fn(&SessionEvent) + Send + Sync>;

/// Handler for permission requests.
pub type PermissionHandler =
    Arc<dyn Fn(&PermissionRequest) -> PermissionRequestResult + Send + Sync>;

/// Handler for tool invocations.
pub type ToolHandler = Arc<dyn Fn(&str, &Value) -> ToolResultObject + Send + Sync>;

/// Handler for tool invocations that also receives the invocation metadata
/// (session id, tool-call id, and propagated W3C Trace Context). Prefer this
/// over [`ToolHandler`] for new code; mirrors nodejs's
/// `ToolHandler<TArgs> = (args, invocation: ToolInvocation) => ...`.
pub type ToolHandlerWithInvocation =
    Arc<dyn Fn(&Value, &crate::types::ToolInvocation) -> ToolResultObject + Send + Sync>;

/// Handler for user input requests.
pub type UserInputHandler =
    Arc<dyn Fn(&UserInputRequest, &UserInputInvocation) -> UserInputResponse + Send + Sync>;

/// Handler invoked when the server dispatches an elicitation request.
///
/// Returns the [`ElicitationResult`] carrying the user's response.
pub type ElicitationHandler = Arc<dyn Fn(&ElicitationContext) -> ElicitationResult + Send + Sync>;

/// Handler invoked when the agent requests to exit plan mode.
pub type ExitPlanModeHandler =
    Arc<dyn Fn(&ExitPlanModeRequest) -> ExitPlanModeResult + Send + Sync>;

/// Handler invoked when the agent requests an auto-mode switch after a rate limit.
pub type AutoModeSwitchHandler =
    Arc<dyn Fn(&AutoModeSwitchRequest) -> AutoModeSwitchResponse + Send + Sync>;

/// Handler invoked when a registered slash command is executed by the user.
pub type CommandHandler = Arc<dyn Fn(&CommandContext) + Send + Sync>;

/// Type alias for the invoke future.
pub type InvokeFuture = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Value>> + Send>>;

type InvokeFn = dyn Fn(&str, Option<Value>) -> InvokeFuture + Send + Sync;

// =============================================================================
// Event Subscription
// =============================================================================

/// A subscription to session events.
///
/// Events are delivered via the broadcast channel receiver.
pub struct EventSubscription {
    pub receiver: broadcast::Receiver<SessionEvent>,
}

impl EventSubscription {
    /// Receive the next event.
    pub async fn recv(&mut self) -> std::result::Result<SessionEvent, broadcast::error::RecvError> {
        self.receiver.recv().await
    }
}

// =============================================================================
// Registered Tool
// =============================================================================

/// A tool registered with the session, including its handler.
#[derive(Clone)]
pub struct RegisteredTool {
    /// Tool definition.
    pub tool: Tool,
    /// Handler for tool invocations.
    pub handler: Option<ToolHandler>,
    /// Invocation-aware handler, preferred over `handler` when both are set.
    pub invocation_handler: Option<ToolHandlerWithInvocation>,
}

// =============================================================================
// Session
// =============================================================================

/// Shared session state.
struct SessionState {
    /// Registered tools.
    tools: HashMap<String, RegisteredTool>,
    /// Permission handler.
    permission_handler: Option<PermissionHandler>,
    /// User input handler.
    user_input_handler: Option<UserInputHandler>,
    /// Elicitation handler.
    elicitation_handler: Option<ElicitationHandler>,
    /// Exit-plan-mode handler.
    exit_plan_mode_handler: Option<ExitPlanModeHandler>,
    /// Auto-mode-switch handler.
    auto_mode_switch_handler: Option<AutoModeSwitchHandler>,
    /// Registered slash-command handlers, keyed by command name.
    command_handlers: HashMap<String, CommandHandler>,
    /// Handler for inbound `canvas.*` reverse-RPC requests.
    canvas_handler: Option<Arc<dyn CanvasHandler>>,
    /// Provider for inbound `sessionFs.*` reverse-RPC requests.
    session_fs_provider: Option<Arc<dyn SessionFsProvider>>,
    /// System message section transform callbacks, keyed by section id.
    transform_callbacks: HashMap<String, SectionTransformFn>,
    /// Host-reported capabilities from create/resume.
    capabilities: SessionCapabilities,
    /// Canvas instances the host reported as open on resume.
    open_canvases: Vec<crate::canvas::OpenCanvasInstance>,
    /// Session hooks.
    hooks: Option<SessionHooks>,
    /// Callback-based event handlers.
    event_handlers: HashMap<u64, EventHandler>,
    /// Next handler ID.
    next_handler_id: AtomicU64,
    /// Provider for the current W3C Trace Context, propagated onto outbound
    /// `session.send` requests. Mirrors nodejs's per-session
    /// `traceContextProvider` (see `session.ts` constructor).
    trace_context_provider: Option<crate::trace::TraceContextProvider>,
}

/// A Copilot conversation session.
///
/// Sessions maintain conversation state, handle events, and manage tool execution.
///
/// # Example
///
/// ```no_run
/// use copilot_sdk::{Client, SessionConfig, SessionEventData};
///
/// #[tokio::main]
/// async fn main() -> copilot_sdk::Result<()> {
/// let client = Client::builder().build()?;
/// client.start().await?;
/// let session = client.create_session(SessionConfig::default()).await?;
///
/// // Subscribe to events
/// let mut events = session.subscribe();
///
/// // Send a message
/// session.send("Hello!").await?;
///
/// // Process events
/// while let Ok(event) = events.recv().await {
///     match &event.data {
///         SessionEventData::AssistantMessage(msg) => println!("{}", msg.content),
///         SessionEventData::SessionIdle(_) => break,
///         _ => {}
///     }
/// }
/// client.stop().await;
/// # Ok(())
/// # }
/// ```
pub struct Session {
    /// Session ID.
    pub(crate) session_id: String,
    /// Workspace path for infinite sessions.
    workspace_path: Option<String>,
    /// Event broadcaster.
    event_tx: broadcast::Sender<SessionEvent>,
    /// Session state.
    state: Arc<RwLock<SessionState>>,
    /// JSON-RPC invoke function (injected by Client).
    pub(crate) invoke_fn: Arc<InvokeFn>,
}

impl Session {
    /// Create a new session.
    ///
    /// This is typically called by the Client when creating a session.
    pub fn new<F>(session_id: String, workspace_path: Option<String>, invoke_fn: F) -> Self
    where
        F: Fn(&str, Option<Value>) -> InvokeFuture + Send + Sync + 'static,
    {
        let (event_tx, _) = broadcast::channel(1024);

        Self {
            session_id,
            workspace_path,
            event_tx,
            state: Arc::new(RwLock::new(SessionState {
                tools: HashMap::new(),
                permission_handler: None,
                user_input_handler: None,
                elicitation_handler: None,
                exit_plan_mode_handler: None,
                auto_mode_switch_handler: None,
                command_handlers: HashMap::new(),
                canvas_handler: None,
                session_fs_provider: None,
                transform_callbacks: HashMap::new(),
                capabilities: SessionCapabilities::default(),
                open_canvases: Vec::new(),
                hooks: None,
                event_handlers: HashMap::new(),
                next_handler_id: AtomicU64::new(1),
                trace_context_provider: None,
            })),
            invoke_fn: Arc::new(invoke_fn),
        }
    }

    // =========================================================================
    // Session Properties
    // =========================================================================

    /// Get the session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get the workspace path for infinite sessions.
    ///
    /// Contains checkpoints/, plan.md, and files/ subdirectories.
    /// Returns None if infinite sessions are disabled.
    pub fn workspace_path(&self) -> Option<&str> {
        self.workspace_path.as_deref()
    }

    /// Return an LSP server configuration associated with this session.
    ///
    /// The configuration carries the session identity and infinite-session workspace
    /// root without inventing a second session or transport protocol.
    pub fn lsp_server_config(&self) -> LspServerConfig {
        LspServerConfig::for_session(&self.session_id, self.workspace_path.as_deref())
    }

    /// (Re)load the merged LSP configuration set for the session's working directory.
    ///
    /// This is the client-facing `session.lsp.initialize` RPC: it asks the Copilot CLI
    /// to load project- and user-level LSP server configs (traversing up to `git_root`
    /// for monorepos). Pass `None` to use the session defaults.
    ///
    /// Note: this configures the CLI's LSP integration and is independent of the
    /// crate's in-process [`crate::lsp::LspServer`].
    pub async fn lsp_initialize(&self, options: Option<LspInitializeOptions>) -> Result<()> {
        let mut params = match options {
            Some(opts) => serde_json::to_value(opts).map_err(|e| {
                CopilotError::Protocol(format!("Failed to serialize LSP options: {}", e))
            })?,
            None => serde_json::json!({}),
        };
        params["sessionId"] = serde_json::json!(self.session_id);
        (self.invoke_fn)("session.lsp.initialize", Some(params)).await?;
        Ok(())
    }

    // =========================================================================
    // Event Handling
    // =========================================================================

    /// Subscribe to session events.
    ///
    /// Returns a receiver that will receive all session events.
    pub fn subscribe(&self) -> EventSubscription {
        EventSubscription {
            receiver: self.event_tx.subscribe(),
        }
    }

    /// Register a callback-based event handler.
    ///
    /// Returns an unsubscribe closure. Call it to remove the handler.
    /// Alternatively, use [`Session::off`] with the internal handler ID.
    pub async fn on<F>(&self, handler: F) -> impl FnOnce()
    where
        F: Fn(&SessionEvent) + Send + Sync + 'static,
    {
        let mut state = self.state.write().await;
        let id = state.next_handler_id.fetch_add(1, Ordering::SeqCst);
        state.event_handlers.insert(id, Arc::new(handler));

        let state_ref = Arc::clone(&self.state);
        move || {
            tokio::spawn(async move {
                state_ref.write().await.event_handlers.remove(&id);
            });
        }
    }

    /// Unsubscribe a callback-based event handler.
    pub async fn off(&self, handler_id: u64) {
        let mut state = self.state.write().await;
        state.event_handlers.remove(&handler_id);
    }

    /// Dispatch an event to all subscribers.
    ///
    /// Broadcast request events (external_tool.requested, permission.requested) are handled
    /// internally before being forwarded to user handlers (protocol v3 model).
    ///
    /// This is called by the Client when events are received.
    pub async fn dispatch_event(&self, event: SessionEvent) {
        // Handle broadcast request events (protocol v3) before dispatching to user handlers.
        // Fire-and-forget: the response is sent asynchronously via RPC.
        self.handle_broadcast_event(&event).await;

        // Keep the cached capabilities live. Without this, `capabilities()` would
        // report whatever the host advertised at create/resume time and silently
        // go stale for the rest of the session.
        self.merge_capabilities_from_event(&event).await;

        // Send to broadcast channel
        let _ = self.event_tx.send(event.clone());

        // Call registered handlers
        let state = self.state.read().await;
        for handler in state.event_handlers.values() {
            handler(&event);
        }
    }

    /// Fold a `capabilities.changed` event into the cached session capabilities.
    ///
    /// Mirrors the upstream Node.js SDK, which applies a shallow top-level merge
    /// (`{ ...this._capabilities, ...event.data }`). Because `ui` is the only
    /// top-level member, a payload that carries `ui` replaces the cached `ui`
    /// wholesale rather than merging field by field; a payload that omits `ui`
    /// leaves the cached value untouched.
    async fn merge_capabilities_from_event(&self, event: &SessionEvent) {
        let SessionEventData::CapabilitiesChanged(ref data) = event.data else {
            return;
        };
        let Some(ref ui) = data.ui else {
            return;
        };

        let mut state = self.state.write().await;
        state.capabilities.ui = Some(UiCapabilities {
            elicitation: ui.elicitation,
            mcp_apps: ui.mcp_apps,
            canvases: ui.canvases,
        });
    }

    /// Handle broadcast request events by executing local handlers and responding via RPC.
    ///
    /// Implements the protocol v3 broadcast model where tool calls and permission requests
    /// are broadcast as session events to all clients.
    async fn handle_broadcast_event(&self, event: &SessionEvent) {
        match &event.data {
            SessionEventData::ExternalToolRequested(data) => {
                let request_id = match &data.request_id {
                    Some(id) => id.clone(),
                    None => return,
                };
                let tool_name = match &data.tool_name {
                    Some(name) => name.clone(),
                    None => return,
                };

                // Check if this session handles this tool
                if self.get_tool(&tool_name).await.is_none() {
                    return; // This client doesn't handle this tool; another client will.
                }

                let _tool_call_id = data.tool_call_id.clone().unwrap_or_default();
                let arguments = data.arguments.clone().unwrap_or(serde_json::json!({}));
                let session_id = self.session_id.clone();

                // Execute tool and respond via handlePendingToolCall RPC
                match self.invoke_tool(&tool_name, &arguments).await {
                    Ok(result) => {
                        // If the tool reported a failure with an error, send via top-level error
                        let params = if result.result_type == "failure"
                            || result.result_type == "error"
                        {
                            serde_json::json!({
                                "sessionId": session_id,
                                "requestId": request_id,
                                "error": result.error.unwrap_or_else(|| result.text_result_for_llm.clone()),
                            })
                        } else {
                            serde_json::json!({
                                "sessionId": session_id,
                                "requestId": request_id,
                                "result": {
                                    "textResultForLlm": result.text_result_for_llm,
                                    "resultType": result.result_type,
                                    "toolTelemetry": result.tool_telemetry.unwrap_or_default(),
                                }
                            })
                        };
                        let _ =
                            (self.invoke_fn)("session.tools.handlePendingToolCall", Some(params))
                                .await;
                    }
                    Err(e) => {
                        let params = serde_json::json!({
                            "sessionId": session_id,
                            "requestId": request_id,
                            "error": e.to_string(),
                        });
                        let _ =
                            (self.invoke_fn)("session.tools.handlePendingToolCall", Some(params))
                                .await;
                    }
                }
            }
            SessionEventData::PermissionRequested(data) => {
                let request_id = match &data.request_id {
                    Some(id) => id.clone(),
                    None => return,
                };
                let perm_data = match &data.permission_request {
                    Some(d) => d.clone(),
                    None => return,
                };

                let session_id = self.session_id.clone();

                // Build PermissionRequest from JSON
                use crate::types::PermissionRequest;
                let kind = perm_data
                    .get("kind")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let tool_call_id = perm_data
                    .get("toolCallId")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                let mut extension_data = std::collections::HashMap::new();
                if let Some(obj) = perm_data.as_object() {
                    for (key, value) in obj {
                        if key != "kind" && key != "toolCallId" {
                            extension_data.insert(key.clone(), value.clone());
                        }
                    }
                }

                let request = PermissionRequest {
                    kind,
                    tool_call_id,
                    extension_data,
                };

                let result = self.handle_permission_request(&request).await;

                let mut perm_result_inner = serde_json::json!({
                    "kind": result.kind,
                });
                if let Some(rules) = &result.rules {
                    perm_result_inner["rules"] = serde_json::json!(rules);
                }
                let perm_result = serde_json::json!({
                    "sessionId": session_id,
                    "requestId": request_id,
                    "result": perm_result_inner,
                });

                let _ = (self.invoke_fn)(
                    "session.permissions.handlePendingPermissionRequest",
                    Some(perm_result),
                )
                .await;
            }
            SessionEventData::ElicitationRequested(data) => {
                let request_id = match &data.request_id {
                    Some(id) => id.clone(),
                    None => return,
                };
                let handler = {
                    let state = self.state.read().await;
                    match &state.elicitation_handler {
                        Some(h) => h.clone(),
                        None => return, // Another client may handle this.
                    }
                };
                let session_id = self.session_id.clone();
                let mode = data.mode.as_deref().map(|m| match m {
                    "url" => ElicitationMode::Url,
                    _ => ElicitationMode::Form,
                });
                let context = ElicitationContext {
                    session_id: session_id.clone(),
                    message: data.message.clone(),
                    requested_schema: data.requested_schema.clone(),
                    mode,
                    elicitation_source: data.elicitation_source.clone(),
                    url: data.url.clone(),
                };
                let result =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| handler(&context)))
                        .unwrap_or_else(|_| ElicitationResult::cancel());
                let params = serde_json::json!({
                    "sessionId": session_id,
                    "requestId": request_id,
                    "result": result,
                });
                let _ = (self.invoke_fn)("session.ui.handlePendingElicitation", Some(params)).await;
            }
            SessionEventData::ExitPlanModeRequested(data) => {
                let request_id = match &data.request_id {
                    Some(id) => id.clone(),
                    None => return,
                };
                let handler = {
                    let state = self.state.read().await;
                    match &state.exit_plan_mode_handler {
                        Some(h) => h.clone(),
                        None => return,
                    }
                };
                let session_id = self.session_id.clone();
                let request = ExitPlanModeRequest {
                    summary: data.summary.clone(),
                    plan_content: data.plan_content.clone(),
                    actions: data.actions.clone(),
                    recommended_action: data.recommended_action.clone(),
                };
                let response = handler(&request);
                let params = serde_json::json!({
                    "sessionId": session_id,
                    "requestId": request_id,
                    "response": response,
                });
                let _ =
                    (self.invoke_fn)("session.ui.handlePendingExitPlanMode", Some(params)).await;
            }
            SessionEventData::AutoModeSwitchRequested(data) => {
                let request_id = match &data.request_id {
                    Some(id) => id.clone(),
                    None => return,
                };
                let handler = {
                    let state = self.state.read().await;
                    match &state.auto_mode_switch_handler {
                        Some(h) => h.clone(),
                        None => return,
                    }
                };
                let session_id = self.session_id.clone();
                let request = AutoModeSwitchRequest {
                    error_code: data.error_code.clone(),
                    retry_after_seconds: data.retry_after_seconds,
                };
                let response = handler(&request);
                let params = serde_json::json!({
                    "sessionId": session_id,
                    "requestId": request_id,
                    "response": response,
                });
                let _ =
                    (self.invoke_fn)("session.ui.handlePendingAutoModeSwitch", Some(params)).await;
            }
            SessionEventData::CommandExecute(data) => {
                let request_id = match &data.request_id {
                    Some(id) => id.clone(),
                    None => return,
                };
                let session_id = self.session_id.clone();
                let handler = {
                    let state = self.state.read().await;
                    state.command_handlers.get(&data.command_name).cloned()
                };
                let error = match handler {
                    Some(handler) => {
                        let ctx = CommandContext {
                            session_id: session_id.clone(),
                            command: data.command.clone(),
                            command_name: data.command_name.clone(),
                            args: data.args.clone(),
                        };
                        handler(&ctx);
                        None
                    }
                    None => Some(format!("Unknown command: {}", data.command_name)),
                };
                let mut params = serde_json::json!({
                    "sessionId": session_id,
                    "requestId": request_id,
                });
                if let Some(error_msg) = error {
                    params["error"] = serde_json::Value::String(error_msg);
                }
                let _ =
                    (self.invoke_fn)("session.commands.handlePendingCommand", Some(params)).await;
            }
            _ => {} // Not a broadcast request event
        }
    }

    // =========================================================================
    // Messaging
    // =========================================================================

    /// Send a message to the session.
    ///
    /// Returns the message ID.
    pub async fn send(&self, options: impl Into<MessageOptions>) -> Result<String> {
        let options = options.into();
        let provider = {
            let state = self.state.read().await;
            state.trace_context_provider.clone()
        };
        let trace = crate::trace::get_trace_context(provider.as_ref()).await;
        let mut params = serde_json::json!({
            "sessionId": self.session_id,
            "prompt": options.prompt,
            "attachments": options.attachments,
            "mode": options.mode,
            "agentMode": options.agent_mode,
            "requestHeaders": options.request_headers,
            "displayPrompt": options.display_prompt,
        });
        if let Some(obj) = params.as_object_mut() {
            if let Some(traceparent) = trace.traceparent {
                obj.insert("traceparent".into(), Value::String(traceparent));
            }
            if let Some(tracestate) = trace.tracestate {
                obj.insert("tracestate".into(), Value::String(tracestate));
            }
        }

        let result = (self.invoke_fn)("session.send", Some(params)).await?;

        result
            .get("messageId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| CopilotError::Protocol("Missing messageId in response".into()))
    }

    /// Abort the current message processing.
    pub async fn abort(&self) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session_id,
        });

        (self.invoke_fn)("session.abort", Some(params)).await?;
        Ok(())
    }

    /// Get all messages in the session.
    pub async fn get_messages(&self) -> Result<Vec<SessionEvent>> {
        let params = serde_json::json!({
            "sessionId": self.session_id,
        });

        let result = (self.invoke_fn)("session.getMessages", Some(params)).await?;

        let events: Vec<SessionEvent> = result
            .get("events")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| SessionEvent::from_json(v).ok())
                    .collect()
            })
            .or_else(|| {
                result
                    .get("messages")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| SessionEvent::from_json(v).ok())
                            .collect()
                    })
            })
            .ok_or_else(|| {
                CopilotError::Protocol("Missing events in getMessages response".into())
            })?;

        Ok(events)
    }

    // =========================================================================
    // Tool Management
    // =========================================================================

    /// Register a tool with this session.
    pub async fn register_tool(&self, tool: Tool) {
        self.register_tool_with_handler(tool, None).await;
    }

    /// Register a tool with a handler.
    pub async fn register_tool_with_handler(&self, tool: Tool, handler: Option<ToolHandler>) {
        let mut state = self.state.write().await;
        let name = tool.name.clone();
        state.tools.insert(
            name,
            RegisteredTool {
                tool,
                handler,
                invocation_handler: None,
            },
        );
    }

    /// Register a tool with an invocation-aware handler.
    ///
    /// The handler receives the raw arguments plus a [`ToolInvocation`]
    /// (session id, tool-call id, and propagated W3C Trace Context), matching
    /// nodejs's `ToolHandler(args, invocation)` signature. Preferred over
    /// [`register_tool_with_handler`](Self::register_tool_with_handler) for new code.
    pub async fn register_tool_with_invocation_handler(
        &self,
        tool: Tool,
        handler: Option<ToolHandlerWithInvocation>,
    ) {
        let mut state = self.state.write().await;
        let name = tool.name.clone();
        state.tools.insert(
            name,
            RegisteredTool {
                tool,
                handler: None,
                invocation_handler: handler,
            },
        );
    }

    /// Register multiple tools.
    pub async fn register_tools(&self, tools: Vec<Tool>) {
        let mut state = self.state.write().await;
        for tool in tools {
            let name = tool.name.clone();
            state.tools.insert(
                name,
                RegisteredTool {
                    tool,
                    handler: None,
                    invocation_handler: None,
                },
            );
        }
    }

    /// Get a registered tool by name.
    pub async fn get_tool(&self, name: &str) -> Option<Tool> {
        let state = self.state.read().await;
        state.tools.get(name).map(|rt| rt.tool.clone())
    }

    /// Get all registered tools.
    pub async fn get_tools(&self) -> Vec<Tool> {
        let state = self.state.read().await;
        state.tools.values().map(|rt| rt.tool.clone()).collect()
    }

    /// Invoke a tool handler.
    ///
    /// Prefer [`invoke_tool_with_invocation`](Self::invoke_tool_with_invocation)
    /// for new code, which also supplies tool-call id and trace context to the
    /// handler.
    pub async fn invoke_tool(&self, name: &str, arguments: &Value) -> Result<ToolResultObject> {
        let state = self.state.read().await;
        let registered = state
            .tools
            .get(name)
            .ok_or_else(|| CopilotError::ToolNotFound(name.to_string()))?;

        if let Some(handler) = registered.invocation_handler.as_ref() {
            let invocation = crate::types::ToolInvocation {
                session_id: self.session_id.clone(),
                tool_call_id: String::new(),
                tool_name: name.to_string(),
                arguments: Some(arguments.clone()),
                traceparent: None,
                tracestate: None,
            };
            return Ok(handler(arguments, &invocation));
        }

        let handler = registered
            .handler
            .as_ref()
            .ok_or_else(|| CopilotError::ToolError(format!("No handler for tool: {}", name)))?;

        Ok(handler(name, arguments))
    }

    /// Invoke a tool handler with full invocation metadata (tool-call id,
    /// W3C Trace Context). Falls back to the legacy `(name, arguments)`
    /// handler shape when only that is registered.
    pub async fn invoke_tool_with_invocation(
        &self,
        invocation: &crate::types::ToolInvocation,
    ) -> Result<ToolResultObject> {
        let state = self.state.read().await;
        let registered = state
            .tools
            .get(invocation.tool_name.as_str())
            .ok_or_else(|| CopilotError::ToolNotFound(invocation.tool_name.clone()))?;

        let empty = Value::Object(serde_json::Map::new());
        let args = invocation.arguments.as_ref().unwrap_or(&empty);

        if let Some(handler) = registered.invocation_handler.as_ref() {
            return Ok(handler(args, invocation));
        }

        let handler = registered.handler.as_ref().ok_or_else(|| {
            CopilotError::ToolError(format!("No handler for tool: {}", invocation.tool_name))
        })?;

        Ok(handler(&invocation.tool_name, args))
    }

    // =========================================================================
    // Permission Handling
    // =========================================================================

    /// Register a permission handler.
    pub async fn register_permission_handler<F>(&self, handler: F)
    where
        F: Fn(&PermissionRequest) -> PermissionRequestResult + Send + Sync + 'static,
    {
        let mut state = self.state.write().await;
        state.permission_handler = Some(Arc::new(handler));
    }

    /// Handle a permission request.
    ///
    /// Delegates to the registered permission handler, or denies by default
    /// if no handler is set.
    pub async fn handle_permission_request(
        &self,
        request: &PermissionRequest,
    ) -> PermissionRequestResult {
        let state = self.state.read().await;

        if let Some(handler) = &state.permission_handler {
            handler(request)
        } else {
            // Default: deny all permissions
            PermissionRequestResult::denied()
        }
    }

    // =========================================================================
    // User Input Handling
    // =========================================================================

    /// Register a handler for user input requests from the server.
    pub async fn register_user_input_handler<F>(&self, handler: F)
    where
        F: Fn(&UserInputRequest, &UserInputInvocation) -> UserInputResponse + Send + Sync + 'static,
    {
        let mut state = self.state.write().await;
        state.user_input_handler = Some(Arc::new(handler));
    }

    /// Handle a user input request from the server.
    pub async fn handle_user_input_request(
        &self,
        request: &UserInputRequest,
    ) -> Result<UserInputResponse> {
        let state = self.state.read().await;
        if let Some(handler) = &state.user_input_handler {
            let invocation = UserInputInvocation {
                session_id: self.session_id.clone(),
            };
            Ok(handler(request, &invocation))
        } else {
            Err(CopilotError::Protocol(
                "No user input handler registered".into(),
            ))
        }
    }

    /// Check if a user input handler is registered.
    pub async fn has_user_input_handler(&self) -> bool {
        let state = self.state.read().await;
        state.user_input_handler.is_some()
    }

    // =========================================================================
    // Hooks
    // =========================================================================

    /// Register session hooks.
    pub async fn register_hooks(&self, hooks: SessionHooks) {
        let mut state = self.state.write().await;
        state.hooks = Some(hooks);
    }

    /// Check if any hooks are registered.
    pub async fn has_hooks(&self) -> bool {
        let state = self.state.read().await;
        state.hooks.as_ref().is_some_and(|h| h.has_any())
    }

    // =========================================================================
    // Trace context
    // =========================================================================

    /// Register the provider used to obtain the current W3C Trace Context for
    /// this session. Called by [`Client`](crate::Client) immediately after
    /// session construction when `ClientOptions::on_get_trace_context` is set,
    /// mirroring nodejs's `Session` constructor (`traceContextProvider`).
    pub(crate) async fn set_trace_context_provider(
        &self,
        provider: crate::trace::TraceContextProvider,
    ) {
        let mut state = self.state.write().await;
        state.trace_context_provider = Some(provider);
    }

    // =========================================================================
    // Capabilities
    // =========================================================================

    /// Set the host-reported capabilities for this session.
    ///
    /// Typically called by the [`Client`](crate::Client) from the
    /// `session.create` / `session.resume` response.
    pub async fn set_capabilities(&self, capabilities: SessionCapabilities) {
        let mut state = self.state.write().await;
        state.capabilities = capabilities;
    }

    /// Get the host-reported capabilities for this session.
    pub async fn capabilities(&self) -> SessionCapabilities {
        let state = self.state.read().await;
        state.capabilities.clone()
    }

    /// Record the canvas instances the host reported as open.
    ///
    /// Typically called by the [`Client`](crate::Client) from the
    /// `session.resume` response.
    pub async fn set_open_canvases(&self, instances: Vec<crate::canvas::OpenCanvasInstance>) {
        let mut state = self.state.write().await;
        state.open_canvases = instances;
    }

    /// Canvas instances the host restored for this session.
    ///
    /// Populated from the `session.resume` response; empty for fresh sessions.
    pub async fn open_canvases(&self) -> Vec<crate::canvas::OpenCanvasInstance> {
        let state = self.state.read().await;
        state.open_canvases.clone()
    }

    // =========================================================================
    // Elicitation / Plan-Mode / Auto-Mode-Switch Handlers
    // =========================================================================

    /// Register a handler invoked when the server dispatches an elicitation request.
    pub async fn register_elicitation_handler<F>(&self, handler: F)
    where
        F: Fn(&ElicitationContext) -> ElicitationResult + Send + Sync + 'static,
    {
        let mut state = self.state.write().await;
        state.elicitation_handler = Some(Arc::new(handler));
    }

    /// Register an already-constructed elicitation handler.
    ///
    /// Used internally by [`Client`](crate::Client) to wire
    /// [`SessionCallbacks::on_elicitation`](crate::SessionCallbacks::on_elicitation)
    /// at config time, immediately after session construction.
    pub(crate) async fn register_elicitation_handler_arc(&self, handler: ElicitationHandler) {
        let mut state = self.state.write().await;
        state.elicitation_handler = Some(handler);
    }

    /// Register a handler invoked when the agent requests to exit plan mode.
    pub async fn register_exit_plan_mode_handler<F>(&self, handler: F)
    where
        F: Fn(&ExitPlanModeRequest) -> ExitPlanModeResult + Send + Sync + 'static,
    {
        let mut state = self.state.write().await;
        state.exit_plan_mode_handler = Some(Arc::new(handler));
    }

    /// Register an already-constructed exit-plan-mode handler.
    ///
    /// Used internally by [`Client`](crate::Client) to wire
    /// [`SessionCallbacks::on_exit_plan_mode`](crate::SessionCallbacks::on_exit_plan_mode)
    /// at config time, immediately after session construction.
    pub(crate) async fn register_exit_plan_mode_handler_arc(&self, handler: ExitPlanModeHandler) {
        let mut state = self.state.write().await;
        state.exit_plan_mode_handler = Some(handler);
    }

    /// Register a handler invoked when the agent requests an auto-mode switch.
    pub async fn register_auto_mode_switch_handler<F>(&self, handler: F)
    where
        F: Fn(&AutoModeSwitchRequest) -> AutoModeSwitchResponse + Send + Sync + 'static,
    {
        let mut state = self.state.write().await;
        state.auto_mode_switch_handler = Some(Arc::new(handler));
    }

    /// Register an already-constructed auto-mode-switch handler.
    ///
    /// Used internally by [`Client`](crate::Client) to wire
    /// [`SessionCallbacks::on_auto_mode_switch`](crate::SessionCallbacks::on_auto_mode_switch)
    /// at config time, immediately after session construction.
    pub(crate) async fn register_auto_mode_switch_handler_arc(
        &self,
        handler: AutoModeSwitchHandler,
    ) {
        let mut state = self.state.write().await;
        state.auto_mode_switch_handler = Some(handler);
    }

    // =========================================================================
    // Slash Commands
    // =========================================================================

    /// Register a handler for a slash command invoked by the user.
    ///
    /// The command name is matched against `command.execute` broadcast events.
    /// To advertise the command in the CLI TUI, also add a
    /// [`CommandDeclaration`](crate::CommandDeclaration) to the session config.
    pub async fn register_command<F>(&self, name: impl Into<String>, handler: F)
    where
        F: Fn(&CommandContext) + Send + Sync + 'static,
    {
        let mut state = self.state.write().await;
        state
            .command_handlers
            .insert(name.into(), Arc::new(handler));
    }

    /// Remove a previously registered slash-command handler.
    pub async fn unregister_command(&self, name: &str) {
        let mut state = self.state.write().await;
        state.command_handlers.remove(name);
    }

    // =========================================================================
    // Canvas Provider
    // =========================================================================

    /// Register the handler for inbound `canvas.*` reverse-RPC requests.
    ///
    /// A single handler dispatches all canvas lifecycle verbs for this session,
    /// switching on the request's `canvas_id`. Declare the canvases themselves
    /// via [`SessionConfig::canvases`](crate::SessionConfig) so the runtime knows
    /// which canvas ids this session provides.
    pub async fn register_canvas_handler(&self, handler: Arc<dyn CanvasHandler>) {
        let mut state = self.state.write().await;
        state.canvas_handler = Some(handler);
    }

    /// Get the registered canvas handler, if any.
    pub async fn canvas_handler(&self) -> Option<Arc<dyn CanvasHandler>> {
        let state = self.state.read().await;
        state.canvas_handler.clone()
    }

    // =========================================================================
    // Session filesystem
    // =========================================================================

    /// Register the provider that answers inbound `sessionFs.*` reverse-RPC
    /// requests for this session.
    ///
    /// Required when the client was constructed with
    /// [`ClientOptions::session_fs`](crate::ClientOptions::session_fs).
    pub async fn register_session_fs_provider(&self, provider: Arc<dyn SessionFsProvider>) {
        let mut state = self.state.write().await;
        state.session_fs_provider = Some(provider);
    }

    /// Get the registered session filesystem provider, if any.
    pub async fn session_fs_provider(&self) -> Option<Arc<dyn SessionFsProvider>> {
        let state = self.state.read().await;
        state.session_fs_provider.clone()
    }

    // =========================================================================
    // System message transforms
    // =========================================================================

    /// Register system message section transform callbacks, keyed by section id.
    ///
    /// Called by the client when a [`SessionConfig`](crate::SessionConfig)
    /// declares [`SectionOverrideAction::Transform`](crate::SectionOverrideAction)
    /// overrides. Passing an empty map clears any previously registered
    /// callbacks.
    pub async fn register_transform_callbacks(
        &self,
        callbacks: HashMap<String, SectionTransformFn>,
    ) {
        let mut state = self.state.write().await;
        state.transform_callbacks = callbacks;
    }

    /// Returns the section ids that currently have a transform callback.
    pub async fn transform_callback_ids(&self) -> Vec<String> {
        let state = self.state.read().await;
        let mut ids: Vec<String> = state.transform_callbacks.keys().cloned().collect();
        ids.sort();
        ids
    }

    /// Applies registered transform callbacks to the runtime-rendered system
    /// message sections.
    ///
    /// Sections with no registered callback pass through unchanged. A callback
    /// that panics is not caught here; a callback returning the original string
    /// is the documented no-op.
    ///
    /// # Internal
    ///
    /// Invoked by the client for inbound `systemMessage.transform` requests.
    pub async fn handle_system_message_transform(
        &self,
        sections: HashMap<String, String>,
    ) -> HashMap<String, String> {
        let callbacks = {
            let state = self.state.read().await;
            state.transform_callbacks.clone()
        };

        let mut result = HashMap::with_capacity(sections.len());
        for (section_id, content) in sections {
            match callbacks.get(&section_id) {
                Some(callback) => {
                    let transformed = callback(content.clone()).await;
                    result.insert(section_id, transformed);
                }
                None => {
                    result.insert(section_id, content);
                }
            }
        }
        result
    }

    // =========================================================================
    // UI API
    // =========================================================================

    /// Access the interactive UI (elicitation) API for this session.
    ///
    /// The returned [`SessionUi`] routes to `session.ui.*` RPCs and requires
    /// host elicitation support. Check
    /// [`capabilities().supports_elicitation()`](SessionCapabilities::supports_elicitation)
    /// before calling its methods.
    pub fn ui(&self) -> SessionUi<'_> {
        SessionUi { session: self }
    }

    async fn assert_elicitation(&self) -> Result<()> {
        let state = self.state.read().await;
        if state.capabilities.supports_elicitation() {
            Ok(())
        } else {
            Err(CopilotError::Protocol(
                "Elicitation is not supported by the host. Check \
                 session.capabilities().supports_elicitation() before calling UI methods."
                    .into(),
            ))
        }
    }

    /// Handle a `hooks.invoke` callback from the server.
    ///
    /// Dispatches to the appropriate hook handler based on `hook_type` and returns
    /// the serialized output JSON.
    pub async fn handle_hooks_invoke(&self, hook_type: &str, input: &Value) -> Result<Value> {
        let state = self.state.read().await;
        let hooks = match &state.hooks {
            Some(h) => h,
            None => return Ok(Value::Null),
        };

        match hook_type {
            "preToolUse" => {
                if let Some(handler) = &hooks.on_pre_tool_use {
                    let hook_input: PreToolUseHookInput = serde_json::from_value(input.clone())
                        .map_err(|e| {
                            CopilotError::Protocol(format!("Invalid preToolUse input: {}", e))
                        })?;
                    let output = handler(hook_input);
                    Ok(serde_json::to_value(output).unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            "postToolUse" => {
                if let Some(handler) = &hooks.on_post_tool_use {
                    let hook_input: PostToolUseHookInput = serde_json::from_value(input.clone())
                        .map_err(|e| {
                            CopilotError::Protocol(format!("Invalid postToolUse input: {}", e))
                        })?;
                    let output = handler(hook_input);
                    Ok(serde_json::to_value(output).unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            "userPromptSubmitted" => {
                if let Some(handler) = &hooks.on_user_prompt_submitted {
                    let hook_input: UserPromptSubmittedHookInput =
                        serde_json::from_value(input.clone()).map_err(|e| {
                            CopilotError::Protocol(format!(
                                "Invalid userPromptSubmitted input: {}",
                                e
                            ))
                        })?;
                    let output = handler(hook_input);
                    Ok(serde_json::to_value(output).unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            "sessionStart" => {
                if let Some(handler) = &hooks.on_session_start {
                    let hook_input: SessionStartHookInput = serde_json::from_value(input.clone())
                        .map_err(|e| {
                        CopilotError::Protocol(format!("Invalid sessionStart input: {}", e))
                    })?;
                    let output = handler(hook_input);
                    Ok(serde_json::to_value(output).unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            "sessionEnd" => {
                if let Some(handler) = &hooks.on_session_end {
                    let hook_input: SessionEndHookInput = serde_json::from_value(input.clone())
                        .map_err(|e| {
                            CopilotError::Protocol(format!("Invalid sessionEnd input: {}", e))
                        })?;
                    let output = handler(hook_input);
                    Ok(serde_json::to_value(output).unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            "errorOccurred" => {
                if let Some(handler) = &hooks.on_error_occurred {
                    let hook_input: ErrorOccurredHookInput = serde_json::from_value(input.clone())
                        .map_err(|e| {
                            CopilotError::Protocol(format!("Invalid errorOccurred input: {}", e))
                        })?;
                    let output = handler(hook_input);
                    Ok(serde_json::to_value(output).unwrap_or(Value::Null))
                } else {
                    Ok(Value::Null)
                }
            }
            _ => Ok(Value::Null),
        }
    }

    // =========================================================================
    // Lifecycle
    // =========================================================================

    /// Destroy the session.
    pub async fn destroy(&self) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session_id,
        });

        (self.invoke_fn)("session.destroy", Some(params)).await?;
        Ok(())
    }

    // =========================================================================
    // Model Management
    // =========================================================================

    /// Get the current model for this session.
    pub async fn get_model(&self) -> Result<String> {
        let params = serde_json::json!({ "sessionId": self.session_id });
        let result = (self.invoke_fn)("session.model.getCurrent", Some(params)).await?;
        result
            .get("modelId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| CopilotError::Protocol("Missing modelId in response".into()))
    }

    /// Switch to a different model mid-session.
    pub async fn set_model(&self, model: &str, options: Option<SetModelOptions>) -> Result<()> {
        let mut params = serde_json::json!({
            "sessionId": self.session_id,
            "modelId": model,
        });
        if let Some(opts) = options {
            if let (Some(obj), Some(extra)) = (
                params.as_object_mut(),
                serde_json::to_value(&opts)?.as_object(),
            ) {
                for (key, value) in extra {
                    obj.insert(key.clone(), value.clone());
                }
            }
        }
        (self.invoke_fn)("session.model.switchTo", Some(params)).await?;
        Ok(())
    }

    /// Patch live session options via `session.options.update`.
    ///
    /// Only the fields you set are sent; everything else is left untouched.
    /// Sending an empty patch is a no-op and performs no RPC.
    pub async fn update_options(&self, options: SessionUpdateOptions) -> Result<()> {
        if options.is_empty() {
            return Ok(());
        }
        let mut params = serde_json::to_value(&options)?;
        if let Some(obj) = params.as_object_mut() {
            obj.insert("sessionId".into(), serde_json::json!(self.session_id));
        }
        (self.invoke_fn)("session.options.update", Some(params)).await?;
        Ok(())
    }

    // =========================================================================
    // Mode Management
    // =========================================================================

    /// Get the current session mode.
    pub async fn get_mode(&self) -> Result<SessionMode> {
        let params = serde_json::json!({ "sessionId": self.session_id });
        let result = (self.invoke_fn)("session.mode.get", Some(params)).await?;
        let mode_str = result
            .get("mode")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CopilotError::Protocol("Missing mode in response".into()))?;
        serde_json::from_value(serde_json::json!(mode_str))
            .map_err(|e| CopilotError::Protocol(format!("Invalid mode '{}': {}", mode_str, e)))
    }

    /// Set the session mode (interactive, plan, or autopilot).
    pub async fn set_mode(&self, mode: SessionMode) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session_id,
            "mode": mode,
        });
        (self.invoke_fn)("session.mode.set", Some(params)).await?;
        Ok(())
    }

    // =========================================================================
    // Session Logging
    // =========================================================================

    /// Add a log entry to the session.
    pub async fn log(&self, message: &str, options: Option<LogOptions>) -> Result<LogResult> {
        let mut params = serde_json::json!({
            "sessionId": self.session_id,
            "message": message,
        });
        if let Some(opts) = options {
            if let Some(level) = opts.level {
                params["level"] = serde_json::to_value(level).unwrap_or_default();
            }
            if let Some(ephemeral) = opts.ephemeral {
                params["ephemeral"] = serde_json::json!(ephemeral);
            }
        }
        let result = (self.invoke_fn)("session.log", Some(params)).await?;
        let event_id = result
            .get("eventId")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        Ok(LogResult { event_id })
    }

    // =========================================================================
    // Plan Management
    // =========================================================================

    /// Read the current plan.
    pub async fn read_plan(&self) -> Result<Option<PlanData>> {
        let params = serde_json::json!({ "sessionId": self.session_id });
        let result = (self.invoke_fn)("session.plan.read", Some(params)).await?;
        if result.is_null() || result.get("content").is_none() {
            return Ok(None);
        }
        serde_json::from_value(result)
            .map(Some)
            .map_err(|e| CopilotError::Protocol(format!("Failed to parse plan: {}", e)))
    }

    /// Update the session plan.
    pub async fn update_plan(&self, plan: &PlanData) -> Result<()> {
        let mut params = serde_json::to_value(plan)
            .map_err(|e| CopilotError::Protocol(format!("Failed to serialize plan: {}", e)))?;
        params["sessionId"] = serde_json::json!(self.session_id);
        (self.invoke_fn)("session.plan.update", Some(params)).await?;
        Ok(())
    }

    /// Delete the session plan.
    pub async fn delete_plan(&self) -> Result<()> {
        let params = serde_json::json!({ "sessionId": self.session_id });
        (self.invoke_fn)("session.plan.delete", Some(params)).await?;
        Ok(())
    }

    // =========================================================================
    // Agent Management
    // =========================================================================

    /// List available agents.
    pub async fn list_agents(&self) -> Result<Vec<AgentInfo>> {
        let params = serde_json::json!({ "sessionId": self.session_id });
        let result = (self.invoke_fn)("session.agent.list", Some(params)).await?;
        let agents = result
            .get("agents")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([]));
        serde_json::from_value(agents)
            .map_err(|e| CopilotError::Protocol(format!("Failed to parse agents: {}", e)))
    }

    /// Get the currently active agent.
    pub async fn get_current_agent(&self) -> Result<Option<AgentInfo>> {
        let params = serde_json::json!({ "sessionId": self.session_id });
        let result = (self.invoke_fn)("session.agent.getCurrent", Some(params)).await?;
        if result.is_null() || result.get("name").is_none() {
            return Ok(None);
        }
        serde_json::from_value(result)
            .map(Some)
            .map_err(|e| CopilotError::Protocol(format!("Failed to parse agent: {}", e)))
    }

    /// Select (activate) a custom agent.
    pub async fn select_agent(&self, name: &str) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session_id,
            "name": name,
        });
        (self.invoke_fn)("session.agent.select", Some(params)).await?;
        Ok(())
    }

    /// Deselect the current custom agent.
    pub async fn deselect_agent(&self) -> Result<()> {
        let params = serde_json::json!({ "sessionId": self.session_id });
        (self.invoke_fn)("session.agent.deselect", Some(params)).await?;
        Ok(())
    }

    // =========================================================================
    // Compaction
    // =========================================================================

    /// Trigger manual context compaction.
    pub async fn compact(&self) -> Result<()> {
        let params = serde_json::json!({ "sessionId": self.session_id });
        (self.invoke_fn)("session.history.compact", Some(params)).await?;
        Ok(())
    }

    // =========================================================================
    // Fleet Management
    // =========================================================================

    /// Start a fleet of parallel agents.
    pub async fn start_fleet(&self, options: Option<FleetStartOptions>) -> Result<()> {
        let mut params = serde_json::json!({ "sessionId": self.session_id });
        if let Some(opts) = options {
            if let Some(prompt) = opts.prompt {
                params["prompt"] = serde_json::json!(prompt);
            }
        }
        (self.invoke_fn)("session.fleet.start", Some(params)).await?;
        Ok(())
    }

    // =========================================================================
    // Shell Operations
    // =========================================================================

    /// Execute a shell command in the session.
    pub async fn shell_exec(&self, options: ShellExecOptions) -> Result<ShellExecResult> {
        let mut params = serde_json::to_value(&options).map_err(|e| {
            CopilotError::Protocol(format!("Failed to serialize shell options: {}", e))
        })?;
        params["sessionId"] = serde_json::json!(self.session_id);
        let result = (self.invoke_fn)("session.shell.exec", Some(params)).await?;
        serde_json::from_value(result)
            .map_err(|e| CopilotError::Protocol(format!("Failed to parse shell result: {}", e)))
    }

    /// Kill a shell process.
    pub async fn shell_kill(&self, process_id: &str, signal: ShellSignal) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session_id,
            "processId": process_id,
            "signal": signal,
        });
        (self.invoke_fn)("session.shell.kill", Some(params)).await?;
        Ok(())
    }

    // =========================================================================
    // Workspace Operations
    // =========================================================================

    /// List files in the session workspace.
    pub async fn workspace_list_files(&self) -> Result<Vec<WorkspaceFile>> {
        let params = serde_json::json!({ "sessionId": self.session_id });
        let result = (self.invoke_fn)("session.workspaces.listFiles", Some(params)).await?;
        let files = result
            .get("files")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([]));
        serde_json::from_value(files)
            .map_err(|e| CopilotError::Protocol(format!("Failed to parse workspace files: {}", e)))
    }

    /// Read a file from the session workspace.
    pub async fn workspace_read_file(&self, path: &str) -> Result<String> {
        let params = serde_json::json!({
            "sessionId": self.session_id,
            "path": path,
        });
        let result = (self.invoke_fn)("session.workspaces.readFile", Some(params)).await?;
        result
            .get("content")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| CopilotError::Protocol("Missing content in response".into()))
    }

    /// Create a file in the session workspace.
    pub async fn workspace_create_file(&self, path: &str, content: &str) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session_id,
            "path": path,
            "content": content,
        });
        (self.invoke_fn)("session.workspaces.createFile", Some(params)).await?;
        Ok(())
    }
}

// =============================================================================
// Convenience methods for waiting on events
// =============================================================================

impl Session {
    /// Default timeout for waiting on session events (60 seconds).
    const DEFAULT_TIMEOUT: Duration = Duration::from_secs(60);

    /// Wait for the session to become idle.
    ///
    /// Returns the last assistant message event, or None if no message was received.
    /// Uses the specified timeout, or 60 seconds if None.
    pub async fn wait_for_idle(&self, timeout: Option<Duration>) -> Result<Option<SessionEvent>> {
        let timeout = timeout.unwrap_or(Self::DEFAULT_TIMEOUT);
        let mut subscription = self.subscribe();
        let mut last_assistant_message: Option<SessionEvent> = None;

        let result = tokio::time::timeout(timeout, async {
            loop {
                match subscription.recv().await {
                    Ok(event) => match &event.data {
                        SessionEventData::AssistantMessage(_) => {
                            last_assistant_message = Some(event);
                        }
                        SessionEventData::AssistantMessageDelta(_) => {
                            // Deltas are intermediate; we track the full message
                        }
                        SessionEventData::SessionIdle(_) => {
                            break;
                        }
                        SessionEventData::SessionError(err) => {
                            return Err(CopilotError::Protocol(format!(
                                "Session error: {}",
                                err.message
                            )));
                        }
                        _ => {}
                    },
                    Err(broadcast::error::RecvError::Closed) => {
                        return Err(CopilotError::ConnectionClosed);
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {
                        // Continue - we missed some events but can recover
                    }
                }
            }
            Ok(())
        })
        .await;

        match result {
            Ok(Ok(())) => Ok(last_assistant_message),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(CopilotError::Timeout(timeout)),
        }
    }

    /// Send a message and wait for the complete response.
    ///
    /// Returns the last `AssistantMessage` event, or `None` if session
    /// became idle without producing an assistant message.
    /// Uses the specified timeout, or 60 seconds if None.
    pub async fn send_and_wait(
        &self,
        options: impl Into<MessageOptions>,
        timeout: Option<Duration>,
    ) -> Result<Option<SessionEvent>> {
        self.send(options).await?;
        self.wait_for_idle(timeout).await
    }

    /// Send a message and wait for the response content as a string.
    ///
    /// Convenience method that collects all assistant message/delta content.
    /// Uses the specified timeout, or 60 seconds if None.
    pub async fn send_and_collect(
        &self,
        options: impl Into<MessageOptions>,
        timeout: Option<Duration>,
    ) -> Result<String> {
        let timeout = timeout.unwrap_or(Self::DEFAULT_TIMEOUT);
        self.send(options).await?;

        let mut subscription = self.subscribe();
        let mut content = String::new();

        let result = tokio::time::timeout(timeout, async {
            loop {
                match subscription.recv().await {
                    Ok(event) => match &event.data {
                        SessionEventData::AssistantMessage(msg) => {
                            content.push_str(&msg.content);
                        }
                        SessionEventData::AssistantMessageDelta(delta) => {
                            content.push_str(&delta.delta_content);
                        }
                        SessionEventData::SessionIdle(_) => {
                            break;
                        }
                        SessionEventData::SessionError(err) => {
                            return Err(CopilotError::Protocol(format!(
                                "Session error: {}",
                                err.message
                            )));
                        }
                        _ => {}
                    },
                    Err(broadcast::error::RecvError::Closed) => {
                        return Err(CopilotError::ConnectionClosed);
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => {}
                }
            }
            Ok(())
        })
        .await;

        match result {
            Ok(Ok(())) => Ok(content),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(CopilotError::Timeout(timeout)),
        }
    }
}

// =============================================================================
// Session UI API (elicitation)
// =============================================================================

/// Interactive UI API for a session, providing elicitation-based dialogs.
///
/// Acquired via [`Session::ui`]. All methods route to `session.ui.*` RPCs and
/// require host elicitation support — check
/// [`SessionCapabilities::supports_elicitation`] before use.
pub struct SessionUi<'a> {
    session: &'a Session,
}

impl SessionUi<'_> {
    /// Request user input via an interactive UI form (elicitation).
    ///
    /// Sends a JSON Schema describing form fields to the CLI host. The host
    /// renders a form dialog and returns the user's response.
    pub async fn elicitation(
        &self,
        message: &str,
        requested_schema: Value,
    ) -> Result<ElicitationResult> {
        self.session.assert_elicitation().await?;
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "message": message,
            "requestedSchema": requested_schema,
        });
        let result = (self.session.invoke_fn)("session.ui.elicitation", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Ask the user a yes/no confirmation question.
    ///
    /// Returns `true` only if the user accepted and confirmed.
    pub async fn confirm(&self, message: &str) -> Result<bool> {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "confirmed": { "type": "boolean", "default": true }
            },
            "required": ["confirmed"]
        });
        let result = self.elicitation(message, schema).await?;
        Ok(result.is_accept()
            && result
                .content
                .as_ref()
                .and_then(|c| c.get("confirmed"))
                .and_then(|v| v.as_bool())
                == Some(true))
    }

    /// Show a selection dialog with the given options.
    ///
    /// Returns the selected value, or `None` if the user declined/cancelled.
    pub async fn select(&self, message: &str, options: &[String]) -> Result<Option<String>> {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "selection": { "type": "string", "enum": options }
            },
            "required": ["selection"]
        });
        let result = self.elicitation(message, schema).await?;
        if result.is_accept() {
            if let Some(sel) = result
                .content
                .as_ref()
                .and_then(|c| c.get("selection"))
                .and_then(|v| v.as_str())
            {
                return Ok(Some(sel.to_string()));
            }
        }
        Ok(None)
    }

    /// Show a text input dialog.
    ///
    /// Returns the entered text, or `None` if the user declined/cancelled.
    pub async fn input(
        &self,
        message: &str,
        options: Option<&UiInputOptions>,
    ) -> Result<Option<String>> {
        let mut field = serde_json::Map::new();
        field.insert("type".to_string(), Value::String("string".to_string()));
        if let Some(opts) = options {
            if let Some(title) = &opts.title {
                field.insert("title".to_string(), Value::String(title.clone()));
            }
            if let Some(description) = &opts.description {
                field.insert(
                    "description".to_string(),
                    Value::String(description.clone()),
                );
            }
            if let Some(min) = opts.min_length {
                field.insert("minLength".to_string(), Value::from(min));
            }
            if let Some(max) = opts.max_length {
                field.insert("maxLength".to_string(), Value::from(max));
            }
            if let Some(format) = &opts.format {
                field.insert("format".to_string(), Value::String(format.clone()));
            }
            if let Some(default) = &opts.default {
                field.insert("default".to_string(), Value::String(default.clone()));
            }
        }
        let schema = serde_json::json!({
            "type": "object",
            "properties": { "value": Value::Object(field) },
            "required": ["value"]
        });
        let result = self.elicitation(message, schema).await?;
        if result.is_accept() {
            if let Some(val) = result
                .content
                .as_ref()
                .and_then(|c| c.get("value"))
                .and_then(|v| v.as_str())
            {
                return Ok(Some(val.to_string()));
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    fn mock_invoke(_method: &str, _params: Option<Value>) -> InvokeFuture {
        Box::pin(async { Ok(serde_json::json!({"messageId": "test-msg-123"})) })
    }

    fn mock_invoke_with_events(method: &str, _params: Option<Value>) -> InvokeFuture {
        let method = method.to_string();
        Box::pin(async move {
            if method == "session.getMessages" {
                return Ok(serde_json::json!({
                    "events": [{
                        "id": "evt-1",
                        "timestamp": "2024-01-01T00:00:00Z",
                        "type": "session.idle",
                        "data": {}
                    }]
                }));
            }
            Ok(serde_json::json!({"messageId": "test-msg-123"}))
        })
    }

    #[tokio::test]
    async fn test_session_id() {
        let session = Session::new("test-session-123".to_string(), None, mock_invoke);
        assert_eq!(session.session_id(), "test-session-123");
    }

    #[tokio::test]
    async fn test_workspace_path() {
        let session = Session::new(
            "test".to_string(),
            Some("/tmp/workspace".to_string()),
            mock_invoke,
        );
        assert_eq!(session.workspace_path(), Some("/tmp/workspace"));
    }

    #[tokio::test]
    async fn test_lsp_server_config() {
        let session = Session::new(
            "session-for-lsp".to_string(),
            Some("/tmp/workspace".to_string()),
            mock_invoke,
        );
        let config = session.lsp_server_config();
        assert_eq!(config.session_id.as_deref(), Some("session-for-lsp"));
        assert_eq!(config.workspace_root.as_deref(), Some("/tmp/workspace"));
    }

    #[tokio::test]
    async fn test_register_tool() {
        let session = Session::new("test".to_string(), None, mock_invoke);

        let tool = Tool::new("my_tool").description("A test tool");

        session.register_tool(tool.clone()).await;

        let retrieved = session.get_tool("my_tool").await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "my_tool");
    }

    #[tokio::test]
    async fn test_register_tool_with_handler() {
        let session = Session::new("test".to_string(), None, mock_invoke);

        let tool = Tool::new("echo").description("Echo tool");
        let handler: ToolHandler = Arc::new(|_name, args| {
            let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("empty");
            ToolResultObject::text(text)
        });

        session
            .register_tool_with_handler(tool, Some(handler))
            .await;

        let result = session
            .invoke_tool("echo", &serde_json::json!({"text": "hello"}))
            .await
            .unwrap();

        assert_eq!(result.text_result_for_llm, "hello");
    }

    #[tokio::test]
    async fn test_invoke_unknown_tool() {
        let session = Session::new("test".to_string(), None, mock_invoke);

        let result = session.invoke_tool("unknown", &serde_json::json!({})).await;

        assert!(matches!(result, Err(CopilotError::ToolNotFound(_))));
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let session = Session::new("test".to_string(), None, mock_invoke);

        let mut sub1 = session.subscribe();
        let mut sub2 = session.subscribe();

        // Dispatch an event
        let event = SessionEvent::from_json(&serde_json::json!({
            "id": "evt-1",
            "timestamp": "2024-01-01T00:00:00Z",
            "type": "session.idle",
            "data": {}
        }))
        .unwrap();

        session.dispatch_event(event).await;

        // Both subscribers should receive it
        let received1 = sub1.recv().await.unwrap();
        let received2 = sub2.recv().await.unwrap();

        assert_eq!(received1.id, "evt-1");
        assert_eq!(received2.id, "evt-1");
    }

    #[tokio::test]
    async fn test_callback_handler() {
        let session = Session::new("test".to_string(), None, mock_invoke);
        let call_count = Arc::new(AtomicUsize::new(0));

        let count_clone = Arc::clone(&call_count);
        let unsubscribe = session
            .on(move |_event| {
                count_clone.fetch_add(1, Ordering::SeqCst);
            })
            .await;

        // Dispatch events
        let event = SessionEvent::from_json(&serde_json::json!({
            "id": "evt-callback-1",
            "timestamp": "2024-01-01T00:00:00Z",
            "type": "session.idle",
            "data": {}
        }))
        .unwrap();

        session.dispatch_event(event).await;

        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Unsubscribe
        unsubscribe();
    }

    #[tokio::test]
    async fn test_permission_handler() {
        let session = Session::new("test".to_string(), None, mock_invoke);

        // Default handler denies
        let request = PermissionRequest {
            kind: "tool_execution".to_string(),
            tool_call_id: Some("call-123".to_string()),
            extension_data: HashMap::new(),
        };
        let result = session.handle_permission_request(&request).await;
        assert!(result.kind.contains("denied"));

        // Register custom handler that approves
        session
            .register_permission_handler(|_req| PermissionRequestResult::approved())
            .await;

        let result = session.handle_permission_request(&request).await;
        assert_eq!(result.kind, "approved");
    }

    #[tokio::test]
    async fn test_dispatch_event_handles_external_tool_requested() {
        let rpc_calls = Arc::new(std::sync::Mutex::new(Vec::<(String, Value)>::new()));
        let rpc_calls_for_invoke = Arc::clone(&rpc_calls);
        let session = Session::new("test".to_string(), None, move |method, params| {
            let method = method.to_string();
            let params = params.unwrap_or(Value::Null);
            let rpc_calls = Arc::clone(&rpc_calls_for_invoke);
            Box::pin(async move {
                rpc_calls.lock().unwrap().push((method, params));
                Ok(serde_json::json!({}))
            })
        });

        session
            .register_tool_with_handler(
                Tool::new("echo").description("Echo tool"),
                Some(Arc::new(|_name, args| {
                    ToolResultObject::text(
                        args.get("text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("missing"),
                    )
                })),
            )
            .await;

        let mut subscription = session.subscribe();
        let event = SessionEvent::from_json(&serde_json::json!({
            "id": "evt-broadcast-tool",
            "timestamp": "2024-01-01T00:00:00Z",
            "type": "external_tool.requested",
            "data": {
                "requestId": "req-tool-1",
                "toolName": "echo",
                "toolCallId": "call-tool-1",
                "arguments": {
                    "text": "hello"
                }
            }
        }))
        .unwrap();

        session.dispatch_event(event).await;

        let forwarded = subscription.recv().await.unwrap();
        assert_eq!(forwarded.event_type, "external_tool.requested");

        let calls = rpc_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "session.tools.handlePendingToolCall");
        assert_eq!(calls[0].1["sessionId"], "test");
        assert_eq!(calls[0].1["requestId"], "req-tool-1");
        assert_eq!(calls[0].1["result"]["textResultForLlm"], "hello");
        assert_eq!(calls[0].1["result"]["resultType"], "success");
    }

    #[tokio::test]
    async fn test_dispatch_event_handles_permission_requested() {
        let rpc_calls = Arc::new(std::sync::Mutex::new(Vec::<(String, Value)>::new()));
        let rpc_calls_for_invoke = Arc::clone(&rpc_calls);
        let seen_request = Arc::new(std::sync::Mutex::new(None::<PermissionRequest>));
        let seen_request_for_handler = Arc::clone(&seen_request);
        let session = Session::new("test".to_string(), None, move |method, params| {
            let method = method.to_string();
            let params = params.unwrap_or(Value::Null);
            let rpc_calls = Arc::clone(&rpc_calls_for_invoke);
            Box::pin(async move {
                rpc_calls.lock().unwrap().push((method, params));
                Ok(serde_json::json!({}))
            })
        });

        session
            .register_permission_handler(move |request| {
                *seen_request_for_handler.lock().unwrap() = Some(request.clone());
                PermissionRequestResult::approved()
            })
            .await;

        let mut subscription = session.subscribe();
        let event = SessionEvent::from_json(&serde_json::json!({
            "id": "evt-broadcast-permission",
            "timestamp": "2024-01-01T00:00:00Z",
            "type": "permission.requested",
            "data": {
                "requestId": "req-perm-1",
                "permissionRequest": {
                    "kind": "tool_execution",
                    "toolCallId": "call-perm-1",
                    "toolName": "shell",
                    "command": "ls"
                }
            }
        }))
        .unwrap();

        session.dispatch_event(event).await;

        let forwarded = subscription.recv().await.unwrap();
        assert_eq!(forwarded.event_type, "permission.requested");

        let request = seen_request.lock().unwrap().clone().unwrap();
        assert_eq!(request.kind, "tool_execution");
        assert_eq!(request.tool_call_id.as_deref(), Some("call-perm-1"));
        assert_eq!(request.extension_data["toolName"], "shell");
        assert_eq!(request.extension_data["command"], "ls");

        let calls = rpc_calls.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(
            calls[0].0,
            "session.permissions.handlePendingPermissionRequest"
        );
        assert_eq!(calls[0].1["sessionId"], "test");
        assert_eq!(calls[0].1["requestId"], "req-perm-1");
        assert_eq!(calls[0].1["result"]["kind"], "approved");
    }

    #[tokio::test]
    async fn test_get_messages_with_events_field() {
        let session = Session::new("test".to_string(), None, mock_invoke_with_events);
        let messages = session.get_messages().await.unwrap();
        assert_eq!(messages.len(), 1);
        assert!(matches!(
            messages[0].data,
            crate::events::SessionEventData::SessionIdle(_)
        ));
    }

    #[tokio::test]
    async fn test_user_input_handler() {
        let session = Session::new("test".to_string(), None, mock_invoke);

        session
            .register_user_input_handler(|req, _inv| {
                assert_eq!(req.question, "What color?");
                UserInputResponse {
                    answer: "blue".into(),
                    was_freeform: Some(true),
                }
            })
            .await;

        let request = UserInputRequest {
            question: "What color?".into(),
            choices: Some(vec!["red".into(), "blue".into()]),
            allow_freeform: Some(true),
        };

        let response = session.handle_user_input_request(&request).await.unwrap();
        assert_eq!(response.answer, "blue");
        assert_eq!(response.was_freeform, Some(true));
    }

    #[tokio::test]
    async fn test_user_input_no_handler_errors() {
        let session = Session::new("test".to_string(), None, mock_invoke);

        let request = UserInputRequest {
            question: "?".into(),
            choices: None,
            allow_freeform: None,
        };

        let result = session.handle_user_input_request(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_hooks() {
        let session = Session::new("test".to_string(), None, mock_invoke);

        assert!(!session.has_hooks().await);

        let hooks = crate::types::SessionHooks {
            on_pre_tool_use: Some(Arc::new(|input| {
                assert_eq!(input.tool_name, "my_tool");
                crate::types::PreToolUseHookOutput {
                    permission_decision: Some("allow".into()),
                    ..Default::default()
                }
            })),
            ..Default::default()
        };

        session.register_hooks(hooks).await;
        assert!(session.has_hooks().await);
    }

    #[tokio::test]
    async fn test_hooks_invoke_pre_tool_use() {
        let session = Session::new("test".to_string(), None, mock_invoke);

        let hooks = crate::types::SessionHooks {
            on_pre_tool_use: Some(Arc::new(|_input| crate::types::PreToolUseHookOutput {
                permission_decision: Some("allow".into()),
                additional_context: Some("extra context".into()),
                ..Default::default()
            })),
            ..Default::default()
        };

        session.register_hooks(hooks).await;

        let input = serde_json::json!({
            "timestamp": 1234567890,
            "cwd": "/tmp",
            "toolName": "test_tool",
            "toolArgs": {"key": "value"}
        });

        let result = session
            .handle_hooks_invoke("preToolUse", &input)
            .await
            .unwrap();
        assert_eq!(
            result.get("permissionDecision").and_then(|v| v.as_str()),
            Some("allow")
        );
        assert_eq!(
            result.get("additionalContext").and_then(|v| v.as_str()),
            Some("extra context")
        );
    }

    #[tokio::test]
    async fn test_hooks_invoke_no_handler_returns_null() {
        let session = Session::new("test".to_string(), None, mock_invoke);

        // No hooks registered at all
        let result = session
            .handle_hooks_invoke("preToolUse", &serde_json::json!({}))
            .await
            .unwrap();
        assert!(result.is_null());

        // Hooks registered but not for this type
        let hooks = crate::types::SessionHooks {
            on_session_start: Some(Arc::new(|_input| {
                crate::types::SessionStartHookOutput::default()
            })),
            ..Default::default()
        };
        session.register_hooks(hooks).await;

        let input = serde_json::json!({
            "timestamp": 1234567890,
            "cwd": "/tmp",
            "toolName": "test_tool",
            "toolArgs": {}
        });
        let result = session
            .handle_hooks_invoke("preToolUse", &input)
            .await
            .unwrap();
        assert!(result.is_null());
    }

    #[tokio::test]
    async fn test_hooks_invoke_unknown_type_returns_null() {
        let session = Session::new("test".to_string(), None, mock_invoke);

        let hooks = crate::types::SessionHooks {
            on_pre_tool_use: Some(Arc::new(|_| crate::types::PreToolUseHookOutput::default())),
            ..Default::default()
        };
        session.register_hooks(hooks).await;

        let result = session
            .handle_hooks_invoke("unknownHookType", &serde_json::json!({}))
            .await
            .unwrap();
        assert!(result.is_null());
    }

    // =========================================================================
    // Wave 2: capabilities / elicitation / UI tests
    // =========================================================================

    fn mock_invoke_elicitation_accept(method: &str, _params: Option<Value>) -> InvokeFuture {
        let method = method.to_string();
        Box::pin(async move {
            if method == "session.ui.elicitation" {
                return Ok(serde_json::json!({
                    "action": "accept",
                    "content": { "confirmed": true, "selection": "b", "value": "typed" }
                }));
            }
            Ok(serde_json::json!({}))
        })
    }

    fn supported_caps() -> crate::types::SessionCapabilities {
        crate::types::SessionCapabilities {
            ui: Some(crate::types::UiCapabilities {
                elicitation: Some(true),
                ..Default::default()
            }),
        }
    }

    #[tokio::test]
    async fn test_capabilities_default_and_set() {
        let session = Session::new("test".to_string(), None, mock_invoke);
        assert!(!session.capabilities().await.supports_elicitation());

        session.set_capabilities(supported_caps()).await;
        assert!(session.capabilities().await.supports_elicitation());
    }

    fn capabilities_changed_event(
        elicitation: Option<bool>,
        canvases: Option<bool>,
    ) -> SessionEvent {
        SessionEvent {
            id: "evt-caps".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            event_type: "capabilities.changed".to_string(),
            parent_id: None,
            ephemeral: None,
            data: SessionEventData::CapabilitiesChanged(crate::events::CapabilitiesChangedData {
                ui: Some(crate::events::CapabilitiesChangedUi {
                    canvases,
                    elicitation,
                    mcp_apps: None,
                }),
            }),
        }
    }

    /// Regression: `capabilities()` used to report only what the host advertised
    /// at create/resume time, so it went stale as soon as the host sent
    /// `capabilities.changed`.
    #[tokio::test]
    async fn test_capabilities_changed_event_updates_cached_capabilities() {
        let session = Session::new("test".to_string(), None, mock_invoke);
        assert!(!session.capabilities().await.supports_elicitation());
        assert!(!session.capabilities().await.supports_canvases());

        session
            .dispatch_event(capabilities_changed_event(Some(true), Some(true)))
            .await;

        assert!(session.capabilities().await.supports_elicitation());
        assert!(session.capabilities().await.supports_canvases());

        // A later event revoking a capability must also be reflected.
        session
            .dispatch_event(capabilities_changed_event(Some(false), Some(true)))
            .await;

        assert!(!session.capabilities().await.supports_elicitation());
        assert!(session.capabilities().await.supports_canvases());
    }

    /// A `capabilities.changed` payload with no `ui` member must leave the
    /// cached capabilities untouched, matching the upstream shallow merge.
    #[tokio::test]
    async fn test_capabilities_changed_without_ui_preserves_existing() {
        let session = Session::new("test".to_string(), None, mock_invoke);
        session.set_capabilities(supported_caps()).await;

        session
            .dispatch_event(SessionEvent {
                id: "evt-caps-empty".to_string(),
                timestamp: "2024-01-01T00:00:00Z".to_string(),
                event_type: "capabilities.changed".to_string(),
                parent_id: None,
                ephemeral: None,
                data: SessionEventData::CapabilitiesChanged(
                    crate::events::CapabilitiesChangedData { ui: None },
                ),
            })
            .await;

        assert!(session.capabilities().await.supports_elicitation());
    }

    #[tokio::test]
    async fn test_ui_elicitation_requires_capability() {
        let session = Session::new("test".to_string(), None, mock_invoke);
        let err = session.ui().confirm("Proceed?").await;
        assert!(matches!(err, Err(CopilotError::Protocol(_))));
    }

    #[tokio::test]
    async fn test_ui_confirm_accept() {
        let session = Session::new("test".to_string(), None, mock_invoke_elicitation_accept);
        session.set_capabilities(supported_caps()).await;
        assert!(session.ui().confirm("Proceed?").await.unwrap());
    }

    #[tokio::test]
    async fn test_ui_select_returns_choice() {
        let session = Session::new("test".to_string(), None, mock_invoke_elicitation_accept);
        session.set_capabilities(supported_caps()).await;
        let choice = session
            .ui()
            .select("Pick", &["a".to_string(), "b".to_string()])
            .await
            .unwrap();
        assert_eq!(choice.as_deref(), Some("b"));
    }

    #[tokio::test]
    async fn test_ui_input_returns_text() {
        let session = Session::new("test".to_string(), None, mock_invoke_elicitation_accept);
        session.set_capabilities(supported_caps()).await;
        let value = session.ui().input("Name?", None).await.unwrap();
        assert_eq!(value.as_deref(), Some("typed"));
    }

    #[tokio::test]
    async fn test_elicitation_result_helpers() {
        let cancelled = crate::types::ElicitationResult::cancel();
        assert!(!cancelled.is_accept());
        assert_eq!(cancelled.action, "cancel");

        let accepted = crate::types::ElicitationResult {
            action: "accept".to_string(),
            content: None,
        };
        assert!(accepted.is_accept());
    }

    #[tokio::test]
    async fn test_register_and_unregister_command() {
        let session = Session::new("test".to_string(), None, mock_invoke);
        session.register_command("deploy", |_ctx| {}).await;
        {
            let state = session.state.read().await;
            assert!(state.command_handlers.contains_key("deploy"));
        }
        session.unregister_command("deploy").await;
        let state = session.state.read().await;
        assert!(!state.command_handlers.contains_key("deploy"));
    }

    #[tokio::test]
    async fn test_register_wave2_handlers() {
        let session = Session::new("test".to_string(), None, mock_invoke);
        session
            .register_elicitation_handler(|_ctx| crate::types::ElicitationResult::cancel())
            .await;
        session
            .register_exit_plan_mode_handler(|_req| crate::types::ExitPlanModeResult::default())
            .await;
        session
            .register_auto_mode_switch_handler(|_req| crate::types::AutoModeSwitchResponse::No)
            .await;
        let state = session.state.read().await;
        assert!(state.elicitation_handler.is_some());
        assert!(state.exit_plan_mode_handler.is_some());
        assert!(state.auto_mode_switch_handler.is_some());
    }

    struct TestCanvasHandler;
    impl crate::canvas::CanvasHandler for TestCanvasHandler {
        fn on_open(
            &self,
            request: crate::canvas::CanvasOpenRequest,
        ) -> std::result::Result<crate::canvas::CanvasOpenResult, crate::canvas::CanvasError>
        {
            Ok(crate::canvas::CanvasOpenResult {
                status: Some(format!("opened:{}", request.canvas_id)),
                title: None,
            })
        }

        fn on_action(
            &self,
            request: crate::canvas::CanvasInvokeActionRequest,
        ) -> std::result::Result<Value, crate::canvas::CanvasError> {
            Ok(serde_json::json!({ "action": request.action_name }))
        }
    }

    #[tokio::test]
    async fn test_register_canvas_handler() {
        let session = Session::new("test".to_string(), None, mock_invoke);
        assert!(session.canvas_handler().await.is_none());

        session
            .register_canvas_handler(Arc::new(TestCanvasHandler))
            .await;
        let handler = session.canvas_handler().await;
        assert!(handler.is_some());

        let result = handler
            .unwrap()
            .on_open(crate::canvas::CanvasOpenRequest {
                session_id: "test".to_string(),
                canvas_id: "charts".to_string(),
                instance_id: "i1".to_string(),
                ..Default::default()
            })
            .unwrap();
        assert_eq!(result.status.as_deref(), Some("opened:charts"));
    }

    // =========================================================================
    // Session filesystem + system message transform
    // =========================================================================

    #[tokio::test]
    async fn test_session_fs_provider_registration() {
        struct Noop;
        impl crate::session_fs::SessionFsProvider for Noop {
            fn read_file<'a>(
                &'a self,
                _path: &'a str,
            ) -> crate::session_fs::SessionFsFuture<'a, String> {
                Box::pin(async { Ok(String::new()) })
            }
            fn write_file<'a>(
                &'a self,
                _path: &'a str,
                _content: &'a str,
                _mode: Option<u32>,
            ) -> crate::session_fs::SessionFsFuture<'a, ()> {
                Box::pin(async { Ok(()) })
            }
            fn exists<'a>(
                &'a self,
                _path: &'a str,
            ) -> crate::session_fs::SessionFsFuture<'a, bool> {
                Box::pin(async { Ok(false) })
            }
            fn stat<'a>(
                &'a self,
                path: &'a str,
            ) -> crate::session_fs::SessionFsFuture<'a, crate::session_fs::SessionFsFileInfo>
            {
                Box::pin(async move { Err(crate::session_fs::SessionFsError::not_found(path)) })
            }
            fn readdir<'a>(
                &'a self,
                _path: &'a str,
            ) -> crate::session_fs::SessionFsFuture<'a, Vec<String>> {
                Box::pin(async { Ok(Vec::new()) })
            }
            fn readdir_with_types<'a>(
                &'a self,
                _path: &'a str,
            ) -> crate::session_fs::SessionFsFuture<'a, Vec<crate::session_fs::SessionFsDirEntry>>
            {
                Box::pin(async { Ok(Vec::new()) })
            }
            fn mkdir<'a>(
                &'a self,
                _path: &'a str,
                _recursive: bool,
                _mode: Option<u32>,
            ) -> crate::session_fs::SessionFsFuture<'a, ()> {
                Box::pin(async { Ok(()) })
            }
            fn rm<'a>(
                &'a self,
                _path: &'a str,
                _recursive: bool,
                _force: bool,
            ) -> crate::session_fs::SessionFsFuture<'a, ()> {
                Box::pin(async { Ok(()) })
            }
            fn rename<'a>(
                &'a self,
                _src: &'a str,
                _dest: &'a str,
            ) -> crate::session_fs::SessionFsFuture<'a, ()> {
                Box::pin(async { Ok(()) })
            }
        }

        let session = Session::new("s".to_string(), None, mock_invoke);
        assert!(session.session_fs_provider().await.is_none());
        session.register_session_fs_provider(Arc::new(Noop)).await;
        assert!(session.session_fs_provider().await.is_some());
    }

    #[tokio::test]
    async fn test_transform_callbacks_are_registered_and_applied() {
        let session = Session::new("s".to_string(), None, mock_invoke);
        assert!(session.transform_callback_ids().await.is_empty());

        let mut callbacks: HashMap<String, SectionTransformFn> = HashMap::new();
        callbacks.insert(
            "identity".to_string(),
            Arc::new(|content: String| Box::pin(async move { format!("<{content}>") }) as _),
        );
        session.register_transform_callbacks(callbacks).await;
        assert_eq!(session.transform_callback_ids().await, vec!["identity"]);

        let mut input = HashMap::new();
        input.insert("identity".to_string(), "core".to_string());
        input.insert("tone".to_string(), "untouched".to_string());

        let out = session.handle_system_message_transform(input).await;
        assert_eq!(out.get("identity").unwrap(), "<core>");
        assert_eq!(out.get("tone").unwrap(), "untouched");
    }
}
