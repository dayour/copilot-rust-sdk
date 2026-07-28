// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! History, event log, queue, schedule, remote, model, tools, and auth RPC bindings.

use crate::{Result, Session};
use serde_json::Value;
use std::collections::HashMap;

/// Access the session agent-management RPC APIs that are not exposed as direct [`Session`] helpers.
pub struct SessionAgent<'a> {
    session: &'a Session,
}

/// Access the session authentication RPC APIs.
pub struct SessionAuth<'a> {
    session: &'a Session,
}

/// Access the session event-log RPC APIs.
pub struct SessionEventLog<'a> {
    session: &'a Session,
}

/// Access the session history-management RPC APIs.
pub struct SessionHistory<'a> {
    session: &'a Session,
}

/// Access the session model-management RPC APIs that complement the direct [`Session`] helpers.
pub struct SessionModel<'a> {
    session: &'a Session,
}

/// Access the session queue-management RPC APIs.
pub struct SessionQueue<'a> {
    session: &'a Session,
}

/// Access the session remote-control RPC APIs.
pub struct SessionRemote<'a> {
    session: &'a Session,
}

/// Access the session scheduling RPC APIs.
pub struct SessionSchedule<'a> {
    session: &'a Session,
}

/// Access the session tool-runtime RPC APIs.
pub struct SessionTools<'a> {
    session: &'a Session,
}

impl Session {
    /// Access the agent RPC APIs.
    pub fn agent(&self) -> SessionAgent<'_> {
        SessionAgent { session: self }
    }

    /// Access the authentication RPC APIs.
    pub fn auth(&self) -> SessionAuth<'_> {
        SessionAuth { session: self }
    }

    /// Access the event-log RPC APIs.
    pub fn event_log(&self) -> SessionEventLog<'_> {
        SessionEventLog { session: self }
    }

    /// Access the history RPC APIs.
    pub fn history(&self) -> SessionHistory<'_> {
        SessionHistory { session: self }
    }

    /// Access the model RPC APIs.
    pub fn model(&self) -> SessionModel<'_> {
        SessionModel { session: self }
    }

    /// Access the queue RPC APIs.
    pub fn queue(&self) -> SessionQueue<'_> {
        SessionQueue { session: self }
    }

    /// Access the remote-control RPC APIs.
    pub fn remote(&self) -> SessionRemote<'_> {
        SessionRemote { session: self }
    }

    /// Access the scheduling RPC APIs.
    pub fn schedule(&self) -> SessionSchedule<'_> {
        SessionSchedule { session: self }
    }

    /// Access the tool-runtime RPC APIs.
    pub fn tools(&self) -> SessionTools<'_> {
        SessionTools { session: self }
    }
}

/// Where an agent definition was loaded from.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionRpcAgentInfoSource {
    /// Agent loaded from the user's personal agent configuration.
    User,
    /// Agent loaded from the current repository or project configuration.
    Project,
    /// Agent inherited from an outer configuration scope.
    Inherited,
    /// Agent obtained from a remote source.
    Remote,
    /// Agent contributed by a plugin.
    Plugin,
    /// Built-in agent bundled with the runtime.
    Builtin,
}

/// Detailed information about a custom agent available to the session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcAgentInfo {
    /// Unique identifier of the custom agent.
    pub name: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Description of the agent's purpose.
    pub description: String,
    /// Absolute local file path of the agent definition, when file-backed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Stable identifier for selection.
    pub id: String,
    /// Where the agent definition was loaded from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SessionRpcAgentInfoSource>,
    /// Whether the agent can be selected directly by the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_invocable: Option<bool>,
    /// Allowed tool names for this agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    /// Preferred model id for this agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// MCP server configurations keyed by server name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<HashMap<String, Value>>,
    /// Skill names preloaded into this agent's context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
}

/// Result returned after reloading custom agents from disk.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcAgentReloadResult {
    /// Refreshed list of custom agents available to the session.
    pub agents: Vec<SessionRpcAgentInfo>,
}

/// Authentication type currently associated with the session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionRpcAuthInfoType {
    /// GitHub App HMAC credentials.
    Hmac,
    /// Token resolved from an environment variable.
    Env,
    /// Interactive user OAuth credentials.
    User,
    /// Authentication delegated to the GitHub CLI.
    #[serde(rename = "gh-cli")]
    GhCli,
    /// API-key credentials for a non-GitHub provider.
    #[serde(rename = "api-key")]
    ApiKey,
    /// SDK-provided GitHub token.
    Token,
    /// Direct Copilot API token authentication.
    #[serde(rename = "copilot-api-token")]
    CopilotApiToken,
}

/// Endpoint URLs returned in a Copilot user snapshot.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcCopilotUserResponseEndpoints {
    /// API endpoint URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api: Option<String>,
    /// Origin-tracker endpoint URL.
    #[serde(rename = "origin-tracker", skip_serializing_if = "Option::is_none")]
    pub origin_tracker: Option<String>,
    /// Proxy endpoint URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,
    /// Telemetry endpoint URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telemetry: Option<String>,
}

/// Organization entry embedded in a Copilot user snapshot.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcCopilotOrganization {
    /// Organization login, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<String>,
    /// Organization display name, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Snapshot of the authenticated user's Copilot subscription information.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcCopilotUserResponse {
    /// Authenticated login name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<String>,
    /// Copilot access SKU identifier.
    #[serde(rename = "access_type_sku", skip_serializing_if = "Option::is_none")]
    pub access_type_sku: Option<String>,
    /// Analytics tracking identifier.
    #[serde(
        rename = "analytics_tracking_id",
        skip_serializing_if = "Option::is_none"
    )]
    pub analytics_tracking_id: Option<String>,
    /// Assigned date, when present.
    #[serde(rename = "assigned_date", skip_serializing_if = "Option::is_none")]
    pub assigned_date: Option<String>,
    /// Whether the user can sign up for the limited plan.
    #[serde(
        rename = "can_signup_for_limited",
        skip_serializing_if = "Option::is_none"
    )]
    pub can_signup_for_limited: Option<bool>,
    /// Whether chat is enabled.
    #[serde(rename = "chat_enabled", skip_serializing_if = "Option::is_none")]
    pub chat_enabled: Option<bool>,
    /// Copilot plan tier.
    #[serde(rename = "copilot_plan", skip_serializing_if = "Option::is_none")]
    pub copilot_plan: Option<String>,
    /// Whether `.copilotignore` support is enabled.
    #[serde(
        rename = "copilotignore_enabled",
        skip_serializing_if = "Option::is_none"
    )]
    pub copilotignore_enabled: Option<bool>,
    /// Endpoint URLs associated with the authenticated user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<SessionRpcCopilotUserResponseEndpoints>,
    /// Organization login names associated with the user.
    #[serde(
        rename = "organization_login_list",
        skip_serializing_if = "Option::is_none"
    )]
    pub organization_login_list: Option<Vec<String>>,
    /// Organization entries associated with the user.
    #[serde(rename = "organization_list", skip_serializing_if = "Option::is_none")]
    pub organization_list: Option<Vec<Option<SessionRpcCopilotOrganization>>>,
    /// Whether Codex agent features are enabled.
    #[serde(
        rename = "codex_agent_enabled",
        skip_serializing_if = "Option::is_none"
    )]
    pub codex_agent_enabled: Option<bool>,
    /// Whether MCP features are enabled.
    #[serde(rename = "is_mcp_enabled", skip_serializing_if = "Option::is_none")]
    pub is_mcp_enabled: Option<bool>,
    /// Quota reset date in local display form.
    #[serde(rename = "quota_reset_date", skip_serializing_if = "Option::is_none")]
    pub quota_reset_date: Option<String>,
    /// Quota snapshot payload keyed by quota type.
    #[serde(rename = "quota_snapshots", skip_serializing_if = "Option::is_none")]
    pub quota_snapshots: Option<Value>,
    /// Whether telemetry is restricted.
    #[serde(
        rename = "restricted_telemetry",
        skip_serializing_if = "Option::is_none"
    )]
    pub restricted_telemetry: Option<bool>,
    /// Whether token-based billing is enabled.
    #[serde(
        rename = "token_based_billing",
        skip_serializing_if = "Option::is_none"
    )]
    pub token_based_billing: Option<bool>,
    /// Quota reset date in UTC form.
    #[serde(
        rename = "quota_reset_date_utc",
        skip_serializing_if = "Option::is_none"
    )]
    pub quota_reset_date_utc: Option<String>,
    /// Limited-user quota payload keyed by quota name.
    #[serde(
        rename = "limited_user_quotas",
        skip_serializing_if = "Option::is_none"
    )]
    pub limited_user_quotas: Option<HashMap<String, f64>>,
    /// Limited-user reset date.
    #[serde(
        rename = "limited_user_reset_date",
        skip_serializing_if = "Option::is_none"
    )]
    pub limited_user_reset_date: Option<String>,
    /// Monthly quota payload keyed by quota name.
    #[serde(rename = "monthly_quotas", skip_serializing_if = "Option::is_none")]
    pub monthly_quotas: Option<HashMap<String, f64>>,
    /// Whether cloud session storage is enabled.
    #[serde(
        rename = "cloud_session_storage_enabled",
        skip_serializing_if = "Option::is_none"
    )]
    pub cloud_session_storage_enabled: Option<bool>,
    /// Whether CLI remote control is enabled.
    #[serde(
        rename = "cli_remote_control_enabled",
        skip_serializing_if = "Option::is_none"
    )]
    pub cli_remote_control_enabled: Option<bool>,
}

/// Auth credential payload accepted by `session.auth.setCredentials`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SessionRpcAuthInfo {
    /// HMAC-based authentication used by GitHub-internal services.
    Hmac {
        /// Authentication host.
        host: String,
        /// HMAC secret used to sign requests.
        hmac: String,
        /// Snapshot of the authenticated user's Copilot subscription info.
        #[serde(skip_serializing_if = "Option::is_none")]
        copilot_user: Option<SessionRpcCopilotUserResponse>,
    },
    /// Personal access token or server token sourced from an environment variable.
    Env {
        /// Authentication host.
        host: String,
        /// User login associated with the token, when known.
        #[serde(skip_serializing_if = "Option::is_none")]
        login: Option<String>,
        /// The token value itself.
        token: String,
        /// Name of the environment variable the token was sourced from.
        env_var: String,
        /// Snapshot of the authenticated user's Copilot subscription info.
        #[serde(skip_serializing_if = "Option::is_none")]
        copilot_user: Option<SessionRpcCopilotUserResponse>,
    },
    /// SDK-side token authentication configured directly by the caller.
    Token {
        /// Authentication host.
        host: String,
        /// The token value itself.
        token: String,
        /// Snapshot of the authenticated user's Copilot subscription info.
        #[serde(skip_serializing_if = "Option::is_none")]
        copilot_user: Option<SessionRpcCopilotUserResponse>,
    },
    /// Direct Copilot API authentication via environment-provided token settings.
    #[serde(rename = "copilot-api-token")]
    CopilotApiToken {
        /// Authentication host.
        host: String,
        /// Snapshot of the authenticated user's Copilot subscription info.
        #[serde(skip_serializing_if = "Option::is_none")]
        copilot_user: Option<SessionRpcCopilotUserResponse>,
    },
    /// OAuth user authentication backed by the runtime's token store.
    User {
        /// Authentication host.
        host: String,
        /// OAuth user login.
        login: String,
        /// Snapshot of the authenticated user's Copilot subscription info.
        #[serde(skip_serializing_if = "Option::is_none")]
        copilot_user: Option<SessionRpcCopilotUserResponse>,
    },
    /// Authentication delegated to the GitHub CLI.
    #[serde(rename = "gh-cli")]
    GhCli {
        /// Authentication host.
        host: String,
        /// User login reported by `gh auth status`.
        login: String,
        /// Token returned by `gh auth token`.
        token: String,
        /// Snapshot of the authenticated user's Copilot subscription info.
        #[serde(skip_serializing_if = "Option::is_none")]
        copilot_user: Option<SessionRpcCopilotUserResponse>,
    },
    /// API-key authentication for non-GitHub providers.
    #[serde(rename = "api-key")]
    ApiKey {
        /// The API key value.
        api_key: String,
        /// Authentication host.
        host: String,
        /// Snapshot of the authenticated user's Copilot subscription info.
        #[serde(skip_serializing_if = "Option::is_none")]
        copilot_user: Option<SessionRpcCopilotUserResponse>,
    },
}

/// Authentication status and account metadata for the session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcSessionAuthStatus {
    /// Whether the session has resolved authentication.
    pub is_authenticated: bool,
    /// Authentication type, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_type: Option<SessionRpcAuthInfoType>,
    /// Authentication host URL, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    /// Authenticated login or username, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<String>,
    /// Human-readable authentication status description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_message: Option<String>,
    /// Copilot plan tier, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copilot_plan: Option<String>,
}

/// Result returned after updating the session's credentials.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcSetCredentialsResult {
    /// Whether the operation succeeded.
    pub success: bool,
}

/// Agent-scope filter for event-log reads.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionRpcEventsAgentScope {
    /// Return only primary-agent events and typed subagent lifecycle events.
    Primary,
    /// Return events from all agents.
    All,
}

/// Cursor status returned from an event-log read.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionRpcEventsCursorStatus {
    /// The supplied cursor was applied successfully.
    Ok,
    /// The supplied cursor referred to history that no longer exists.
    Expired,
}

/// Options for reading batches from the persisted session event log.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcEventLogReadOptions {
    /// Opaque cursor returned by a previous read.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Maximum number of events to return in this batch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<u64>,
    /// Milliseconds to wait for new events when already at the tail.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wait_ms: Option<u64>,
    /// Either `"*"` or a list of event type strings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<Value>,
    /// Agent-scope filter to apply while reading.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_scope: Option<SessionRpcEventsAgentScope>,
}

/// Batch of events returned by `session.eventLog.read`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcEventsReadResult {
    /// Event payloads returned by the runtime in read order.
    pub events: Vec<Value>,
    /// Opaque cursor for the next read.
    pub cursor: String,
    /// Whether more events are available immediately.
    pub has_more: bool,
    /// Status of the supplied cursor.
    pub cursor_status: SessionRpcEventsCursorStatus,
}

/// Opaque handle returned from `registerInterest`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcRegisterEventInterestResult {
    /// Registration handle to pass to `releaseInterest`.
    pub handle: String,
}

/// Result returned after releasing event-log interest.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcEventLogReleaseInterestResult {
    /// Whether the operation succeeded.
    pub success: bool,
}

/// Tail snapshot of the event log.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcEventLogTailResult {
    /// Opaque cursor pointing at the current tail.
    pub cursor: String,
}

/// Result returned after attempting to abort manual history compaction.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcHistoryAbortManualCompactionResult {
    /// Whether an in-progress manual compaction was aborted.
    pub aborted: bool,
}

/// Result returned after attempting to cancel background compaction.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcHistoryCancelBackgroundCompactionResult {
    /// Whether an in-progress background compaction was cancelled.
    pub cancelled: bool,
}

/// Markdown summary produced for hand-off scenarios.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcHistorySummarizeForHandoffResult {
    /// Markdown summary of the conversation context.
    pub summary: String,
}

/// Result returned after truncating session history.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcHistoryTruncateResult {
    /// Number of events removed by the truncation.
    pub events_removed: u64,
}

/// Optional inputs for listing models available to the session.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcModelListOptions {
    /// When true, bypass the per-session model-list cache.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_cache: Option<bool>,
}

/// Result returned from `session.model.list`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcModelListResult {
    /// Available models ordered with the most preferred default first.
    pub list: Vec<Value>,
    /// Per-quota snapshots returned alongside the model list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_snapshots: Option<Value>,
}

/// Result returned after updating the session's reasoning effort.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcModelSetReasoningEffortResult {
    /// Reasoning effort level recorded on the session after the update.
    pub reasoning_effort: String,
}

/// Kind of user-facing item currently pending in the queue.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionRpcQueuePendingItemKind {
    /// A queued user message.
    Message,
    /// A queued slash command or model-change command.
    Command,
}

/// One pending user-facing queue entry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcQueuePendingItem {
    /// Whether this entry is a message or command.
    pub kind: SessionRpcQueuePendingItemKind,
    /// Human-readable display text for the queue entry.
    pub display_text: String,
}

/// Snapshot of pending user-facing queue entries.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcQueuePendingItemsResult {
    /// Pending queued items in submission order.
    pub items: Vec<SessionRpcQueuePendingItem>,
    /// Display text for immediate steering messages awaiting delivery.
    pub steering_messages: Vec<String>,
}

/// Result returned after removing the most recently queued item.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcQueueRemoveMostRecentResult {
    /// Whether a removable user-facing pending item was removed.
    pub removed: bool,
}

/// Remote session mode to apply for `session.remote.enable`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionRpcRemoteSessionMode {
    /// Disable remote session export and steering.
    Off,
    /// Export session events without enabling remote steering.
    Export,
    /// Enable both export and remote steering.
    On,
}

/// Optional inputs for `session.remote.enable`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcRemoteEnableOptions {
    /// Per-session remote mode to apply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<SessionRpcRemoteSessionMode>,
}

/// Result returned after enabling remote export or steering.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcRemoteEnableResult {
    /// GitHub frontend URL for the session, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Whether remote steering is enabled.
    pub remote_steerable: bool,
}

/// Result returned after persisting a remote-steerability change.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcRemoteNotifySteerableChangedResult {}

/// One active scheduled prompt.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcScheduleEntry {
    /// Sequential id assigned by the runtime within the session.
    pub id: u64,
    /// Interval between scheduled ticks, in milliseconds.
    pub interval_ms: u64,
    /// Prompt text that is enqueued on every tick.
    pub prompt: String,
    /// Whether the schedule is recurring.
    pub recurring: bool,
    /// Display-only label for the prompt as shown in the UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_prompt: Option<String>,
    /// ISO 8601 timestamp for the next scheduled run.
    pub next_run_at: String,
}

/// Snapshot of active scheduled prompts.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcScheduleList {
    /// Active scheduled prompts ordered by id.
    pub entries: Vec<SessionRpcScheduleEntry>,
}

/// Result returned after removing a scheduled prompt.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcScheduleStopResult {
    /// The removed entry, or `None` when the id was unknown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry: Option<SessionRpcScheduleEntry>,
}

/// Lightweight metadata for one initialized session tool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcCurrentToolMetadata {
    /// Model-facing tool name.
    pub name: String,
    /// Optional namespaced tool name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespaced_name: Option<String>,
    /// MCP server name for MCP-backed tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_server_name: Option<String>,
    /// Raw MCP tool name for MCP-backed tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_tool_name: Option<String>,
    /// Tool description.
    pub description: String,
    /// JSON Schema describing the tool input.
    #[serde(rename = "input_schema", skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<Value>,
    /// Whether the tool is loaded on demand via tool search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defer_loading: Option<bool>,
}

/// Result returned by `session.tools.getCurrentMetadata`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcToolsGetCurrentMetadataResult {
    /// Current tool metadata, or `None` when tools have not been initialized yet.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<SessionRpcCurrentToolMetadata>>,
}

/// Result returned by `session.tools.initializeAndValidate`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRpcToolsInitializeAndValidateResult {}

impl SessionAgent<'_> {
    /// Reload custom agent definitions and return the refreshed list.
    pub async fn reload(&self) -> Result<SessionRpcAgentReloadResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result = (self.session.invoke_fn)("session.agent.reload", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionAuth<'_> {
    /// Get authentication status and account metadata for the session.
    pub async fn get_status(&self) -> Result<SessionRpcSessionAuthStatus> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result = (self.session.invoke_fn)("session.auth.getStatus", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Update the session's outbound authentication credentials.
    ///
    /// Passing `None` is a no-op that preserves the session's existing credentials.
    pub async fn set_credentials(
        &self,
        credentials: Option<SessionRpcAuthInfo>,
    ) -> Result<SessionRpcSetCredentialsResult> {
        let mut params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        if let Some(credentials) = credentials {
            params["credentials"] = serde_json::to_value(credentials)?;
        }
        let result = (self.session.invoke_fn)("session.auth.setCredentials", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionEventLog<'_> {
    /// Read a batch of session events from a cursor, optionally waiting for new events.
    pub async fn read(
        &self,
        options: SessionRpcEventLogReadOptions,
    ) -> Result<SessionRpcEventsReadResult> {
        let mut params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        if let Some(cursor) = options.cursor {
            params["cursor"] = serde_json::json!(cursor);
        }
        if let Some(max) = options.max {
            params["max"] = serde_json::json!(max);
        }
        if let Some(wait_ms) = options.wait_ms {
            params["waitMs"] = serde_json::json!(wait_ms);
        }
        if let Some(types) = options.types {
            params["types"] = types;
        }
        if let Some(agent_scope) = options.agent_scope {
            params["agentScope"] = serde_json::to_value(agent_scope)?;
        }
        let result = (self.session.invoke_fn)("session.eventLog.read", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Register consumer interest in an event type for runtime gating purposes.
    pub async fn register_interest(
        &self,
        event_type: &str,
    ) -> Result<SessionRpcRegisterEventInterestResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "eventType": event_type,
        });
        let result =
            (self.session.invoke_fn)("session.eventLog.registerInterest", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Release a previously-registered event-type interest handle.
    pub async fn release_interest(
        &self,
        handle: &str,
    ) -> Result<SessionRpcEventLogReleaseInterestResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "handle": handle,
        });
        let result =
            (self.session.invoke_fn)("session.eventLog.releaseInterest", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Return a snapshot of the current tail cursor without consuming events.
    pub async fn tail(&self) -> Result<SessionRpcEventLogTailResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result = (self.session.invoke_fn)("session.eventLog.tail", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionHistory<'_> {
    /// Abort any in-progress manual compaction on a local session.
    pub async fn abort_manual_compaction(
        &self,
    ) -> Result<SessionRpcHistoryAbortManualCompactionResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result =
            (self.session.invoke_fn)("session.history.abortManualCompaction", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Cancel any in-progress background compaction on a local session.
    pub async fn cancel_background_compaction(
        &self,
    ) -> Result<SessionRpcHistoryCancelBackgroundCompactionResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result =
            (self.session.invoke_fn)("session.history.cancelBackgroundCompaction", Some(params))
                .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Produce a markdown summary of the session context for hand-off scenarios.
    pub async fn summarize_for_handoff(
        &self,
    ) -> Result<SessionRpcHistorySummarizeForHandoffResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result =
            (self.session.invoke_fn)("session.history.summarizeForHandoff", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Truncate persisted session history to the specified event.
    pub async fn truncate(&self, event_id: &str) -> Result<SessionRpcHistoryTruncateResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "eventId": event_id,
        });
        let result = (self.session.invoke_fn)("session.history.truncate", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionModel<'_> {
    /// List models available to this session in its current auth and integration context.
    pub async fn list(
        &self,
        options: Option<SessionRpcModelListOptions>,
    ) -> Result<SessionRpcModelListResult> {
        let mut params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        if let Some(options) = options {
            if let Some(skip_cache) = options.skip_cache {
                params["skipCache"] = serde_json::json!(skip_cache);
            }
        }
        let result = (self.session.invoke_fn)("session.model.list", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Update the session's reasoning effort without changing the selected model.
    pub async fn set_reasoning_effort(
        &self,
        reasoning_effort: &str,
    ) -> Result<SessionRpcModelSetReasoningEffortResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "reasoningEffort": reasoning_effort,
        });
        let result =
            (self.session.invoke_fn)("session.model.setReasoningEffort", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionQueue<'_> {
    /// Clear all pending queued items on the local session.
    pub async fn clear(&self) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        (self.session.invoke_fn)("session.queue.clear", Some(params)).await?;
        Ok(())
    }

    /// Return the session's pending user-facing queued items and steering messages.
    pub async fn pending_items(&self) -> Result<SessionRpcQueuePendingItemsResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result = (self.session.invoke_fn)("session.queue.pendingItems", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Remove the most recently queued user-facing item.
    pub async fn remove_most_recent(&self) -> Result<SessionRpcQueueRemoveMostRecentResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result =
            (self.session.invoke_fn)("session.queue.removeMostRecent", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionRemote<'_> {
    /// Disable remote session export and steering.
    pub async fn disable(&self) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        (self.session.invoke_fn)("session.remote.disable", Some(params)).await?;
        Ok(())
    }

    /// Enable remote session export or steering.
    pub async fn enable(
        &self,
        options: Option<SessionRpcRemoteEnableOptions>,
    ) -> Result<SessionRpcRemoteEnableResult> {
        let mut params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        if let Some(options) = options {
            if let Some(mode) = options.mode {
                params["mode"] = serde_json::to_value(mode)?;
            }
        }
        let result = (self.session.invoke_fn)("session.remote.enable", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Persist a remote-steerability change emitted by the host as a session event.
    pub async fn notify_steerable_changed(
        &self,
        remote_steerable: bool,
    ) -> Result<SessionRpcRemoteNotifySteerableChangedResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "remoteSteerable": remote_steerable,
        });
        let result =
            (self.session.invoke_fn)("session.remote.notifySteerableChanged", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionSchedule<'_> {
    /// List the session's currently active scheduled prompts.
    pub async fn list(&self) -> Result<SessionRpcScheduleList> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result = (self.session.invoke_fn)("session.schedule.list", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Remove a scheduled prompt by id.
    pub async fn stop(&self, id: u64) -> Result<SessionRpcScheduleStopResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "id": id,
        });
        let result = (self.session.invoke_fn)("session.schedule.stop", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionTools<'_> {
    /// Return lightweight metadata for the session's currently initialized tools.
    pub async fn get_current_metadata(&self) -> Result<SessionRpcToolsGetCurrentMetadataResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result =
            (self.session.invoke_fn)("session.tools.getCurrentMetadata", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Resolve, build, and validate the runtime tool list for this session.
    pub async fn initialize_and_validate(
        &self,
    ) -> Result<SessionRpcToolsInitializeAndValidateResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result =
            (self.session.invoke_fn)("session.tools.initializeAndValidate", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}
