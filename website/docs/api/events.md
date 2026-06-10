---
id: events
title: "API: events"
sidebar_label: events
---

# Module `events`

The typed streaming event model. Re-exported at the crate root. See the [Events concept](/docs/core-concepts/events) for usage.

## `SessionEvent`

A parsed session event with typed data.

```rust
pub struct SessionEvent {
    pub id: String,
    pub timestamp: String,
    pub event_type: String,
    pub parent_id: Option<String>,
    pub ephemeral: Option<bool>,
    pub data: SessionEventData,
}
```

### Methods

```rust
impl SessionEvent {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error>;
    pub fn from_raw(raw: RawSessionEvent) -> Self;

    // Predicates
    pub fn is_assistant_message(&self) -> bool;
    pub fn is_assistant_message_delta(&self) -> bool;
    pub fn is_session_idle(&self) -> bool;
    pub fn is_session_error(&self) -> bool;
    pub fn is_terminal(&self) -> bool; // idle or error

    // Typed accessors
    pub fn as_assistant_message(&self) -> Option<&AssistantMessageData>;
    pub fn as_assistant_message_delta(&self) -> Option<&AssistantMessageDeltaData>;
    pub fn as_session_error(&self) -> Option<&SessionErrorData>;
    pub fn as_tool_execution_complete(&self) -> Option<&ToolExecutionCompleteData>;
    pub fn content(&self) -> Option<&str>; // best-effort text content
}
```

## `RawSessionEvent`

The untyped event as received from the CLI, before parsing.

```rust
pub struct RawSessionEvent {
    pub id: String,
    pub timestamp: String,
    pub event_type: String,
    pub parent_id: Option<String>,
    pub ephemeral: Option<bool>,
    pub data: serde_json::Value,
}
```

## `SessionEventData`

The tagged union of all event payloads. Includes an `Unknown` fallback for forward compatibility.

```rust
pub enum SessionEventData {
    SessionStart(SessionStartData),
    SessionResume(SessionResumeData),
    SessionError(SessionErrorData),
    SessionIdle(SessionIdleData),
    SessionInfo(SessionInfoData),
    SessionModelChange(SessionModelChangeData),
    SessionHandoff(SessionHandoffData),
    SessionTruncation(SessionTruncationData),
    UserMessage(UserMessageData),
    PendingMessagesModified(PendingMessagesModifiedData),
    AssistantTurnStart(AssistantTurnStartData),
    AssistantIntent(AssistantIntentData),
    AssistantReasoning(AssistantReasoningData),
    AssistantReasoningDelta(AssistantReasoningDeltaData),
    AssistantMessage(AssistantMessageData),
    AssistantMessageDelta(AssistantMessageDeltaData),
    AssistantTurnEnd(AssistantTurnEndData),
    AssistantUsage(AssistantUsageData),
    Abort(AbortData),
    ToolUserRequested(ToolUserRequestedData),
    ToolExecutionStart(ToolExecutionStartData),
    ToolExecutionPartialResult(ToolExecutionPartialResultData),
    ToolExecutionComplete(ToolExecutionCompleteData),
    ToolExecutionProgress(ToolExecutionProgressData),
    CustomAgentStarted(CustomAgentStartedData),
    CustomAgentCompleted(CustomAgentCompletedData),
    CustomAgentFailed(CustomAgentFailedData),
    CustomAgentSelected(CustomAgentSelectedData),
    HookStart(HookStartData),
    HookEnd(HookEndData),
    SystemMessage(SystemMessageEventData),
    SessionCompactionStart(SessionCompactionStartData),
    SessionCompactionComplete(SessionCompactionCompleteData),
    SessionShutdown(SessionShutdownData),
    SessionSnapshotRewind(SessionSnapshotRewindData),
    SessionUsageInfo(SessionUsageInfoData),
    SkillInvoked(SkillInvokedData),
    ExternalToolRequested(ExternalToolRequestedData),
    PermissionRequested(PermissionRequestedData),
    Unknown(serde_json::Value),
}
```

## Enums and shared structs

```rust
pub enum HandoffSourceType { Remote, Local }
pub enum SystemMessageRole { System, Developer }
pub enum ShutdownType { Routine, Error }

pub struct RepositoryInfo { pub owner: String, pub name: String, pub branch: Option<String> }
pub struct UserMessageAttachmentItem { pub attachment_type: AttachmentType, pub path: String, pub display_name: String }
pub struct ToolRequestItem { pub tool_call_id: String, pub name: String, pub arguments: Option<serde_json::Value> }
pub struct ToolResultContent { pub content: String }
pub struct ToolExecutionError { pub message: String, pub code: Option<String> }
pub struct HookError { pub message: String, pub stack: Option<String> }
pub struct SystemMessageMetadata { pub prompt_version: Option<String>, pub variables: Option<HashMap<String, serde_json::Value>> }
```

## Event data structs

### Session lifecycle

```rust
pub struct SessionStartData {
    pub session_id: String, pub version: f64, pub producer: String,
    pub copilot_version: String, pub start_time: String, pub selected_model: Option<String>,
}
pub struct SessionResumeData { pub resume_time: String, pub event_count: f64 }
pub struct SessionErrorData {
    pub error_type: String, pub message: String, pub stack: Option<String>,
    pub code: Option<f64>, pub provider_call_id: Option<String>,
}
pub struct SessionIdleData {}
pub struct SessionInfoData { pub info_type: String, pub message: String }
pub struct SessionModelChangeData { pub previous_model: Option<String>, pub new_model: String }
pub struct SessionHandoffData {
    pub handoff_time: String, pub source_type: HandoffSourceType, pub repository: Option<RepositoryInfo>,
    pub context: Option<String>, pub summary: Option<String>, pub remote_session_id: Option<String>,
}
pub struct SessionTruncationData {
    pub token_limit: f64, pub pre_truncation_tokens_in_messages: f64, pub pre_truncation_messages_length: f64,
    pub post_truncation_tokens_in_messages: f64, pub post_truncation_messages_length: f64,
    pub tokens_removed_during_truncation: f64, pub messages_removed_during_truncation: f64, pub performed_by: String,
}
pub struct SessionUsageInfoData { pub token_limit: f64, pub current_tokens: f64, pub messages_length: f64 }
pub struct SessionSnapshotRewindData { pub up_to_event_id: String, pub events_removed: f64 }
pub struct ShutdownCodeChanges { pub lines_added: f64, pub lines_removed: f64, pub files_modified: Vec<String> }
pub struct SessionShutdownData {
    pub shutdown_type: ShutdownType, pub error_reason: Option<String>, pub total_premium_requests: f64,
    pub total_api_duration_ms: f64, pub session_start_time: f64, pub code_changes: ShutdownCodeChanges,
    pub model_metrics: HashMap<String, serde_json::Value>, pub current_model: Option<String>,
}
```

### Assistant output

```rust
pub struct AssistantTurnStartData { pub turn_id: String }
pub struct AssistantIntentData { pub intent: String }
pub struct AssistantReasoningData { pub reasoning_id: String, pub content: String, pub chunk_content: Option<String> }
pub struct AssistantReasoningDeltaData { pub reasoning_id: String, pub delta_content: String }
pub struct AssistantMessageData {
    pub message_id: String, pub content: String, pub chunk_content: Option<String>,
    pub total_response_size_bytes: Option<f64>, pub tool_requests: Option<Vec<ToolRequestItem>>,
    pub parent_tool_call_id: Option<String>,
}
pub struct AssistantMessageDeltaData {
    pub message_id: String, pub delta_content: String,
    pub total_response_size_bytes: Option<f64>, pub parent_tool_call_id: Option<String>,
}
pub struct AssistantTurnEndData { pub turn_id: String }
pub struct AssistantUsageData {
    pub model: Option<String>, pub input_tokens: Option<f64>, pub output_tokens: Option<f64>,
    pub cache_read_tokens: Option<f64>, pub cache_write_tokens: Option<f64>, pub cost: Option<f64>,
    pub duration: Option<f64>, pub initiator: Option<String>, pub api_call_id: Option<String>,
    pub provider_call_id: Option<String>, pub quota_snapshots: Option<HashMap<String, serde_json::Value>>,
}
pub struct AbortData { pub reason: String }
pub struct UserMessageData {
    pub content: String, pub transformed_content: Option<String>,
    pub attachments: Option<Vec<UserMessageAttachmentItem>>, pub source: Option<String>,
}
pub struct PendingMessagesModifiedData {}
```

### Tools

```rust
pub struct ToolUserRequestedData { pub tool_call_id: String, pub tool_name: String, pub arguments: Option<serde_json::Value> }
pub struct ToolExecutionStartData { pub tool_call_id: String, pub tool_name: String, pub arguments: Option<serde_json::Value>, pub parent_tool_call_id: Option<String> }
pub struct ToolExecutionPartialResultData { pub tool_call_id: String, pub partial_output: String }
pub struct ToolExecutionProgressData { pub tool_call_id: String, pub progress_message: String }
pub struct ToolExecutionCompleteData {
    pub tool_call_id: String, pub success: bool, pub is_user_requested: Option<bool>,
    pub result: Option<ToolResultContent>, pub error: Option<ToolExecutionError>,
    pub tool_telemetry: Option<HashMap<String, serde_json::Value>>, pub parent_tool_call_id: Option<String>,
    pub mcp_server_name: Option<String>, pub mcp_tool_name: Option<String>,
}
pub struct ExternalToolRequestedData {
    pub request_id: Option<String>, pub tool_name: Option<String>,
    pub tool_call_id: Option<String>, pub arguments: Option<serde_json::Value>,
}
pub struct PermissionRequestedData { pub request_id: Option<String>, pub permission_request: Option<serde_json::Value> }
```

### Custom agents

```rust
pub struct CustomAgentStartedData { pub tool_call_id: String, pub agent_name: String, pub agent_display_name: String, pub agent_description: String }
pub struct CustomAgentCompletedData { pub tool_call_id: String, pub agent_name: String }
pub struct CustomAgentFailedData { pub tool_call_id: String, pub agent_name: String, pub error: String }
pub struct CustomAgentSelectedData { pub agent_name: String, pub agent_display_name: String, pub tools: Vec<String> }
```

### Hooks, system, skills, compaction

```rust
pub struct HookStartData { pub hook_invocation_id: String, pub hook_type: String, pub input: Option<serde_json::Value> }
pub struct HookEndData {
    pub hook_invocation_id: String, pub hook_type: String, pub output: Option<serde_json::Value>,
    pub success: bool, pub error: Option<HookError>,
}
pub struct SystemMessageEventData { pub content: String, pub role: SystemMessageRole, pub name: Option<String>, pub metadata: Option<SystemMessageMetadata> }
pub struct SkillInvokedData { pub name: String, pub path: String, pub content: String, pub allowed_tools: Option<Vec<String>> }
pub struct SessionCompactionStartData {}
pub struct CompactionTokensUsed { pub input: f64, pub output: f64, pub cached_input: f64 }
pub struct SessionCompactionCompleteData {
    pub success: bool, pub error: Option<String>, pub pre_compaction_tokens: Option<f64>,
    pub post_compaction_tokens: Option<f64>, pub pre_compaction_messages_length: Option<f64>,
    pub post_compaction_messages_length: Option<f64>, pub compaction_tokens_used: Option<CompactionTokensUsed>,
    pub messages_removed: Option<f64>, pub tokens_removed: Option<f64>, pub summary_content: Option<String>,
    pub checkpoint_number: Option<f64>, pub checkpoint_path: Option<String>,
}
```

## Related

- [Events concept](/docs/core-concepts/events)
- [Session](/docs/api/session)
