// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

#![forbid(unsafe_code)]

//! # Copilot SDK for Rust
//!
//! A Rust SDK for interacting with the GitHub Copilot CLI.
//!
//! ## Quick Start
//!
//! ```no_run
//! use copilot_sdk::{Client, SessionConfig, SessionEventData};
//!
//! #[tokio::main]
//! async fn main() -> copilot_sdk::Result<()> {
//!     let client = Client::builder().build()?;
//!     client.start().await?;
//!
//!     let session = client.create_session(SessionConfig::default()).await?;
//!     let mut events = session.subscribe();
//!
//!     session.send("What is the capital of France?").await?;
//!
//!     while let Ok(event) = events.recv().await {
//!         match &event.data {
//!             SessionEventData::AssistantMessage(msg) => println!("{}", msg.content),
//!             SessionEventData::SessionIdle(_) => break,
//!             _ => {}
//!         }
//!     }
//!
//!     client.stop().await;
//!     Ok(())
//! }
//! ```

pub mod canvas;
pub mod client;
pub mod error;
pub mod events;
pub mod generated;
pub mod jsonrpc;
pub mod lsp;
pub mod process;
pub mod rpc;
pub mod session;
pub mod session_fs;
pub mod tools;
pub mod toolset;
pub mod trace;
pub mod transport;
pub mod types;

// Re-export tool utilities
pub use tools::{convert_mcp_call_tool_result, define_tool};

// Re-export canvas declaration model
pub use canvas::{
    CanvasActionDeclaration, CanvasBuilder, CanvasCloseRequest, CanvasDeclaration, CanvasError,
    CanvasHandler, CanvasInstanceAvailability, CanvasInvokeActionRequest, CanvasJsonSchema,
    CanvasOpenRequest, CanvasOpenResult, OpenCanvasInstance,
};

// Re-export tool-set builder
pub use toolset::{BuiltInTools, InvalidToolName, ToolSet};

// Re-export client-provided session filesystem
pub use session_fs::{
    SessionFsCapabilities, SessionFsConfig, SessionFsConventions, SessionFsDirEntry,
    SessionFsEntryType, SessionFsError, SessionFsErrorCode, SessionFsFileInfo, SessionFsFuture,
    SessionFsProvider, SessionFsResult, SessionFsSqliteParams, SessionFsSqliteProvider,
    SessionFsSqliteQueryResult, SessionFsSqliteQueryType,
};

// Re-export trace-context helpers
pub use trace::{get_trace_context, TraceContext, TraceContextFuture, TraceContextProvider};

// Re-export main types at crate root for convenience
pub use error::{CopilotError, Result};
pub use types::{
    // Free functions / helpers
    approve_all,
    default_join_session_permission_handler,
    session_lifecycle_event_types,
    system_message_sections,
    // Config types
    AgentInfo,
    // Enums
    AttachmentType,
    AutoModeSwitchRequest,
    AutoModeSwitchResponse,
    AzureOptions,
    ClientOptions,
    // Cloud / remote session types
    CloudSessionOptions,
    CloudSessionRepository,
    // Command types
    CommandContext,
    CommandDeclaration,
    ConnectionState,
    CustomAgentConfig,
    DefaultAgentConfig,
    // UI elicitation types
    ElicitationContext,
    ElicitationMode,
    ElicitationParams,
    ElicitationResult,
    // Hook types
    ErrorOccurredHandler,
    ErrorOccurredHookInput,
    ErrorOccurredHookOutput,
    ExitPlanModeRequest,
    ExitPlanModeResult,
    ExtensionInfo,
    FleetStartOptions,
    // Response types
    GetAuthStatusResponse,
    GetForegroundSessionResponse,
    GetStatusResponse,
    InfiniteSessionConfig,
    LargeToolOutputConfig,
    LogLevel,
    LogOptions,
    LogResult,
    LspInitializeOptions,
    McpLocalServerConfig,
    McpRemoteServerConfig,
    McpServerConfig,
    MessageOptions,
    ModelBilling,
    ModelCapabilities,
    // Model capability override types
    ModelCapabilitiesOverride,
    ModelCapabilitiesOverrideLimits,
    ModelCapabilitiesOverrideLimitsVision,
    ModelCapabilitiesOverrideSupports,
    ModelInfo,
    ModelLimits,
    ModelPolicy,
    ModelSupports,
    ModelVisionLimits,
    // Permission types
    PermissionRequest,
    PermissionRequestResult,
    PingResponse,
    PlanData,
    PostToolUseHandler,
    PostToolUseHookInput,
    PostToolUseHookOutput,
    PreToolUseHandler,
    PreToolUseHookInput,
    PreToolUseHookOutput,
    ProviderConfig,
    // Quota types
    QuotaResult,
    QuotaSnapshot,
    ReasoningSummary,
    RemoteSessionMode,
    ResumeSessionConfig,
    // System message section override types
    SectionOverride,
    SectionOverrideAction,
    // Selection types
    SelectionAttachment,
    SelectionPosition,
    SelectionRange,
    SessionCapabilities,
    SessionConfig,
    SessionEndHandler,
    SessionEndHookInput,
    SessionEndHookOutput,
    SessionHooks,
    SessionInstalledPlugin,
    // Session lifecycle types
    SessionLifecycleEvent,
    SessionLifecycleEventMetadata,
    SessionLogLevel,
    SessionMetadata,
    SessionMode,
    SessionStartHandler,
    SessionStartHookInput,
    SessionStartHookOutput,
    SessionUpdateOptions,
    SetForegroundSessionResponse,
    SetModelOptions,
    // Shell types
    ShellExecOptions,
    ShellExecResult,
    ShellSignal,
    StopError,
    StorageMode,
    SystemMessageConfig,
    SystemMessageMode,
    SystemMessageSection,
    // Telemetry types
    TelemetryConfig,
    // Tool types
    Tool,
    ToolBinaryResult,
    ToolInfo,
    ToolInvocation,
    ToolResult,
    ToolResultObject,
    ToolsListResult,
    UiCapabilities,
    UiInputOptions,
    // User input types
    UserInputInvocation,
    UserInputRequest,
    UserInputResponse,
    UserMessageAttachment,
    UserPromptSubmittedHandler,
    UserPromptSubmittedHookInput,
    UserPromptSubmittedHookOutput,
    // Workspace types
    WorkspaceFile,
    // Constants
    SDK_PROTOCOL_VERSION,
};

// Re-export event types
pub use events::{
    // Event data types
    AbortData,
    AssistantIntentData,
    AssistantMessageData,
    AssistantMessageDeltaData,
    AssistantMessageStartData,
    AssistantReasoningData,
    AssistantReasoningDeltaData,
    AssistantStreamingDeltaData,
    AssistantTurnEndData,
    AssistantTurnStartData,
    AssistantUsageData,
    AutoModeSwitchCompletedData,
    AutopilotObjectiveChangedData,
    AutopilotObjectiveOperation,
    AutopilotObjectiveStatus,
    BackgroundTasksChangedData,
    CanvasOpenedData,
    CanvasRegistryChangedData,
    CapabilitiesChangedData,
    CapabilitiesChangedUi,
    ChangedCommand,
    CommandCompletedData,
    CommandQueuedData,
    CommandsChangedData,
    CompactionTokensUsed,
    CustomAgentCompletedData,
    CustomAgentDeselectedData,
    CustomAgentFailedData,
    CustomAgentSelectedData,
    CustomAgentStartedData,
    CustomAgentsUpdatedData,
    CustomNotificationData,
    ElicitationCompletedAction,
    ElicitationCompletedData,
    ExitPlanModeAction,
    ExitPlanModeCompletedData,
    ExtensionSource,
    ExtensionStatus,
    ExtensionsLoadedData,
    ExternalToolCompletedData,
    HandoffSourceType,
    HookEndData,
    HookError,
    HookProgressData,
    HookStartData,
    LoadedExtension,
    LoadedMcpServer,
    LoadedSkill,
    McpAppToolCallCompleteData,
    McpAppToolCallError,
    McpAppToolMeta,
    McpAppToolMetaUi,
    McpOauthCompletedData,
    McpOauthRequiredData,
    McpOauthStaticClientConfig,
    McpServerSource,
    McpServerStatus,
    McpServerStatusChangedData,
    McpServerTransport,
    McpServersLoadedData,
    ModeChangedData,
    ModelCallFailureData,
    ModelCallFailureSource,
    PendingMessagesModifiedData,
    PermissionCompletedData,
    PermissionResult,
    PermissionRule,
    PermissionsChangedData,
    PlanChangedData,
    PlanChangedOperation,
    // Main event types
    RawSessionEvent,
    RegisteredCanvas,
    RegisteredCanvasAction,
    RemoteSteerableChangedData,
    RepositoryInfo,
    SamplingCompletedData,
    SamplingRequestedData,
    ScheduleCancelledData,
    ScheduleCreatedData,
    SessionCompactionCompleteData,
    SessionCompactionStartData,
    SessionErrorData,
    SessionEvent,
    SessionEventData,
    SessionHandoffData,
    SessionIdleData,
    SessionInfoData,
    SessionModelChangeData,
    SessionResumeData,
    SessionShutdownData,
    SessionSnapshotRewindData,
    SessionStartData,
    SessionTruncationData,
    SessionUsageInfoData,
    SessionWarningData,
    ShutdownCodeChanges,
    ShutdownType,
    SkillInvokedData,
    SkillSource,
    SkillsLoadedData,
    SystemMessageEventData,
    SystemMessageMetadata,
    SystemMessageRole,
    SystemNotification,
    SystemNotificationData,
    TaskCompleteData,
    TitleChangedData,
    ToolExecutionCompleteData,
    ToolExecutionError,
    ToolExecutionPartialResultData,
    ToolExecutionProgressData,
    ToolExecutionStartData,
    ToolRequestItem,
    ToolResultContent,
    ToolUserRequestedData,
    ToolsUpdatedData,
    UpdatedCustomAgent,
    UserInputCompletedData,
    UserInputRequestedData,
    UserMessageAttachmentItem,
    UserMessageData,
    WorkingDirectoryContextData,
    WorkingDirectoryHostType,
    WorkspaceFileChangedData,
    WorkspaceFileChangedOperation,
};

// Re-export transport types
pub use transport::{MessageFramer, StdioTransport, Transport};

// Re-export JSON-RPC types
pub use jsonrpc::{
    JsonRpcClient, JsonRpcError, JsonRpcId, JsonRpcRequest, JsonRpcResponse, NotificationHandler,
    RequestHandler,
};

// Re-export LSP types
pub use lsp::{
    CanonicalName, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DocumentStore, DocumentSymbol, DocumentSymbolParams, Hover,
    InitializeParams, InitializeResult, Location, LspServer, LspServerConfig, Position, Range,
    RustSemanticProvider, SemanticId, SemanticProvider, ServerCapabilities, ServerInfo, StrongName,
    SymbolInformation, SymbolKind, TextDocumentContentChangeEvent, TextDocumentIdentifier,
    TextDocumentItem, VersionedTextDocumentIdentifier, WorkspaceFolder, WorkspaceSymbolParams,
    LSP_PROTOCOL_VERSION, SEMANTIC_ID_PREFIX, SEMANTIC_SCHEMA_VERSION,
};

// Re-export process types
pub use process::{
    find_copilot_cli, find_executable, find_node, is_node_script, CopilotProcess, ProcessOptions,
};

// Re-export session types
pub use session::{
    AutoModeSwitchHandler, CommandHandler, ElicitationHandler, EventHandler, EventSubscription,
    ExitPlanModeHandler, InvokeFuture, PermissionHandler, RegisteredTool, Session, SessionUi,
    ToolHandler, UserInputHandler,
};

// Re-export client types
pub use client::{Client, ClientBuilder, LifecycleHandler};
