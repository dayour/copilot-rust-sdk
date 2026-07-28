// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Session event types for the Copilot SDK.
//!
//! Events are received from the Copilot CLI during a session. They include
//! assistant messages, tool executions, session lifecycle events, and more.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// =============================================================================
// Nested Types (used within event data)
// =============================================================================

/// Handoff source type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HandoffSourceType {
    Remote,
    Local,
}

/// System message role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SystemMessageRole {
    System,
    Developer,
}

/// Repository info for handoff events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryInfo {
    pub owner: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

/// Attachment in user message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMessageAttachmentItem {
    #[serde(rename = "type")]
    pub attachment_type: super::AttachmentType,
    pub path: String,
    pub display_name: String,
}

/// Tool request in assistant message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolRequestItem {
    pub tool_call_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
}

/// Tool execution result content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResultContent {
    pub content: String,
}

/// Tool execution error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExecutionError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Hook error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookError {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
}

/// System message metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemMessageMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, serde_json::Value>>,
}

// =============================================================================
// Event Data Types
// =============================================================================

/// Data for session.start event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStartData {
    #[serde(default)]
    pub session_id: String,
    #[serde(default)]
    pub version: f64,
    #[serde(default)]
    pub producer: String,
    #[serde(default)]
    pub copilot_version: String,
    #[serde(default)]
    pub start_time: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_model: Option<String>,
}

/// Data for session.resume event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResumeData {
    pub resume_time: String,
    pub event_count: f64,
}

/// Data for session.error event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionErrorData {
    pub error_type: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_call_id: Option<String>,
}

/// Data for session.idle event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionIdleData {}

/// Data for session.info event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfoData {
    pub info_type: String,
    pub message: String,
}

/// Data for session.model_change event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionModelChangeData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_model: Option<String>,
    pub new_model: String,
}

/// Data for session.handoff event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionHandoffData {
    pub handoff_time: String,
    pub source_type: HandoffSourceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<RepositoryInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_session_id: Option<String>,
}

/// Data for session.truncation event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTruncationData {
    pub token_limit: f64,
    pub pre_truncation_tokens_in_messages: f64,
    pub pre_truncation_messages_length: f64,
    pub post_truncation_tokens_in_messages: f64,
    pub post_truncation_messages_length: f64,
    pub tokens_removed_during_truncation: f64,
    pub messages_removed_during_truncation: f64,
    pub performed_by: String,
}

/// Data for user.message event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMessageData {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transformed_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<UserMessageAttachmentItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Data for pending_messages.modified event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PendingMessagesModifiedData {}

/// Data for assistant.turn_start event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantTurnStartData {
    pub turn_id: String,
}

/// Data for assistant.intent event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantIntentData {
    pub intent: String,
}

/// Data for assistant.reasoning event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantReasoningData {
    pub reasoning_id: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_content: Option<String>,
}

/// Data for assistant.reasoning_delta event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantReasoningDeltaData {
    pub reasoning_id: String,
    pub delta_content: String,
}

/// Data for assistant.message event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantMessageData {
    pub message_id: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_response_size_bytes: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_requests: Option<Vec<ToolRequestItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_call_id: Option<String>,
}

/// Data for assistant.message_delta event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantMessageDeltaData {
    pub message_id: String,
    pub delta_content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_response_size_bytes: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_call_id: Option<String>,
}

/// Data for assistant.turn_end event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantTurnEndData {
    pub turn_id: String,
}

/// Data for assistant.usage event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantUsageData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_tokens: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_write_tokens: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initiator: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_snapshots: Option<HashMap<String, serde_json::Value>>,
}

/// Data for abort event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbortData {
    pub reason: String,
}

/// Data for tool.user_requested event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolUserRequestedData {
    pub tool_call_id: String,
    pub tool_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
}

/// Data for tool.execution_start event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolExecutionStartData {
    pub tool_call_id: String,
    pub tool_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_call_id: Option<String>,
}

/// Data for tool.execution_partial_result event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolExecutionPartialResultData {
    pub tool_call_id: String,
    pub partial_output: String,
}

/// Data for tool.execution_complete event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolExecutionCompleteData {
    pub tool_call_id: String,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_user_requested: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<ToolResultContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ToolExecutionError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_telemetry: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_server_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_tool_name: Option<String>,
}

/// Data for custom_agent.started event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomAgentStartedData {
    pub tool_call_id: String,
    pub agent_name: String,
    pub agent_display_name: String,
    pub agent_description: String,
}

/// Data for custom_agent.completed event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomAgentCompletedData {
    pub tool_call_id: String,
    pub agent_name: String,
}

/// Data for custom_agent.failed event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomAgentFailedData {
    pub tool_call_id: String,
    pub agent_name: String,
    pub error: String,
}

/// Data for custom_agent.selected event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomAgentSelectedData {
    pub agent_name: String,
    pub agent_display_name: String,
    pub tools: Vec<String>,
}

/// Data for hook.start event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookStartData {
    pub hook_invocation_id: String,
    pub hook_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
}

/// Data for hook.end event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookEndData {
    pub hook_invocation_id: String,
    pub hook_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<HookError>,
}

/// Data for system.message event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemMessageEventData {
    pub content: String,
    pub role: SystemMessageRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<SystemMessageMetadata>,
}

/// Data for session.compaction_start event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionCompactionStartData {}

/// Tokens used during compaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactionTokensUsed {
    #[serde(default)]
    pub input: f64,
    #[serde(default)]
    pub output: f64,
    #[serde(default)]
    pub cached_input: f64,
}

/// Data for session.compaction_complete event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCompactionCompleteData {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_compaction_tokens: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_compaction_tokens: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_compaction_messages_length: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_compaction_messages_length: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compaction_tokens_used: Option<CompactionTokensUsed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages_removed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_removed: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_number: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_path: Option<String>,
}

/// Shutdown type for session.shutdown event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ShutdownType {
    Routine,
    Error,
}

/// Code changes reported in shutdown event.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShutdownCodeChanges {
    #[serde(default)]
    pub lines_added: f64,
    #[serde(default)]
    pub lines_removed: f64,
    #[serde(default)]
    pub files_modified: Vec<String>,
}

/// Data for session.shutdown event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionShutdownData {
    pub shutdown_type: ShutdownType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_reason: Option<String>,
    #[serde(default)]
    pub total_premium_requests: f64,
    #[serde(default)]
    pub total_api_duration_ms: f64,
    #[serde(default)]
    pub session_start_time: f64,
    #[serde(default)]
    pub code_changes: ShutdownCodeChanges,
    #[serde(default)]
    pub model_metrics: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_model: Option<String>,
}

/// Data for session.snapshot_rewind event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSnapshotRewindData {
    pub up_to_event_id: String,
    #[serde(default)]
    pub events_removed: f64,
}

/// Data for session.usage_info event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionUsageInfoData {
    #[serde(default)]
    pub token_limit: f64,
    #[serde(default)]
    pub current_tokens: f64,
    #[serde(default)]
    pub messages_length: f64,
}

/// Data for tool.execution_progress event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolExecutionProgressData {
    pub tool_call_id: String,
    pub progress_message: String,
}

/// Data for skill.invoked event.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillInvokedData {
    pub name: String,
    pub path: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<Vec<String>>,
}

// =============================================================================
// Session Event (Discriminated Union)
// =============================================================================

/// Data for `external_tool.requested` event (protocol v3 broadcast model).
///
/// In protocol v3, tool calls are broadcast as session events instead of
/// RPC requests. The SDK handles these internally and responds via
/// `session.tools.handlePendingToolCall` RPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalToolRequestedData {
    /// Unique request ID for correlating the response.
    pub request_id: Option<String>,
    /// Name of the tool being requested.
    pub tool_name: Option<String>,
    /// Tool call ID for tracking.
    pub tool_call_id: Option<String>,
    /// Arguments to pass to the tool handler.
    pub arguments: Option<serde_json::Value>,
}

/// Data for `permission.requested` event (protocol v3 broadcast model).
///
/// In protocol v3, permission requests are broadcast as session events.
/// The SDK handles these internally and responds via
/// `session.permissions.handlePendingPermissionRequest` RPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRequestedData {
    /// Unique request ID for correlating the response.
    pub request_id: Option<String>,
    /// The permission request details.
    pub permission_request: Option<serde_json::Value>,
}

/// Data for `elicitation.requested` event (protocol v3 broadcast model).
///
/// Handled internally by the SDK when an elicitation handler is registered;
/// the response is sent via `session.ui.handlePendingElicitation` RPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElicitationRequestedData {
    /// Unique request ID for correlating the response.
    pub request_id: Option<String>,
    /// Message describing what information is needed from the user.
    #[serde(default)]
    pub message: String,
    /// JSON Schema describing the form fields (form mode only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_schema: Option<serde_json::Value>,
    /// Elicitation mode ("form" or "url").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    /// The source that initiated the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elicitation_source: Option<String>,
    /// URL to open in the user's browser (url mode only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Data for `exit_plan_mode.requested` event (protocol v3 broadcast model).
///
/// Handled internally by the SDK when an exit-plan-mode handler is registered;
/// the response is sent via `session.ui.handlePendingExitPlanMode` RPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExitPlanModeRequestedData {
    /// Unique request ID for correlating the response.
    pub request_id: Option<String>,
    /// Summary of the plan that was created.
    #[serde(default)]
    pub summary: String,
    /// Full content of the plan file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_content: Option<String>,
    /// Available actions the user can take.
    #[serde(default)]
    pub actions: Vec<String>,
    /// Recommended action to preselect for the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended_action: Option<String>,
}

/// Data for `auto_mode_switch.requested` event (protocol v3 broadcast model).
///
/// Handled internally by the SDK when an auto-mode-switch handler is registered;
/// the response is sent via `session.ui.handlePendingAutoModeSwitch` RPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoModeSwitchRequestedData {
    /// Unique request ID for correlating the response.
    pub request_id: Option<String>,
    /// The rate-limit error code that triggered the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// Seconds until the rate limit resets, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_after_seconds: Option<i64>,
}

/// Data for `command.execute` event (protocol v3 broadcast model).
///
/// Routed to the owning client when a registered slash command is invoked;
/// the acknowledgement is sent via `session.commands.handlePendingCommand` RPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandExecuteData {
    /// Unique request ID for correlating the acknowledgement.
    pub request_id: Option<String>,
    /// The full command text (e.g. `/deploy production`).
    #[serde(default)]
    pub command: String,
    /// Command name without the leading `/`.
    #[serde(default)]
    pub command_name: String,
    /// Raw argument string after the command name.
    #[serde(default)]
    pub args: String,
}

// =============================================================================
// Wave 3 - additional nested types
// =============================================================================

/// Operation applied to an autopilot objective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AutopilotObjectiveOperation {
    Create,
    Update,
    Delete,
}

/// Status of an autopilot objective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutopilotObjectiveStatus {
    Active,
    Paused,
    CapReached,
    Completed,
}

/// Operation applied to the session plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PlanChangedOperation {
    Create,
    Update,
    Delete,
}

/// Operation applied to a workspace file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceFileChangedOperation {
    Create,
    Update,
}

/// Host type for the working-directory context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkingDirectoryHostType {
    GitHub,
    Ado,
}

/// Where a failed model call originated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelCallFailureSource {
    TopLevel,
    Subagent,
    McpSampling,
}

/// Action the user took on an elicitation request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ElicitationCompletedAction {
    Accept,
    Decline,
    Cancel,
}

/// Action selected when exiting plan mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitPlanModeAction {
    ExitOnly,
    Interactive,
    Autopilot,
    AutopilotFleet,
}

/// Connection status of an MCP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum McpServerStatus {
    #[serde(rename = "connected")]
    Connected,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "needs-auth")]
    NeedsAuth,
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "disabled")]
    Disabled,
    #[serde(rename = "not_configured")]
    NotConfigured,
}

/// Where an MCP server definition came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpServerSource {
    User,
    Workspace,
    Plugin,
    Builtin,
}

/// Transport used to reach an MCP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpServerTransport {
    Stdio,
    Http,
    Sse,
    Memory,
}

/// Where a skill was loaded from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SkillSource {
    Project,
    Inherited,
    PersonalCopilot,
    PersonalAgents,
    Plugin,
    Custom,
    Builtin,
}

/// Where an extension was loaded from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtensionSource {
    Project,
    User,
}

/// Runtime status of a loaded extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtensionStatus {
    Running,
    Disabled,
    Failed,
    Starting,
}

/// A permission rule that matched during a permission decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionRule {
    /// Rule kind (e.g. `shell`, `write`).
    pub kind: String,
    /// Rule argument, or `None` when the rule matches unconditionally.
    pub argument: Option<String>,
}

/// Outcome of a permission request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum PermissionResult {
    /// Approved for this single invocation.
    #[serde(rename = "approved")]
    Approved,
    /// Approved for the remainder of the session.
    #[serde(rename = "approved-for-session")]
    ApprovedForSession {
        /// The session-scoped approval descriptor.
        approval: serde_json::Value,
    },
    /// Approved for a specific location for the remainder of the session.
    #[serde(rename = "approved-for-location")]
    ApprovedForLocation {
        /// The session-scoped approval descriptor.
        approval: serde_json::Value,
        /// The location the approval is scoped to.
        #[serde(rename = "locationKey")]
        location_key: String,
    },
    /// The request was cancelled.
    #[serde(rename = "cancelled")]
    Cancelled {
        /// Optional cancellation reason.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
    /// Denied because configured rules rejected it.
    #[serde(rename = "denied-by-rules")]
    DeniedByRules {
        /// The rules that caused the denial.
        rules: Vec<PermissionRule>,
    },
    /// Denied because no rule approved it and the user could not be asked.
    #[serde(rename = "denied-no-approval-rule-and-could-not-request-from-user")]
    DeniedNoApprovalRule,
    /// Denied interactively by the user.
    #[serde(rename = "denied-interactively-by-user")]
    DeniedInteractivelyByUser {
        /// Optional feedback supplied by the user.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        feedback: Option<String>,
        /// Whether the rejection should be treated as forced.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        force_reject: Option<bool>,
    },
    /// Denied by an organization content-exclusion policy.
    #[serde(rename = "denied-by-content-exclusion-policy")]
    DeniedByContentExclusionPolicy {
        /// Human readable explanation.
        message: String,
        /// The excluded path.
        path: String,
    },
    /// Denied by a permission-request hook.
    #[serde(rename = "denied-by-permission-request-hook")]
    DeniedByPermissionRequestHook {
        /// Optional explanation from the hook.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        /// Whether the hook requested the turn be interrupted.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        interrupt: Option<bool>,
    },
}

/// A system notification payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SystemNotification {
    /// A background agent finished.
    #[serde(rename = "agent_completed", rename_all = "camelCase")]
    AgentCompleted {
        /// Identifier of the agent.
        agent_id: String,
        /// Agent type name.
        agent_type: String,
        /// Optional status string.
        status: String,
        /// Optional human readable description.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// The prompt the agent was running.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        prompt: Option<String>,
    },
    /// A background agent went idle.
    #[serde(rename = "agent_idle", rename_all = "camelCase")]
    AgentIdle {
        /// Identifier of the agent.
        agent_id: String,
        /// Agent type name.
        agent_type: String,
        /// Optional human readable description.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    /// A new inbox message arrived.
    #[serde(rename = "new_inbox_message", rename_all = "camelCase")]
    NewInboxMessage {
        /// Inbox entry identifier.
        entry_id: String,
        /// Display name of the sender.
        sender_name: String,
        /// Sender category.
        sender_type: String,
        /// Short summary of the message.
        summary: String,
    },
    /// A tracked shell command completed.
    #[serde(rename = "shell_completed", rename_all = "camelCase")]
    ShellCompleted {
        /// Identifier of the shell session.
        shell_id: String,
        /// Process exit code, when known.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        exit_code: Option<i64>,
        /// Optional human readable description.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    /// A detached shell command completed.
    #[serde(rename = "shell_detached_completed", rename_all = "camelCase")]
    ShellDetachedCompleted {
        /// Identifier of the shell session.
        shell_id: String,
        /// Optional human readable description.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
    /// An on-demand instruction file was discovered.
    #[serde(rename = "instruction_discovered", rename_all = "camelCase")]
    InstructionDiscovered {
        /// Path of the discovered instruction file.
        source_path: String,
        /// The file that triggered discovery.
        trigger_file: String,
        /// The tool that triggered discovery.
        trigger_tool: String,
        /// Optional human readable description.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        description: Option<String>,
    },
}

/// Static OAuth client configuration advertised by an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpOauthStaticClientConfig {
    /// OAuth client identifier.
    pub client_id: String,
    /// OAuth grant type (only `client_credentials` is emitted today).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grant_type: Option<String>,
    /// Whether the client is a public (non-confidential) client.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public_client: Option<bool>,
}

/// A command exposed by the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangedCommand {
    /// Command name without the leading `/`.
    pub name: String,
    /// Optional human readable description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// UI capabilities advertised by the client.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitiesChangedUi {
    /// Whether canvases are supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canvases: Option<bool>,
    /// Whether elicitation is supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elicitation: Option<bool>,
    /// Whether MCP apps are supported.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_apps: Option<bool>,
}

/// A skill loaded into the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedSkill {
    /// Skill name.
    pub name: String,
    /// Skill description.
    pub description: String,
    /// Whether the skill is currently enabled.
    pub enabled: bool,
    /// Where the skill was loaded from.
    pub source: SkillSource,
    /// Whether the user may invoke the skill directly.
    pub user_invocable: bool,
    /// On-disk path, when the skill came from a file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// A custom agent available to the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatedCustomAgent {
    /// Stable agent identifier.
    pub id: String,
    /// Agent name.
    pub name: String,
    /// Display name shown in UI.
    pub display_name: String,
    /// Agent description.
    pub description: String,
    /// Where the agent definition came from.
    pub source: String,
    /// Allowed tool names, or `None` when unrestricted.
    pub tools: Option<Vec<String>>,
    /// Whether the user may invoke the agent directly.
    pub user_invocable: bool,
    /// Model override for the agent, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// An MCP server known to the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedMcpServer {
    /// Server name.
    pub name: String,
    /// Current connection status.
    pub status: McpServerStatus,
    /// Error message when the server failed to start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Owning plugin name, when contributed by a plugin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_name: Option<String>,
    /// Owning plugin version, when contributed by a plugin.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_version: Option<String>,
    /// Where the server definition came from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<McpServerSource>,
    /// Transport used to reach the server.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transport: Option<McpServerTransport>,
}

/// An extension loaded into the session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadedExtension {
    /// Extension identifier.
    pub id: String,
    /// Extension name.
    pub name: String,
    /// Where the extension was loaded from.
    pub source: ExtensionSource,
    /// Runtime status.
    pub status: ExtensionStatus,
}

/// An action exposed by a registered canvas.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisteredCanvasAction {
    /// Action name.
    pub name: String,
    /// Optional description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON schema describing the action input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
}

/// A canvas in the session canvas registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisteredCanvas {
    /// Canvas identifier.
    pub canvas_id: String,
    /// Display name.
    pub display_name: String,
    /// Canvas description.
    pub description: String,
    /// Owning extension identifier.
    pub extension_id: String,
    /// Owning extension display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extension_name: Option<String>,
    /// JSON schema describing the canvas open input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
    /// Actions the canvas supports.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<RegisteredCanvasAction>>,
}

/// Error detail for a failed MCP app tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpAppToolCallError {
    /// Error message.
    pub message: String,
}

/// UI metadata attached to an MCP app tool.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppToolMetaUi {
    /// Resource URI backing the tool UI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_uri: Option<String>,
    /// Visibility hints.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Vec<String>>,
    /// Additional provider specific fields.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Metadata attached to an MCP app tool.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpAppToolMeta {
    /// UI metadata, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui: Option<McpAppToolMetaUi>,
    /// Additional provider specific fields.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// =============================================================================
// Wave 3 - additional event payloads
// =============================================================================

/// Data for `session.remote_steerable_changed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSteerableChangedData {
    /// Whether the remote session can currently be steered.
    pub remote_steerable: bool,
}

/// Data for `session.title_changed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleChangedData {
    /// The new session title.
    pub title: String,
}

/// Data for `session.schedule_created`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleCreatedData {
    /// Schedule identifier.
    pub id: i64,
    /// Interval between runs, in milliseconds.
    pub interval_ms: i64,
    /// The prompt the schedule will run.
    pub prompt: String,
    /// Prompt text to show in UI, when different from `prompt`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_prompt: Option<String>,
    /// Whether the schedule repeats.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recurring: Option<bool>,
}

/// Data for `session.schedule_cancelled`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleCancelledData {
    /// Schedule identifier.
    pub id: i64,
}

/// Data for `session.autopilot_objective_changed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutopilotObjectiveChangedData {
    /// The operation that was applied.
    pub operation: AutopilotObjectiveOperation,
    /// Objective identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// Objective status after the operation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<AutopilotObjectiveStatus>,
}

/// Data for `session.warning`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionWarningData {
    /// Warning message.
    pub message: String,
    /// Machine readable warning category.
    pub warning_type: String,
    /// Optional documentation URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Data for `session.mode_changed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModeChangedData {
    /// The mode the session switched to.
    pub new_mode: crate::types::SessionMode,
    /// The mode the session switched from.
    pub previous_mode: crate::types::SessionMode,
}

/// Data for `session.permissions_changed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsChangedData {
    /// Whether all permissions are currently auto-approved.
    pub allow_all_permissions: bool,
    /// Previous auto-approval state.
    pub previous_allow_all_permissions: bool,
}

/// Data for `session.plan_changed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanChangedData {
    /// The operation that was applied to the plan.
    pub operation: PlanChangedOperation,
}

/// Data for `session.workspace_file_changed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFileChangedData {
    /// The operation that was applied.
    pub operation: WorkspaceFileChangedOperation,
    /// Path of the affected file.
    pub path: String,
}

/// Data for `session.context_changed` (the working-directory context).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkingDirectoryContextData {
    /// Current working directory.
    pub cwd: String,
    /// Base commit SHA, when in a git repository.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_commit: Option<String>,
    /// Current branch name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Git repository root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub git_root: Option<String>,
    /// HEAD commit SHA.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head_commit: Option<String>,
    /// Repository hosting provider.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host_type: Option<WorkingDirectoryHostType>,
    /// Repository slug.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// Repository host name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_host: Option<String>,
}

/// Data for `session.task_complete`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompleteData {
    /// Whether the task succeeded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success: Option<bool>,
    /// Human readable summary of the task result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

/// Data for `assistant.streaming_delta`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantStreamingDeltaData {
    /// Total bytes streamed so far for this response.
    pub total_response_size_bytes: i64,
}

/// Data for `assistant.message_start`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantMessageStartData {
    /// Identifier of the message that is starting.
    pub message_id: String,
    /// Optional phase label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
}

/// Data for `model.call_failure`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCallFailureData {
    /// Where the failing call originated.
    pub source: ModelCallFailureSource,
    /// Client-side call identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_call_id: Option<String>,
    /// Call duration in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    /// Error message.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// What initiated the call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initiator: Option<String>,
    /// Model identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Provider-side call identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_call_id: Option<String>,
    /// Service request identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_request_id: Option<String>,
    /// HTTP status code, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status_code: Option<i64>,
}

/// Data for `subagent.deselected` (no fields).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomAgentDeselectedData {}

/// Data for `hook.progress`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookProgressData {
    /// Progress message emitted by the hook.
    pub message: String,
}

/// Data for `system.notification`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemNotificationData {
    /// Rendered notification text.
    pub content: String,
    /// Structured notification payload.
    pub kind: SystemNotification,
}

/// Data for `permission.completed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionCompletedData {
    /// Identifier of the original permission request.
    pub request_id: String,
    /// The permission decision.
    pub result: PermissionResult,
    /// Identifier of the tool call the permission was for.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Data for `user_input.requested`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInputRequestedData {
    /// Identifier used to correlate the response.
    pub request_id: String,
    /// The question shown to the user.
    pub question: String,
    /// Whether free-form input is permitted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_freeform: Option<bool>,
    /// Predefined choices, when offered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<String>>,
    /// Identifier of the tool call that requested input.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// Data for `user_input.completed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInputCompletedData {
    /// Identifier of the original request.
    pub request_id: String,
    /// The answer supplied by the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<String>,
    /// Whether the answer was free-form rather than a listed choice.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub was_freeform: Option<bool>,
}

/// Data for `elicitation.completed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElicitationCompletedData {
    /// Identifier of the original request.
    pub request_id: String,
    /// The action the user took.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<ElicitationCompletedAction>,
    /// Field values supplied by the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<HashMap<String, serde_json::Value>>,
}

/// Data for `sampling.requested`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplingRequestedData {
    /// Identifier used to correlate the response.
    pub request_id: String,
    /// Name of the MCP server requesting sampling.
    pub server_name: String,
    /// The MCP-side request identifier (string or number).
    pub mcp_request_id: serde_json::Value,
    /// Additional sampling parameters passed through verbatim.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Data for `sampling.completed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SamplingCompletedData {
    /// Identifier of the original request.
    pub request_id: String,
}

/// Data for `mcp.oauth_required`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpOauthRequiredData {
    /// Identifier used to correlate the response.
    pub request_id: String,
    /// Name of the MCP server requiring authorization.
    pub server_name: String,
    /// URL of the MCP server.
    pub server_url: String,
    /// Static OAuth client configuration, when advertised.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub static_client_config: Option<McpOauthStaticClientConfig>,
}

/// Data for `mcp.oauth_completed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpOauthCompletedData {
    /// Identifier of the original request.
    pub request_id: String,
}

/// Data for `session.custom_notification`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomNotificationData {
    /// Notification name.
    pub name: String,
    /// Arbitrary notification payload.
    pub payload: serde_json::Value,
    /// Source that emitted the notification.
    pub source: String,
    /// Optional subject metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<HashMap<String, String>>,
    /// Payload schema version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
}

/// Data for `external_tool.completed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalToolCompletedData {
    /// Identifier of the original request.
    pub request_id: String,
}

/// Data for `command.queued`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandQueuedData {
    /// Identifier used to correlate completion.
    pub request_id: String,
    /// The full command text.
    pub command: String,
}

/// Data for `command.completed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandCompletedData {
    /// Identifier of the original request.
    pub request_id: String,
}

/// Data for `auto_mode_switch.completed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoModeSwitchCompletedData {
    /// Identifier of the original request.
    pub request_id: String,
    /// The user response.
    pub response: crate::types::AutoModeSwitchResponse,
}

/// Data for `commands.changed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandsChangedData {
    /// The full current command list.
    pub commands: Vec<ChangedCommand>,
}

/// Data for `capabilities.changed`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilitiesChangedData {
    /// UI capabilities, when advertised.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui: Option<CapabilitiesChangedUi>,
}

/// Data for `exit_plan_mode.completed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExitPlanModeCompletedData {
    /// Identifier of the original request.
    pub request_id: String,
    /// Whether the plan was approved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approved: Option<bool>,
    /// Whether subsequent edits are auto-approved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_approve_edits: Option<bool>,
    /// Feedback supplied by the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feedback: Option<String>,
    /// The action the user selected.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_action: Option<ExitPlanModeAction>,
}

/// Data for `session.tools_updated`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsUpdatedData {
    /// The model the tool set was recomputed for.
    pub model: String,
}

/// Data for `session.background_tasks_changed` (no fields).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BackgroundTasksChangedData {}

/// Data for `session.skills_loaded`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillsLoadedData {
    /// The skills that are now loaded.
    pub skills: Vec<LoadedSkill>,
}

/// Data for `session.custom_agents_updated`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomAgentsUpdatedData {
    /// The agents that are now available.
    pub agents: Vec<UpdatedCustomAgent>,
    /// Errors encountered while loading agents.
    #[serde(default)]
    pub errors: Vec<String>,
    /// Warnings encountered while loading agents.
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Data for `session.mcp_servers_loaded`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServersLoadedData {
    /// The MCP servers that are now known.
    pub servers: Vec<LoadedMcpServer>,
}

/// Data for `session.mcp_server_status_changed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerStatusChangedData {
    /// Name of the affected server.
    pub server_name: String,
    /// The new status.
    pub status: McpServerStatus,
    /// Error message, when the server failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Data for `session.extensions_loaded`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionsLoadedData {
    /// The extensions that are now loaded.
    pub extensions: Vec<LoadedExtension>,
}

/// Data for `session.canvas.opened`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasOpenedData {
    /// Whether the canvas is ready or stale.
    pub availability: crate::canvas::CanvasInstanceAvailability,
    /// Canvas identifier.
    pub canvas_id: String,
    /// Owning extension identifier.
    pub extension_id: String,
    /// Caller-chosen instance identifier.
    pub instance_id: String,
    /// Whether an existing panel was reopened.
    pub reopen: bool,
    /// Owning extension display name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extension_name: Option<String>,
    /// Input passed to the canvas.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    /// Canvas status string.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Canvas title.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Canvas URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Data for `session.canvas.registry_changed`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasRegistryChangedData {
    /// The full current canvas registry.
    pub canvases: Vec<RegisteredCanvas>,
}

/// Data for `mcp_app.tool_call_complete`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppToolCallCompleteData {
    /// Name of the MCP server that handled the call.
    pub server_name: String,
    /// Name of the tool that was called.
    pub tool_name: String,
    /// Whether the call succeeded.
    pub success: bool,
    /// Call duration in milliseconds.
    pub duration_ms: i64,
    /// Arguments the tool was called with.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
    /// Error detail, when the call failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<McpAppToolCallError>,
    /// Result payload, when the call succeeded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Tool metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_meta: Option<McpAppToolMeta>,
}

/// Event data variants - the payload of each event type.
#[derive(Debug, Clone, Serialize)]
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
    /// External tool requested (protocol v3 broadcast).
    ExternalToolRequested(ExternalToolRequestedData),
    /// Permission requested (protocol v3 broadcast).
    PermissionRequested(PermissionRequestedData),
    /// Elicitation requested (protocol v3 broadcast).
    ElicitationRequested(ElicitationRequestedData),
    /// Exit-plan-mode requested (protocol v3 broadcast).
    ExitPlanModeRequested(ExitPlanModeRequestedData),
    /// Auto-mode-switch requested (protocol v3 broadcast).
    AutoModeSwitchRequested(AutoModeSwitchRequestedData),
    /// Registered command dispatch (protocol v3 broadcast).
    CommandExecute(CommandExecuteData),
    /// Payload for `session.remote_steerable_changed`.
    SessionRemoteSteerableChanged(RemoteSteerableChangedData),
    /// Payload for `session.title_changed`.
    SessionTitleChanged(TitleChangedData),
    /// Payload for `session.schedule_created`.
    SessionScheduleCreated(ScheduleCreatedData),
    /// Payload for `session.schedule_cancelled`.
    SessionScheduleCancelled(ScheduleCancelledData),
    /// Payload for `session.autopilot_objective_changed`.
    SessionAutopilotObjectiveChanged(AutopilotObjectiveChangedData),
    /// Payload for `session.warning`.
    SessionWarning(SessionWarningData),
    /// Payload for `session.mode_changed`.
    SessionModeChanged(ModeChangedData),
    /// Payload for `session.permissions_changed`.
    SessionPermissionsChanged(PermissionsChangedData),
    /// Payload for `session.plan_changed`.
    SessionPlanChanged(PlanChangedData),
    /// Payload for `session.workspace_file_changed`.
    SessionWorkspaceFileChanged(WorkspaceFileChangedData),
    /// Payload for `session.context_changed`.
    SessionContextChanged(WorkingDirectoryContextData),
    /// Payload for `session.task_complete`.
    SessionTaskComplete(TaskCompleteData),
    /// Payload for `assistant.streaming_delta`.
    AssistantStreamingDelta(AssistantStreamingDeltaData),
    /// Payload for `assistant.message_start`.
    AssistantMessageStart(AssistantMessageStartData),
    /// Payload for `model.call_failure`.
    ModelCallFailure(ModelCallFailureData),
    /// Payload for `subagent.deselected`.
    CustomAgentDeselected(CustomAgentDeselectedData),
    /// Payload for `hook.progress`.
    HookProgress(HookProgressData),
    /// Payload for `system.notification`.
    SystemNotification(SystemNotificationData),
    /// Payload for `permission.completed`.
    PermissionCompleted(PermissionCompletedData),
    /// Payload for `user_input.requested`.
    UserInputRequested(UserInputRequestedData),
    /// Payload for `user_input.completed`.
    UserInputCompleted(UserInputCompletedData),
    /// Payload for `elicitation.completed`.
    ElicitationCompleted(ElicitationCompletedData),
    /// Payload for `sampling.requested`.
    SamplingRequested(SamplingRequestedData),
    /// Payload for `sampling.completed`.
    SamplingCompleted(SamplingCompletedData),
    /// Payload for `mcp.oauth_required`.
    McpOauthRequired(McpOauthRequiredData),
    /// Payload for `mcp.oauth_completed`.
    McpOauthCompleted(McpOauthCompletedData),
    /// Payload for `session.custom_notification`.
    SessionCustomNotification(CustomNotificationData),
    /// Payload for `external_tool.completed`.
    ExternalToolCompleted(ExternalToolCompletedData),
    /// Payload for `command.queued`.
    CommandQueued(CommandQueuedData),
    /// Payload for `command.completed`.
    CommandCompleted(CommandCompletedData),
    /// Payload for `auto_mode_switch.completed`.
    AutoModeSwitchCompleted(AutoModeSwitchCompletedData),
    /// Payload for `commands.changed`.
    CommandsChanged(CommandsChangedData),
    /// Payload for `capabilities.changed`.
    CapabilitiesChanged(CapabilitiesChangedData),
    /// Payload for `exit_plan_mode.completed`.
    ExitPlanModeCompleted(ExitPlanModeCompletedData),
    /// Payload for `session.tools_updated`.
    SessionToolsUpdated(ToolsUpdatedData),
    /// Payload for `session.background_tasks_changed`.
    SessionBackgroundTasksChanged(BackgroundTasksChangedData),
    /// Payload for `session.skills_loaded`.
    SessionSkillsLoaded(SkillsLoadedData),
    /// Payload for `session.custom_agents_updated`.
    SessionCustomAgentsUpdated(CustomAgentsUpdatedData),
    /// Payload for `session.mcp_servers_loaded`.
    SessionMcpServersLoaded(McpServersLoadedData),
    /// Payload for `session.mcp_server_status_changed`.
    SessionMcpServerStatusChanged(McpServerStatusChangedData),
    /// Payload for `session.extensions_loaded`.
    SessionExtensionsLoaded(ExtensionsLoadedData),
    /// Payload for `session.canvas.opened`.
    SessionCanvasOpened(CanvasOpenedData),
    /// Payload for `session.canvas.registry_changed`.
    SessionCanvasRegistryChanged(CanvasRegistryChangedData),
    /// Payload for `mcp_app.tool_call_complete`.
    McpAppToolCallComplete(McpAppToolCallCompleteData),
    /// Unknown event - preserves raw JSON for forward compatibility.
    Unknown(serde_json::Value),
}

/// Raw session event as received from the CLI.
///
/// The event has common fields (id, timestamp, type) and a data payload
/// that varies based on the event type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawSessionEvent {
    pub id: String,
    pub timestamp: String,
    #[serde(rename = "type")]
    pub event_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ephemeral: Option<bool>,
    pub data: serde_json::Value,
}

/// A parsed session event with typed data.
#[derive(Debug, Clone)]
pub struct SessionEvent {
    /// Unique event ID.
    pub id: String,
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// Original type string (e.g., "assistant.message").
    pub event_type: String,
    /// Parent event ID, if any.
    pub parent_id: Option<String>,
    /// Whether this event is ephemeral.
    pub ephemeral: Option<bool>,
    /// Typed event data.
    pub data: SessionEventData,
}

impl SessionEvent {
    /// Parse a session event from JSON.
    pub fn from_json(json: &serde_json::Value) -> Result<Self, serde_json::Error> {
        let raw: RawSessionEvent = serde_json::from_value(json.clone())?;
        Ok(Self::from_raw(raw))
    }

    /// Convert a raw event to a typed event.
    pub fn from_raw(raw: RawSessionEvent) -> Self {
        let data = parse_event_data(&raw.event_type, raw.data);
        Self {
            id: raw.id,
            timestamp: raw.timestamp,
            event_type: raw.event_type,
            parent_id: raw.parent_id,
            ephemeral: raw.ephemeral,
            data,
        }
    }

    // =========================================================================
    // Type checking helpers
    // =========================================================================

    /// Check if this is an assistant message event.
    pub fn is_assistant_message(&self) -> bool {
        matches!(self.data, SessionEventData::AssistantMessage(_))
    }

    /// Check if this is an assistant message delta event.
    pub fn is_assistant_message_delta(&self) -> bool {
        matches!(self.data, SessionEventData::AssistantMessageDelta(_))
    }

    /// Check if this is a session idle event.
    pub fn is_session_idle(&self) -> bool {
        matches!(self.data, SessionEventData::SessionIdle(_))
    }

    /// Check if this is a session error event.
    pub fn is_session_error(&self) -> bool {
        matches!(self.data, SessionEventData::SessionError(_))
    }

    /// Check if this is a terminal event (session ended).
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.data,
            SessionEventData::SessionIdle(_) | SessionEventData::SessionError(_)
        )
    }

    // =========================================================================
    // Data extraction helpers
    // =========================================================================

    /// Get assistant message data if this is an assistant.message event.
    pub fn as_assistant_message(&self) -> Option<&AssistantMessageData> {
        match &self.data {
            SessionEventData::AssistantMessage(data) => Some(data),
            _ => None,
        }
    }

    /// Get assistant message delta data if this is an assistant.message_delta event.
    pub fn as_assistant_message_delta(&self) -> Option<&AssistantMessageDeltaData> {
        match &self.data {
            SessionEventData::AssistantMessageDelta(data) => Some(data),
            _ => None,
        }
    }

    /// Get session error data if this is a session.error event.
    pub fn as_session_error(&self) -> Option<&SessionErrorData> {
        match &self.data {
            SessionEventData::SessionError(data) => Some(data),
            _ => None,
        }
    }

    /// Get tool execution complete data if this is a tool.execution_complete event.
    pub fn as_tool_execution_complete(&self) -> Option<&ToolExecutionCompleteData> {
        match &self.data {
            SessionEventData::ToolExecutionComplete(data) => Some(data),
            _ => None,
        }
    }

    /// Extract the content from an assistant message or delta.
    pub fn content(&self) -> Option<&str> {
        match &self.data {
            SessionEventData::AssistantMessage(data) => Some(&data.content),
            SessionEventData::AssistantMessageDelta(data) => Some(&data.delta_content),
            _ => None,
        }
    }
}

/// Parse event data based on event type string.
fn parse_event_data(event_type: &str, data: serde_json::Value) -> SessionEventData {
    match event_type {
        "session.start" => serde_json::from_value(data)
            .map(SessionEventData::SessionStart)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.resume" => serde_json::from_value(data)
            .map(SessionEventData::SessionResume)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.error" => serde_json::from_value(data)
            .map(SessionEventData::SessionError)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.idle" => SessionEventData::SessionIdle(SessionIdleData {}),
        "session.info" => serde_json::from_value(data)
            .map(SessionEventData::SessionInfo)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.model_change" => serde_json::from_value(data)
            .map(SessionEventData::SessionModelChange)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.handoff" => serde_json::from_value(data)
            .map(SessionEventData::SessionHandoff)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.truncation" => serde_json::from_value(data)
            .map(SessionEventData::SessionTruncation)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "user.message" => serde_json::from_value(data)
            .map(SessionEventData::UserMessage)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "pending_messages.modified" => {
            SessionEventData::PendingMessagesModified(PendingMessagesModifiedData {})
        }
        "assistant.turn_start" => serde_json::from_value(data)
            .map(SessionEventData::AssistantTurnStart)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "assistant.intent" => serde_json::from_value(data)
            .map(SessionEventData::AssistantIntent)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "assistant.reasoning" => serde_json::from_value(data)
            .map(SessionEventData::AssistantReasoning)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "assistant.reasoning_delta" => serde_json::from_value(data)
            .map(SessionEventData::AssistantReasoningDelta)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "assistant.message" => serde_json::from_value(data)
            .map(SessionEventData::AssistantMessage)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "assistant.message_delta" => serde_json::from_value(data)
            .map(SessionEventData::AssistantMessageDelta)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "assistant.turn_end" => serde_json::from_value(data)
            .map(SessionEventData::AssistantTurnEnd)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "assistant.usage" => serde_json::from_value(data)
            .map(SessionEventData::AssistantUsage)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "abort" => serde_json::from_value(data)
            .map(SessionEventData::Abort)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "tool.user_requested" => serde_json::from_value(data)
            .map(SessionEventData::ToolUserRequested)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "tool.execution_start" => serde_json::from_value(data)
            .map(SessionEventData::ToolExecutionStart)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "tool.execution_partial_result" => serde_json::from_value(data)
            .map(SessionEventData::ToolExecutionPartialResult)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "tool.execution_complete" => serde_json::from_value(data)
            .map(SessionEventData::ToolExecutionComplete)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "tool.execution_progress" => serde_json::from_value(data)
            .map(SessionEventData::ToolExecutionProgress)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        // Primary wire names (subagent.*) + legacy aliases (custom_agent.*)
        "subagent.started" | "custom_agent.started" => serde_json::from_value(data)
            .map(SessionEventData::CustomAgentStarted)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "subagent.completed" | "custom_agent.completed" => serde_json::from_value(data)
            .map(SessionEventData::CustomAgentCompleted)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "subagent.failed" | "custom_agent.failed" => serde_json::from_value(data)
            .map(SessionEventData::CustomAgentFailed)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "subagent.selected" | "custom_agent.selected" => serde_json::from_value(data)
            .map(SessionEventData::CustomAgentSelected)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "hook.start" => serde_json::from_value(data)
            .map(SessionEventData::HookStart)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "hook.end" => serde_json::from_value(data)
            .map(SessionEventData::HookEnd)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "system.message" => serde_json::from_value(data)
            .map(SessionEventData::SystemMessage)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.compaction_start" => {
            SessionEventData::SessionCompactionStart(SessionCompactionStartData {})
        }
        "session.compaction_complete" => serde_json::from_value(data)
            .map(SessionEventData::SessionCompactionComplete)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.shutdown" => serde_json::from_value(data)
            .map(SessionEventData::SessionShutdown)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.snapshot_rewind" => serde_json::from_value(data)
            .map(SessionEventData::SessionSnapshotRewind)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.usage_info" => serde_json::from_value(data)
            .map(SessionEventData::SessionUsageInfo)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "skill.invoked" => serde_json::from_value(data)
            .map(SessionEventData::SkillInvoked)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "external_tool.requested" => serde_json::from_value(data)
            .map(SessionEventData::ExternalToolRequested)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "permission.requested" => serde_json::from_value(data)
            .map(SessionEventData::PermissionRequested)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "elicitation.requested" => serde_json::from_value(data)
            .map(SessionEventData::ElicitationRequested)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "exit_plan_mode.requested" => serde_json::from_value(data)
            .map(SessionEventData::ExitPlanModeRequested)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "auto_mode_switch.requested" => serde_json::from_value(data)
            .map(SessionEventData::AutoModeSwitchRequested)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "command.execute" => serde_json::from_value(data)
            .map(SessionEventData::CommandExecute)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.remote_steerable_changed" => serde_json::from_value(data)
            .map(SessionEventData::SessionRemoteSteerableChanged)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.title_changed" => serde_json::from_value(data)
            .map(SessionEventData::SessionTitleChanged)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.schedule_created" => serde_json::from_value(data)
            .map(SessionEventData::SessionScheduleCreated)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.schedule_cancelled" => serde_json::from_value(data)
            .map(SessionEventData::SessionScheduleCancelled)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.autopilot_objective_changed" => serde_json::from_value(data)
            .map(SessionEventData::SessionAutopilotObjectiveChanged)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.warning" => serde_json::from_value(data)
            .map(SessionEventData::SessionWarning)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.mode_changed" => serde_json::from_value(data)
            .map(SessionEventData::SessionModeChanged)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.permissions_changed" => serde_json::from_value(data)
            .map(SessionEventData::SessionPermissionsChanged)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.plan_changed" => serde_json::from_value(data)
            .map(SessionEventData::SessionPlanChanged)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.workspace_file_changed" => serde_json::from_value(data)
            .map(SessionEventData::SessionWorkspaceFileChanged)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.context_changed" => serde_json::from_value(data)
            .map(SessionEventData::SessionContextChanged)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.task_complete" => serde_json::from_value(data)
            .map(SessionEventData::SessionTaskComplete)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "assistant.streaming_delta" => serde_json::from_value(data)
            .map(SessionEventData::AssistantStreamingDelta)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "assistant.message_start" => serde_json::from_value(data)
            .map(SessionEventData::AssistantMessageStart)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "model.call_failure" => serde_json::from_value(data)
            .map(SessionEventData::ModelCallFailure)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "subagent.deselected" | "custom_agent.deselected" => {
            SessionEventData::CustomAgentDeselected(CustomAgentDeselectedData {})
        }
        "hook.progress" => serde_json::from_value(data)
            .map(SessionEventData::HookProgress)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "system.notification" => serde_json::from_value(data)
            .map(SessionEventData::SystemNotification)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "permission.completed" => serde_json::from_value(data)
            .map(SessionEventData::PermissionCompleted)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "user_input.requested" => serde_json::from_value(data)
            .map(SessionEventData::UserInputRequested)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "user_input.completed" => serde_json::from_value(data)
            .map(SessionEventData::UserInputCompleted)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "elicitation.completed" => serde_json::from_value(data)
            .map(SessionEventData::ElicitationCompleted)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "sampling.requested" => serde_json::from_value(data)
            .map(SessionEventData::SamplingRequested)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "sampling.completed" => serde_json::from_value(data)
            .map(SessionEventData::SamplingCompleted)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "mcp.oauth_required" => serde_json::from_value(data)
            .map(SessionEventData::McpOauthRequired)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "mcp.oauth_completed" => serde_json::from_value(data)
            .map(SessionEventData::McpOauthCompleted)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.custom_notification" => serde_json::from_value(data)
            .map(SessionEventData::SessionCustomNotification)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "external_tool.completed" => serde_json::from_value(data)
            .map(SessionEventData::ExternalToolCompleted)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "command.queued" => serde_json::from_value(data)
            .map(SessionEventData::CommandQueued)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "command.completed" => serde_json::from_value(data)
            .map(SessionEventData::CommandCompleted)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "auto_mode_switch.completed" => serde_json::from_value(data)
            .map(SessionEventData::AutoModeSwitchCompleted)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "commands.changed" => serde_json::from_value(data)
            .map(SessionEventData::CommandsChanged)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "capabilities.changed" => serde_json::from_value(data)
            .map(SessionEventData::CapabilitiesChanged)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "exit_plan_mode.completed" => serde_json::from_value(data)
            .map(SessionEventData::ExitPlanModeCompleted)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.tools_updated" => serde_json::from_value(data)
            .map(SessionEventData::SessionToolsUpdated)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.background_tasks_changed" => {
            SessionEventData::SessionBackgroundTasksChanged(BackgroundTasksChangedData {})
        }
        "session.skills_loaded" => serde_json::from_value(data)
            .map(SessionEventData::SessionSkillsLoaded)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.custom_agents_updated" => serde_json::from_value(data)
            .map(SessionEventData::SessionCustomAgentsUpdated)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.mcp_servers_loaded" => serde_json::from_value(data)
            .map(SessionEventData::SessionMcpServersLoaded)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.mcp_server_status_changed" => serde_json::from_value(data)
            .map(SessionEventData::SessionMcpServerStatusChanged)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.extensions_loaded" => serde_json::from_value(data)
            .map(SessionEventData::SessionExtensionsLoaded)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.canvas.opened" => serde_json::from_value(data)
            .map(SessionEventData::SessionCanvasOpened)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "session.canvas.registry_changed" => serde_json::from_value(data)
            .map(SessionEventData::SessionCanvasRegistryChanged)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        "mcp_app.tool_call_complete" => serde_json::from_value(data)
            .map(SessionEventData::McpAppToolCallComplete)
            .unwrap_or_else(|_| SessionEventData::Unknown(serde_json::Value::Null)),
        // Unknown event type - preserve raw data
        _ => SessionEventData::Unknown(data),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_assistant_message() {
        let json = json!({
            "id": "evt_123",
            "timestamp": "2024-01-15T10:30:00Z",
            "type": "assistant.message",
            "data": {
                "messageId": "msg_456",
                "content": "Hello, world!"
            }
        });

        let event = SessionEvent::from_json(&json).unwrap();
        assert_eq!(event.id, "evt_123");
        assert_eq!(event.event_type, "assistant.message");
        assert!(event.is_assistant_message());

        let msg = event.as_assistant_message().unwrap();
        assert_eq!(msg.message_id, "msg_456");
        assert_eq!(msg.content, "Hello, world!");
    }

    #[test]
    fn test_parse_assistant_message_delta() {
        let json = json!({
            "id": "evt_124",
            "timestamp": "2024-01-15T10:30:01Z",
            "type": "assistant.message_delta",
            "data": {
                "messageId": "msg_456",
                "deltaContent": "Hello"
            }
        });

        let event = SessionEvent::from_json(&json).unwrap();
        assert!(event.is_assistant_message_delta());
        assert_eq!(event.content(), Some("Hello"));
    }

    #[test]
    fn test_parse_session_idle() {
        let json = json!({
            "id": "evt_125",
            "timestamp": "2024-01-15T10:30:02Z",
            "type": "session.idle",
            "data": {}
        });

        let event = SessionEvent::from_json(&json).unwrap();
        assert!(event.is_session_idle());
        assert!(event.is_terminal());
    }

    #[test]
    fn test_parse_external_tool_requested() {
        let json = json!({
            "id": "evt_125b",
            "timestamp": "2024-01-15T10:30:02Z",
            "type": "external_tool.requested",
            "data": {
                "requestId": "req_123",
                "toolName": "echo",
                "toolCallId": "call_456",
                "arguments": {
                    "text": "hello"
                }
            }
        });

        let event = SessionEvent::from_json(&json).unwrap();
        match &event.data {
            SessionEventData::ExternalToolRequested(data) => {
                assert_eq!(data.request_id.as_deref(), Some("req_123"));
                assert_eq!(data.tool_name.as_deref(), Some("echo"));
                assert_eq!(data.tool_call_id.as_deref(), Some("call_456"));
                assert_eq!(data.arguments.as_ref().unwrap()["text"], "hello");
            }
            other => panic!("Expected ExternalToolRequested, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_permission_requested() {
        let json = json!({
            "id": "evt_125c",
            "timestamp": "2024-01-15T10:30:02Z",
            "type": "permission.requested",
            "data": {
                "requestId": "req_789",
                "permissionRequest": {
                    "kind": "tool_execution",
                    "toolCallId": "call_456",
                    "toolName": "shell"
                }
            }
        });

        let event = SessionEvent::from_json(&json).unwrap();
        match &event.data {
            SessionEventData::PermissionRequested(data) => {
                assert_eq!(data.request_id.as_deref(), Some("req_789"));
                assert_eq!(
                    data.permission_request.as_ref().unwrap()["kind"],
                    "tool_execution"
                );
                assert_eq!(
                    data.permission_request.as_ref().unwrap()["toolName"],
                    "shell"
                );
            }
            other => panic!("Expected PermissionRequested, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_elicitation_requested() {
        let json = json!({
            "id": "evt_e1",
            "timestamp": "2024-01-15T10:30:02Z",
            "type": "elicitation.requested",
            "data": {
                "requestId": "req_e1",
                "message": "Please confirm",
                "mode": "form",
                "requestedSchema": { "type": "object" }
            }
        });
        let event = SessionEvent::from_json(&json).unwrap();
        match &event.data {
            SessionEventData::ElicitationRequested(data) => {
                assert_eq!(data.request_id.as_deref(), Some("req_e1"));
                assert_eq!(data.message, "Please confirm");
                assert_eq!(data.mode.as_deref(), Some("form"));
                assert!(data.requested_schema.is_some());
            }
            other => panic!("Expected ElicitationRequested, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_exit_plan_mode_requested() {
        let json = json!({
            "id": "evt_p1",
            "timestamp": "2024-01-15T10:30:02Z",
            "type": "exit_plan_mode.requested",
            "data": {
                "requestId": "req_p1",
                "summary": "Ship it",
                "planContent": "# Plan",
                "actions": ["approve", "reject"],
                "recommendedAction": "approve"
            }
        });
        let event = SessionEvent::from_json(&json).unwrap();
        match &event.data {
            SessionEventData::ExitPlanModeRequested(data) => {
                assert_eq!(data.request_id.as_deref(), Some("req_p1"));
                assert_eq!(data.summary, "Ship it");
                assert_eq!(data.actions, vec!["approve", "reject"]);
                assert_eq!(data.recommended_action.as_deref(), Some("approve"));
            }
            other => panic!("Expected ExitPlanModeRequested, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_auto_mode_switch_requested() {
        let json = json!({
            "id": "evt_a1",
            "timestamp": "2024-01-15T10:30:02Z",
            "type": "auto_mode_switch.requested",
            "data": {
                "requestId": "req_a1",
                "errorCode": "rate_limited",
                "retryAfterSeconds": 30
            }
        });
        let event = SessionEvent::from_json(&json).unwrap();
        match &event.data {
            SessionEventData::AutoModeSwitchRequested(data) => {
                assert_eq!(data.request_id.as_deref(), Some("req_a1"));
                assert_eq!(data.error_code.as_deref(), Some("rate_limited"));
                assert_eq!(data.retry_after_seconds, Some(30));
            }
            other => panic!("Expected AutoModeSwitchRequested, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_command_execute() {
        let json = json!({
            "id": "evt_c1",
            "timestamp": "2024-01-15T10:30:02Z",
            "type": "command.execute",
            "data": {
                "requestId": "req_c1",
                "command": "/deploy production",
                "commandName": "deploy",
                "args": "production"
            }
        });
        let event = SessionEvent::from_json(&json).unwrap();
        match &event.data {
            SessionEventData::CommandExecute(data) => {
                assert_eq!(data.request_id.as_deref(), Some("req_c1"));
                assert_eq!(data.command, "/deploy production");
                assert_eq!(data.command_name, "deploy");
                assert_eq!(data.args, "production");
            }
            other => panic!("Expected CommandExecute, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_session_error() {
        let json = json!({
            "id": "evt_126",
            "timestamp": "2024-01-15T10:30:03Z",
            "type": "session.error",
            "data": {
                "errorType": "api_error",
                "message": "Rate limit exceeded"
            }
        });

        let event = SessionEvent::from_json(&json).unwrap();
        assert!(event.is_session_error());
        assert!(event.is_terminal());

        let err = event.as_session_error().unwrap();
        assert_eq!(err.error_type, "api_error");
        assert_eq!(err.message, "Rate limit exceeded");
    }

    #[test]
    fn test_parse_tool_execution_complete() {
        let json = json!({
            "id": "evt_127",
            "timestamp": "2024-01-15T10:30:04Z",
            "type": "tool.execution_complete",
            "data": {
                "toolCallId": "call_789",
                "success": true,
                "result": {
                    "content": "Tool output"
                }
            }
        });

        let event = SessionEvent::from_json(&json).unwrap();
        let tool = event.as_tool_execution_complete().unwrap();
        assert_eq!(tool.tool_call_id, "call_789");
        assert!(tool.success);
        assert_eq!(tool.result.as_ref().unwrap().content, "Tool output");
    }

    #[test]
    fn test_parse_unknown_event() {
        let json = json!({
            "id": "evt_128",
            "timestamp": "2024-01-15T10:30:05Z",
            "type": "future.unknown_event",
            "data": {
                "someField": "someValue"
            }
        });

        let event = SessionEvent::from_json(&json).unwrap();
        assert_eq!(event.event_type, "future.unknown_event");
        assert!(matches!(event.data, SessionEventData::Unknown(_)));
    }

    #[test]
    fn test_parse_session_start() {
        let json = json!({
            "id": "evt_001",
            "timestamp": "2024-01-15T10:30:00Z",
            "type": "session.start",
            "data": {
                "sessionId": "sess_123",
                "version": 1.0,
                "producer": "copilot-cli",
                "copilotVersion": "1.0.0",
                "startTime": "2024-01-15T10:30:00Z"
            }
        });

        let event = SessionEvent::from_json(&json).unwrap();
        if let SessionEventData::SessionStart(data) = &event.data {
            assert_eq!(data.session_id, "sess_123");
            assert_eq!(data.producer, "copilot-cli");
        } else {
            panic!("Expected SessionStart");
        }
    }

    #[test]
    fn test_event_with_parent_id() {
        let json = json!({
            "id": "evt_129",
            "timestamp": "2024-01-15T10:30:06Z",
            "type": "assistant.message",
            "parentId": "evt_128",
            "ephemeral": true,
            "data": {
                "messageId": "msg_789",
                "content": "Nested message"
            }
        });

        let event = SessionEvent::from_json(&json).unwrap();
        assert_eq!(event.parent_id, Some("evt_128".to_string()));
        assert_eq!(event.ephemeral, Some(true));
    }

    #[test]
    fn test_parse_subagent_started() {
        let json = json!({
            "id": "evt_200",
            "timestamp": "2024-01-15T10:30:00Z",
            "type": "subagent.started",
            "data": {
                "toolCallId": "call_1",
                "agentName": "test-agent",
                "agentDisplayName": "Test Agent",
                "agentDescription": "A test agent"
            }
        });
        let event = SessionEvent::from_json(&json).unwrap();
        assert!(matches!(
            event.data,
            SessionEventData::CustomAgentStarted(_)
        ));
        if let SessionEventData::CustomAgentStarted(data) = &event.data {
            assert_eq!(data.agent_name, "test-agent");
        }
    }

    #[test]
    fn test_parse_subagent_completed_legacy_alias() {
        // Verify legacy custom_agent.* wire names still work
        let json = json!({
            "id": "evt_201",
            "timestamp": "2024-01-15T10:30:00Z",
            "type": "custom_agent.completed",
            "data": {
                "toolCallId": "call_1",
                "agentName": "test-agent"
            }
        });
        let event = SessionEvent::from_json(&json).unwrap();
        assert!(matches!(
            event.data,
            SessionEventData::CustomAgentCompleted(_)
        ));
    }

    #[test]
    fn test_parse_subagent_all_wire_names() {
        for wire_name in &["subagent.failed", "custom_agent.failed"] {
            let json = json!({
                "id": "evt_202",
                "timestamp": "2024-01-15T10:30:00Z",
                "type": wire_name,
                "data": {
                    "toolCallId": "call_1",
                    "agentName": "agent",
                    "error": "boom"
                }
            });
            let event = SessionEvent::from_json(&json).unwrap();
            assert!(
                matches!(event.data, SessionEventData::CustomAgentFailed(_)),
                "Failed to parse {wire_name}"
            );
        }
    }

    #[test]
    fn test_parse_session_compaction_start() {
        let json = json!({
            "id": "evt_300",
            "timestamp": "2024-01-15T10:30:00Z",
            "type": "session.compaction_start",
            "data": {}
        });
        let event = SessionEvent::from_json(&json).unwrap();
        assert!(matches!(
            event.data,
            SessionEventData::SessionCompactionStart(_)
        ));
    }

    #[test]
    fn test_parse_session_compaction_complete() {
        let json = json!({
            "id": "evt_301",
            "timestamp": "2024-01-15T10:30:00Z",
            "type": "session.compaction_complete",
            "data": {
                "success": true,
                "preCompactionTokens": 50000.0,
                "postCompactionTokens": 10000.0,
                "compactionTokensUsed": {
                    "input": 100.0,
                    "output": 200.0,
                    "cachedInput": 50.0
                },
                "summaryContent": "Session was compacted"
            }
        });
        let event = SessionEvent::from_json(&json).unwrap();
        if let SessionEventData::SessionCompactionComplete(data) = &event.data {
            assert!(data.success);
            assert_eq!(data.pre_compaction_tokens, Some(50000.0));
            assert_eq!(data.compaction_tokens_used.as_ref().unwrap().input, 100.0);
        } else {
            panic!("Expected SessionCompactionComplete");
        }
    }

    #[test]
    fn test_parse_session_shutdown() {
        let json = json!({
            "id": "evt_302",
            "timestamp": "2024-01-15T10:30:00Z",
            "type": "session.shutdown",
            "data": {
                "shutdownType": "routine",
                "totalPremiumRequests": 5.0,
                "totalApiDurationMs": 1200.0,
                "sessionStartTime": 1700000000.0,
                "codeChanges": {
                    "linesAdded": 10.0,
                    "linesRemoved": 3.0,
                    "filesModified": ["src/main.rs"]
                },
                "modelMetrics": {},
                "currentModel": "gpt-4"
            }
        });
        let event = SessionEvent::from_json(&json).unwrap();
        if let SessionEventData::SessionShutdown(data) = &event.data {
            assert_eq!(data.shutdown_type, ShutdownType::Routine);
            assert_eq!(data.current_model, Some("gpt-4".to_string()));
            assert_eq!(data.code_changes.lines_added, 10.0);
        } else {
            panic!("Expected SessionShutdown");
        }
    }

    #[test]
    fn test_parse_session_snapshot_rewind() {
        let json = json!({
            "id": "evt_303",
            "timestamp": "2024-01-15T10:30:00Z",
            "type": "session.snapshot_rewind",
            "data": {
                "upToEventId": "evt_100",
                "eventsRemoved": 5.0
            }
        });
        let event = SessionEvent::from_json(&json).unwrap();
        if let SessionEventData::SessionSnapshotRewind(data) = &event.data {
            assert_eq!(data.up_to_event_id, "evt_100");
            assert_eq!(data.events_removed, 5.0);
        } else {
            panic!("Expected SessionSnapshotRewind");
        }
    }

    #[test]
    fn test_parse_session_usage_info() {
        let json = json!({
            "id": "evt_304",
            "timestamp": "2024-01-15T10:30:00Z",
            "type": "session.usage_info",
            "data": {
                "tokenLimit": 100000.0,
                "currentTokens": 50000.0,
                "messagesLength": 42.0
            }
        });
        let event = SessionEvent::from_json(&json).unwrap();
        if let SessionEventData::SessionUsageInfo(data) = &event.data {
            assert_eq!(data.token_limit, 100000.0);
            assert_eq!(data.current_tokens, 50000.0);
        } else {
            panic!("Expected SessionUsageInfo");
        }
    }

    #[test]
    fn test_parse_tool_execution_progress() {
        let json = json!({
            "id": "evt_305",
            "timestamp": "2024-01-15T10:30:00Z",
            "type": "tool.execution_progress",
            "data": {
                "toolCallId": "call_100",
                "progressMessage": "Processing file 3 of 10..."
            }
        });
        let event = SessionEvent::from_json(&json).unwrap();
        if let SessionEventData::ToolExecutionProgress(data) = &event.data {
            assert_eq!(data.tool_call_id, "call_100");
            assert_eq!(data.progress_message, "Processing file 3 of 10...");
        } else {
            panic!("Expected ToolExecutionProgress");
        }
    }

    #[test]
    fn test_parse_skill_invoked() {
        let json = json!({
            "id": "evt_306",
            "timestamp": "2024-01-15T10:30:00Z",
            "type": "skill.invoked",
            "data": {
                "name": "code-review",
                "path": "/skills/code-review",
                "content": "Review this code",
                "allowedTools": ["read_file", "search"]
            }
        });
        let event = SessionEvent::from_json(&json).unwrap();
        if let SessionEventData::SkillInvoked(data) = &event.data {
            assert_eq!(data.name, "code-review");
            assert_eq!(data.allowed_tools.as_ref().unwrap().len(), 2);
        } else {
            panic!("Expected SkillInvoked");
        }
    }

    #[test]
    fn test_session_error_with_code_and_provider_call_id() {
        let json = json!({
            "id": "evt_err",
            "timestamp": "2024-01-15T10:30:00Z",
            "type": "session.error",
            "data": {
                "errorType": "provider_error",
                "message": "Rate limited",
                "code": 429.0,
                "providerCallId": "call-abc-123"
            }
        });
        let event = SessionEvent::from_json(&json).unwrap();
        if let SessionEventData::SessionError(data) = &event.data {
            assert_eq!(data.error_type, "provider_error");
            assert_eq!(data.code, Some(429.0));
            assert_eq!(data.provider_call_id.as_deref(), Some("call-abc-123"));
        } else {
            panic!("Expected SessionError");
        }
    }

    #[test]
    fn test_tool_execution_complete_with_mcp_fields() {
        let json = json!({
            "id": "evt_mcp",
            "timestamp": "2024-01-15T10:30:00Z",
            "type": "tool.execution_complete",
            "data": {
                "toolCallId": "call-1",
                "success": true,
                "mcpServerName": "my-server",
                "mcpToolName": "read_file"
            }
        });
        let event = SessionEvent::from_json(&json).unwrap();
        if let SessionEventData::ToolExecutionComplete(data) = &event.data {
            assert_eq!(data.mcp_server_name.as_deref(), Some("my-server"));
            assert_eq!(data.mcp_tool_name.as_deref(), Some("read_file"));
        } else {
            panic!("Expected ToolExecutionComplete");
        }
    }

    #[test]
    fn test_session_start_data_optional_fields() {
        // All fields missing should still parse with defaults
        let json = json!({
            "id": "evt_start",
            "timestamp": "2024-01-15T10:30:00Z",
            "type": "session.start",
            "data": {}
        });
        let event = SessionEvent::from_json(&json).unwrap();
        if let SessionEventData::SessionStart(data) = &event.data {
            assert_eq!(data.session_id, "");
            assert_eq!(data.version, 0.0);
            assert_eq!(data.producer, "");
        } else {
            panic!("Expected SessionStart");
        }
    }

    #[test]
    fn test_unknown_event_type_handled_gracefully() {
        let json = json!({
            "id": "evt_unknown",
            "timestamp": "2025-01-01T00:00:00Z",
            "type": "some.future.event.type",
            "data": {"someField": "someValue"}
        });
        // Parsing an unknown event type should not panic
        let raw: RawSessionEvent = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(raw.event_type, "some.future.event.type");

        // It should also parse into a SessionEvent with Unknown data
        let event = SessionEvent::from_json(&json).unwrap();
        assert_eq!(event.event_type, "some.future.event.type");
        assert!(matches!(event.data, SessionEventData::Unknown(_)));
    }

    #[test]
    fn test_session_shutdown_event_parsed() {
        let json = json!({
            "id": "evt_shutdown",
            "timestamp": "2025-01-01T00:00:00Z",
            "type": "session.shutdown",
            "data": {
                "shutdownType": "routine",
                "reason": "user requested"
            }
        });
        let raw: RawSessionEvent = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(raw.event_type, "session.shutdown");

        let event = SessionEvent::from_json(&json).unwrap();
        assert_eq!(event.event_type, "session.shutdown");
    }

    #[test]
    fn test_session_usage_info_recognized() {
        let json = json!({
            "id": "evt_usage",
            "timestamp": "2025-01-01T00:00:00Z",
            "type": "session.usage_info",
            "data": {}
        });
        let raw: RawSessionEvent = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(raw.event_type, "session.usage_info");

        let event = SessionEvent::from_json(&json).unwrap();
        assert_eq!(event.event_type, "session.usage_info");
    }

    // =========================================================================
    // Wave 3 - event coverage tests
    // =========================================================================

    fn parse(event_type: &str, data: serde_json::Value) -> SessionEventData {
        parse_event_data(event_type, data)
    }

    #[test]
    fn test_parse_session_lifecycle_wave3_events() {
        assert!(matches!(
            parse("session.title_changed", json!({ "title": "New title" })),
            SessionEventData::SessionTitleChanged(ref d) if d.title == "New title"
        ));
        assert!(matches!(
            parse(
                "session.remote_steerable_changed",
                json!({ "remoteSteerable": true })
            ),
            SessionEventData::SessionRemoteSteerableChanged(ref d) if d.remote_steerable
        ));
        assert!(matches!(
            parse(
                "session.warning",
                json!({ "message": "m", "warningType": "quota" })
            ),
            SessionEventData::SessionWarning(ref d) if d.warning_type == "quota"
        ));
        assert!(matches!(
            parse(
                "session.mode_changed",
                json!({ "newMode": "plan", "previousMode": "interactive" })
            ),
            SessionEventData::SessionModeChanged(_)
        ));
        assert!(matches!(
            parse(
                "session.permissions_changed",
                json!({ "allowAllPermissions": true, "previousAllowAllPermissions": false })
            ),
            SessionEventData::SessionPermissionsChanged(ref d) if d.allow_all_permissions
        ));
        assert!(matches!(
            parse("session.plan_changed", json!({ "operation": "update" })),
            SessionEventData::SessionPlanChanged(ref d)
                if matches!(d.operation, PlanChangedOperation::Update)
        ));
        assert!(matches!(
            parse(
                "session.workspace_file_changed",
                json!({ "operation": "create", "path": "a.txt" })
            ),
            SessionEventData::SessionWorkspaceFileChanged(ref d) if d.path == "a.txt"
        ));
        assert!(matches!(
            parse("session.task_complete", json!({ "success": true })),
            SessionEventData::SessionTaskComplete(ref d) if d.success == Some(true)
        ));
        assert!(matches!(
            parse("session.tools_updated", json!({ "model": "gpt-5" })),
            SessionEventData::SessionToolsUpdated(ref d) if d.model == "gpt-5"
        ));
        assert!(matches!(
            parse("session.background_tasks_changed", json!({})),
            SessionEventData::SessionBackgroundTasksChanged(_)
        ));
    }

    #[test]
    fn test_parse_context_changed_event() {
        let ev = parse(
            "session.context_changed",
            json!({
                "cwd": "/repo",
                "branch": "main",
                "gitRoot": "/repo",
                "headCommit": "abc",
                "hostType": "github",
                "repository": "o/r",
                "repositoryHost": "github.com"
            }),
        );
        match ev {
            SessionEventData::SessionContextChanged(d) => {
                assert_eq!(d.cwd, "/repo");
                assert_eq!(d.branch.as_deref(), Some("main"));
                assert!(matches!(
                    d.host_type,
                    Some(WorkingDirectoryHostType::GitHub)
                ));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_parse_schedule_and_autopilot_events() {
        match parse(
            "session.schedule_created",
            json!({ "id": 7, "intervalMs": 60000, "prompt": "check", "recurring": true }),
        ) {
            SessionEventData::SessionScheduleCreated(d) => {
                assert_eq!(d.id, 7);
                assert_eq!(d.interval_ms, 60000);
                assert_eq!(d.recurring, Some(true));
            }
            other => panic!("unexpected: {other:?}"),
        }
        assert!(matches!(
            parse("session.schedule_cancelled", json!({ "id": 7 })),
            SessionEventData::SessionScheduleCancelled(ref d) if d.id == 7
        ));
        match parse(
            "session.autopilot_objective_changed",
            json!({ "operation": "update", "id": 3, "status": "cap_reached" }),
        ) {
            SessionEventData::SessionAutopilotObjectiveChanged(d) => {
                assert!(matches!(d.operation, AutopilotObjectiveOperation::Update));
                assert!(matches!(
                    d.status,
                    Some(AutopilotObjectiveStatus::CapReached)
                ));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_parse_assistant_and_model_wave3_events() {
        assert!(matches!(
            parse(
                "assistant.streaming_delta",
                json!({ "totalResponseSizeBytes": 1024 })
            ),
            SessionEventData::AssistantStreamingDelta(ref d)
                if d.total_response_size_bytes == 1024
        ));
        assert!(matches!(
            parse("assistant.message_start", json!({ "messageId": "m1" })),
            SessionEventData::AssistantMessageStart(ref d) if d.message_id == "m1"
        ));
        match parse(
            "model.call_failure",
            json!({ "source": "top_level", "statusCode": 429, "model": "gpt-5" }),
        ) {
            SessionEventData::ModelCallFailure(d) => {
                assert!(matches!(d.source, ModelCallFailureSource::TopLevel));
                assert_eq!(d.status_code, Some(429));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_parse_subagent_deselected_and_hook_progress() {
        assert!(matches!(
            parse("subagent.deselected", json!({})),
            SessionEventData::CustomAgentDeselected(_)
        ));
        assert!(matches!(
            parse("custom_agent.deselected", json!({})),
            SessionEventData::CustomAgentDeselected(_)
        ));
        assert!(matches!(
            parse("hook.progress", json!({ "message": "step 1" })),
            SessionEventData::HookProgress(ref d) if d.message == "step 1"
        ));
    }

    #[test]
    fn test_parse_system_notification_event() {
        match parse(
            "system.notification",
            json!({
                "content": "Shell finished",
                "kind": { "type": "shell_completed", "shellId": "s1", "exitCode": 0 }
            }),
        ) {
            SessionEventData::SystemNotification(d) => {
                assert_eq!(d.content, "Shell finished");
                match d.kind {
                    SystemNotification::ShellCompleted {
                        shell_id,
                        exit_code,
                        ..
                    } => {
                        assert_eq!(shell_id, "s1");
                        assert_eq!(exit_code, Some(0));
                    }
                    other => panic!("unexpected kind: {other:?}"),
                }
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_parse_permission_completed_event() {
        match parse(
            "permission.completed",
            json!({
                "requestId": "r1",
                "toolCallId": "t1",
                "result": { "kind": "denied-by-rules", "rules": [{ "kind": "shell", "argument": null }] }
            }),
        ) {
            SessionEventData::PermissionCompleted(d) => {
                assert_eq!(d.request_id, "r1");
                match d.result {
                    PermissionResult::DeniedByRules { rules } => {
                        assert_eq!(rules.len(), 1);
                        assert_eq!(rules[0].kind, "shell");
                        assert!(rules[0].argument.is_none());
                    }
                    other => panic!("unexpected result: {other:?}"),
                }
            }
            other => panic!("unexpected: {other:?}"),
        }

        assert!(matches!(
            parse(
                "permission.completed",
                json!({ "requestId": "r2", "result": { "kind": "approved" } })
            ),
            SessionEventData::PermissionCompleted(ref d)
                if matches!(d.result, PermissionResult::Approved)
        ));
    }

    #[test]
    fn test_parse_request_completion_events() {
        assert!(matches!(
            parse(
                "user_input.requested",
                json!({ "requestId": "r", "question": "q", "choices": ["a", "b"] })
            ),
            SessionEventData::UserInputRequested(ref d)
                if d.choices.as_ref().map(|c| c.len()) == Some(2)
        ));
        assert!(matches!(
            parse(
                "user_input.completed",
                json!({ "requestId": "r", "answer": "a", "wasFreeform": true })
            ),
            SessionEventData::UserInputCompleted(ref d) if d.was_freeform == Some(true)
        ));
        assert!(matches!(
            parse(
                "elicitation.completed",
                json!({ "requestId": "r", "action": "accept", "content": { "name": "x" } })
            ),
            SessionEventData::ElicitationCompleted(ref d)
                if matches!(d.action, Some(ElicitationCompletedAction::Accept))
        ));
        assert!(matches!(
            parse(
                "sampling.requested",
                json!({ "requestId": "r", "serverName": "s", "mcpRequestId": 3, "extraField": 1 })
            ),
            SessionEventData::SamplingRequested(ref d) if d.extra.contains_key("extraField")
        ));
        assert!(matches!(
            parse("sampling.completed", json!({ "requestId": "r" })),
            SessionEventData::SamplingCompleted(_)
        ));
        assert!(matches!(
            parse("external_tool.completed", json!({ "requestId": "r" })),
            SessionEventData::ExternalToolCompleted(_)
        ));
        assert!(matches!(
            parse(
                "auto_mode_switch.completed",
                json!({ "requestId": "r", "response": "yes_always" })
            ),
            SessionEventData::AutoModeSwitchCompleted(ref d)
                if matches!(d.response, crate::types::AutoModeSwitchResponse::YesAlways)
        ));
        assert!(matches!(
            parse(
                "exit_plan_mode.completed",
                json!({ "requestId": "r", "approved": true, "selectedAction": "autopilot_fleet" })
            ),
            SessionEventData::ExitPlanModeCompleted(ref d)
                if matches!(d.selected_action, Some(ExitPlanModeAction::AutopilotFleet))
        ));
    }

    #[test]
    fn test_parse_mcp_events() {
        match parse(
            "mcp.oauth_required",
            json!({
                "requestId": "r",
                "serverName": "srv",
                "serverUrl": "https://x",
                "staticClientConfig": { "clientId": "c", "publicClient": true }
            }),
        ) {
            SessionEventData::McpOauthRequired(d) => {
                assert_eq!(d.server_url, "https://x");
                let cfg = d.static_client_config.expect("config");
                assert_eq!(cfg.client_id, "c");
                assert_eq!(cfg.public_client, Some(true));
            }
            other => panic!("unexpected: {other:?}"),
        }
        assert!(matches!(
            parse("mcp.oauth_completed", json!({ "requestId": "r" })),
            SessionEventData::McpOauthCompleted(_)
        ));
        match parse(
            "session.mcp_servers_loaded",
            json!({
                "servers": [
                    { "name": "a", "status": "needs-auth", "transport": "http", "source": "user" },
                    { "name": "b", "status": "not_configured" }
                ]
            }),
        ) {
            SessionEventData::SessionMcpServersLoaded(d) => {
                assert_eq!(d.servers.len(), 2);
                assert!(matches!(d.servers[0].status, McpServerStatus::NeedsAuth));
                assert!(matches!(
                    d.servers[0].transport,
                    Some(McpServerTransport::Http)
                ));
                assert!(matches!(
                    d.servers[1].status,
                    McpServerStatus::NotConfigured
                ));
            }
            other => panic!("unexpected: {other:?}"),
        }
        assert!(matches!(
            parse(
                "session.mcp_server_status_changed",
                json!({ "serverName": "a", "status": "connected" })
            ),
            SessionEventData::SessionMcpServerStatusChanged(ref d)
                if matches!(d.status, McpServerStatus::Connected)
        ));
        match parse(
            "mcp_app.tool_call_complete",
            json!({
                "serverName": "s",
                "toolName": "t",
                "success": false,
                "durationMs": 12,
                "error": { "message": "boom" }
            }),
        ) {
            SessionEventData::McpAppToolCallComplete(d) => {
                assert!(!d.success);
                assert_eq!(d.duration_ms, 12);
                assert_eq!(d.error.expect("error").message, "boom");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_parse_registry_events() {
        match parse(
            "commands.changed",
            json!({ "commands": [{ "name": "deploy", "description": "Ship it" }] }),
        ) {
            SessionEventData::CommandsChanged(d) => {
                assert_eq!(d.commands.len(), 1);
                assert_eq!(d.commands[0].name, "deploy");
            }
            other => panic!("unexpected: {other:?}"),
        }
        assert!(matches!(
            parse("command.queued", json!({ "requestId": "r", "command": "/x" })),
            SessionEventData::CommandQueued(ref d) if d.command == "/x"
        ));
        assert!(matches!(
            parse("command.completed", json!({ "requestId": "r" })),
            SessionEventData::CommandCompleted(_)
        ));
        assert!(matches!(
            parse(
                "capabilities.changed",
                json!({ "ui": { "canvases": true, "mcpApps": false } })
            ),
            SessionEventData::CapabilitiesChanged(ref d)
                if d.ui.as_ref().and_then(|u| u.canvases) == Some(true)
        ));
        match parse(
            "session.skills_loaded",
            json!({
                "skills": [{
                    "name": "s", "description": "d", "enabled": true,
                    "source": "personal-copilot", "userInvocable": true
                }]
            }),
        ) {
            SessionEventData::SessionSkillsLoaded(d) => {
                assert!(matches!(d.skills[0].source, SkillSource::PersonalCopilot));
            }
            other => panic!("unexpected: {other:?}"),
        }
        match parse(
            "session.custom_agents_updated",
            json!({
                "agents": [{
                    "id": "a", "name": "a", "displayName": "A", "description": "d",
                    "source": "project", "tools": null, "userInvocable": true
                }],
                "errors": [],
                "warnings": ["w"]
            }),
        ) {
            SessionEventData::SessionCustomAgentsUpdated(d) => {
                assert!(d.agents[0].tools.is_none());
                assert_eq!(d.warnings, vec!["w".to_string()]);
            }
            other => panic!("unexpected: {other:?}"),
        }
        match parse(
            "session.extensions_loaded",
            json!({
                "extensions": [
                    { "id": "e", "name": "E", "source": "project", "status": "running" }
                ]
            }),
        ) {
            SessionEventData::SessionExtensionsLoaded(d) => {
                assert!(matches!(d.extensions[0].source, ExtensionSource::Project));
                assert!(matches!(d.extensions[0].status, ExtensionStatus::Running));
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_parse_canvas_events() {
        match parse(
            "session.canvas.opened",
            json!({
                "availability": "ready",
                "canvasId": "c",
                "extensionId": "e",
                "instanceId": "i",
                "reopen": false,
                "title": "T"
            }),
        ) {
            SessionEventData::SessionCanvasOpened(d) => {
                assert_eq!(d.instance_id, "i");
                assert!(!d.reopen);
                assert!(matches!(
                    d.availability,
                    crate::canvas::CanvasInstanceAvailability::Ready
                ));
            }
            other => panic!("unexpected: {other:?}"),
        }
        match parse(
            "session.canvas.registry_changed",
            json!({
                "canvases": [{
                    "canvasId": "c", "displayName": "C", "description": "d",
                    "extensionId": "e",
                    "actions": [{ "name": "refresh" }]
                }]
            }),
        ) {
            SessionEventData::SessionCanvasRegistryChanged(d) => {
                assert_eq!(d.canvases.len(), 1);
                let actions = d.canvases[0].actions.as_ref().expect("actions");
                assert_eq!(actions[0].name, "refresh");
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_parse_custom_notification_event() {
        match parse(
            "session.custom_notification",
            json!({
                "name": "n",
                "source": "ext",
                "payload": { "a": 1 },
                "subject": { "k": "v" },
                "version": 2
            }),
        ) {
            SessionEventData::SessionCustomNotification(d) => {
                assert_eq!(d.name, "n");
                assert_eq!(d.version, Some(2));
                assert_eq!(
                    d.subject.expect("subject").get("k").map(String::as_str),
                    Some("v")
                );
            }
            other => panic!("unexpected: {other:?}"),
        }
    }

    #[test]
    fn test_wave3_events_parse_through_session_event() {
        let ev = SessionEvent::from_json(&json!({
            "id": "evt_1",
            "timestamp": "2026-01-01T00:00:00Z",
            "type": "session.title_changed",
            "parentId": null,
            "ephemeral": true,
            "data": { "title": "Hello" }
        }))
        .expect("event should parse");
        assert_eq!(ev.event_type, "session.title_changed");
        assert!(matches!(ev.data, SessionEventData::SessionTitleChanged(_)));
    }
}
