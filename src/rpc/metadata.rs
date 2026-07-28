// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Session metadata, naming, usage, telemetry, and instruction RPC bindings.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{Result, Session};

/// Accessor for `session.instructions.*` RPC methods.
pub struct SessionInstructions<'a> {
    session: &'a Session,
}

/// Accessor for `session.metadata.*` RPC methods.
///
/// This type is intentionally named `SessionMetadataApi` to avoid colliding
/// with existing `SessionMetadata` data types elsewhere in the crate.
pub struct SessionMetadataApi<'a> {
    session: &'a Session,
}

/// Accessor for `session.name.*` RPC methods.
pub struct SessionName<'a> {
    session: &'a Session,
}

/// Accessor for `session.telemetry.*` RPC methods.
pub struct SessionTelemetry<'a> {
    session: &'a Session,
}

/// Accessor for `session.usage.*` RPC methods.
pub struct SessionUsage<'a> {
    session: &'a Session,
}

/// Parameters for `session.metadata.contextInfo`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataContextInfoRequest {
    /// Target session identifier.
    pub session_id: String,
    /// Maximum prompt tokens allowed by the target model.
    ///
    /// Pass `0` to use the runtime default.
    pub prompt_token_limit: u64,
    /// Maximum output tokens allowed by the target model.
    ///
    /// Pass `0` if unknown.
    pub output_token_limit: u64,
    /// Model identifier used for tokenization.
    ///
    /// Omit to use the session default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_model: Option<String>,
}

/// Result for `session.metadata.contextInfo`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataContextInfoResult {
    /// Token breakdown for the current context window, or `None` when the
    /// session has not yet been initialized.
    pub context_info: Option<SessionContextInfo>,
}

/// Token-usage breakdown for the session's current context window.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionContextInfo {
    /// Model identifier used for token counting.
    pub model_name: String,
    /// Tokens consumed by the system prompt.
    pub system_tokens: u64,
    /// Tokens consumed by user, assistant, and tool messages.
    pub conversation_tokens: u64,
    /// Tokens consumed by tool definitions sent to the model.
    pub tool_definitions_tokens: u64,
    /// Tokens consumed by MCP tool definitions.
    pub mcp_tools_tokens: u64,
    /// Sum of system, conversation, and tool-definition tokens.
    pub total_tokens: u64,
    /// Maximum prompt tokens allowed by the model.
    pub prompt_token_limit: u64,
    /// Token count at which background compaction begins.
    pub compaction_threshold: u64,
    /// Total context limit displayed to the user.
    pub limit: u64,
    /// Output reserve plus tokens after the buffer threshold.
    pub buffer_tokens: u64,
}

/// Result for `session.metadata.isProcessing`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataIsProcessingResult {
    /// Whether the session is currently processing a turn or continuation.
    pub processing: bool,
}

/// Parameters for `session.metadata.recomputeContextTokens`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataRecomputeContextTokensRequest {
    /// Target session identifier.
    pub session_id: String,
    /// Model identifier used for tokenization.
    pub model_id: String,
}

/// Result for `session.metadata.recomputeContextTokens`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataRecomputeContextTokensResult {
    /// Sum of tokens across chat-context and system-context messages.
    pub total_tokens: u64,
    /// Tokens contributed by user, assistant, and tool messages.
    pub messages_token_count: u64,
    /// Tokens contributed by system and developer prompt snapshots.
    pub system_token_count: u64,
}

/// Parameters for `session.metadata.recordContextChange`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataRecordContextChangeRequest {
    /// Target session identifier.
    pub session_id: String,
    /// Updated working-directory and git-context snapshot.
    pub context: SessionWorkingDirectoryContext,
}

/// Updated working-directory and git context for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionWorkingDirectoryContext {
    /// Current working directory path.
    pub cwd: String,
    /// Root directory of the git repository, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_root: Option<String>,
    /// Repository identifier derived from the git remote URL, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// Hosting platform type of the repository, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_type: Option<SessionWorkingDirectoryContextHostType>,
    /// Raw host string from the git remote URL, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_host: Option<String>,
    /// Current git branch name, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Current HEAD commit SHA, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head_commit: Option<String>,
    /// Merge-base commit SHA, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_commit: Option<String>,
}

/// Repository hosting platform for a working-directory context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionWorkingDirectoryContextHostType {
    /// The repository is hosted on GitHub.
    Github,
    /// The repository is hosted on Azure DevOps.
    Ado,
}

/// Result for `session.metadata.recordContextChange`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataRecordContextChangeResult {}

/// Parameters for `session.metadata.setWorkingDirectory`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataSetWorkingDirectoryRequest {
    /// Target session identifier.
    pub session_id: String,
    /// Absolute path to set as the session's working directory.
    pub working_directory: String,
}

/// Result for `session.metadata.setWorkingDirectory`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataSetWorkingDirectoryResult {
    /// Working directory after the update.
    pub working_directory: String,
}

/// Point-in-time snapshot of slow-changing session identifier and state fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMetadataSnapshot {
    /// Unique identifier of the session.
    pub session_id: String,
    /// ISO 8601 timestamp of when the session started.
    pub start_time: String,
    /// ISO 8601 timestamp of the last persisted modification time.
    pub modified_time: String,
    /// Whether this is a remote session.
    pub is_remote: bool,
    /// Whether another process was already using the session at construction.
    pub already_in_use: bool,
    /// Absolute path to the workspace directory, or `None` when absent.
    pub workspace_path: Option<String>,
    /// User-provided name supplied at session construction, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_name: Option<String>,
    /// Runtime client name associated with the session, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,
    /// Remote-session-specific metadata when the session is remote.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_metadata: Option<MetadataSnapshotRemoteMetadata>,
    /// Short human-readable session summary, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Absolute path to the session's current working directory.
    pub working_directory: String,
    /// Current agent mode for the session.
    pub current_mode: MetadataSnapshotCurrentMode,
    /// Currently selected model identifier, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_model: Option<String>,
    /// Public-facing workspace metadata, or `None` when the session has none.
    pub workspace: Option<WorkspaceSummary>,
}

/// Current agent mode for a session snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetadataSnapshotCurrentMode {
    /// The agent is responding interactively.
    Interactive,
    /// The agent is preparing a plan.
    Plan,
    /// The agent is working autonomously.
    Autopilot,
}

/// Remote-session-specific metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataSnapshotRemoteMetadata {
    /// Original resource identifier preserved across reconstructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    /// Repository targeted by the remote session.
    pub repository: MetadataSnapshotRemoteMetadataRepository,
    /// Pull request number associated with the remote session, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_request_number: Option<u64>,
    /// Origin of the remote task, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<MetadataSnapshotRemoteMetadataTaskType>,
}

/// Repository descriptor within remote-session metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataSnapshotRemoteMetadataRepository {
    /// GitHub owner or organization.
    pub owner: String,
    /// Repository name without owner.
    pub name: String,
    /// Branch the remote session is operating on.
    pub branch: String,
}

/// Origin of a remote session task.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MetadataSnapshotRemoteMetadataTaskType {
    /// Task originated from Copilot Coding Agent.
    Cca,
    /// Task originated from a CLI remote-session invocation.
    Cli,
}

/// Public-facing workspace metadata for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSummary {
    /// Workspace identifier.
    pub id: String,
    /// Current working directory at session start, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Resolved git root for the working directory, if any.
    #[serde(rename = "git_root", skip_serializing_if = "Option::is_none")]
    pub git_root: Option<String>,
    /// Repository identifier, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// Repository host type, if known.
    #[serde(rename = "host_type", skip_serializing_if = "Option::is_none")]
    pub host_type: Option<WorkspaceSummaryHostType>,
    /// Branch checked out at session start, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Display name for the session, if set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// ISO 8601 timestamp when the workspace was created, if known.
    #[serde(rename = "created_at", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// ISO 8601 timestamp when the workspace was last updated, if known.
    #[serde(rename = "updated_at", skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

/// Repository host type for a workspace summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceSummaryHostType {
    /// The repository is hosted on GitHub.
    Github,
    /// The repository is hosted on Azure DevOps.
    Ado,
}

/// Result for `session.name.get`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NameGetResult {
    /// The session's friendly name, or `None` when not yet set.
    pub name: Option<String>,
}

/// Parameters for `session.name.set`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NameSetRequest {
    /// Target session identifier.
    pub session_id: String,
    /// New session name.
    pub name: String,
}

/// Parameters for `session.name.setAuto`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NameSetAutoRequest {
    /// Target session identifier.
    pub session_id: String,
    /// Auto-generated session summary to persist when permitted.
    pub summary: String,
}

/// Result for `session.name.setAuto`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NameSetAutoResult {
    /// Whether the auto-generated summary was applied.
    pub applied: bool,
}

/// Parameters for `session.telemetry.setFeatureOverrides`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetrySetFeatureOverridesRequest {
    /// Target session identifier.
    pub session_id: String,
    /// Replacement feature override key/value pairs.
    pub features: BTreeMap<String, String>,
}

/// Result for `session.usage.getMetrics`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageGetMetricsResult {
    /// Total user-initiated premium request cost across all models.
    pub total_premium_request_cost: f64,
    /// Raw count of user-initiated API requests.
    pub total_user_requests: u64,
    /// Session-wide accumulated nano-AI-unit cost, if reported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_nano_aiu: Option<f64>,
    /// Session-wide per-token-type accumulated token counts, if reported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_details: Option<BTreeMap<String, UsageMetricsTokenDetail>>,
    /// Total time spent in model API calls, in milliseconds.
    pub total_api_duration_ms: u64,
    /// ISO 8601 timestamp when the session started.
    pub session_start_time: String,
    /// Aggregated code-change metrics.
    pub code_changes: UsageMetricsCodeChanges,
    /// Per-model token and request metrics keyed by model identifier.
    pub model_metrics: BTreeMap<String, UsageMetricsModelMetric>,
    /// Currently active model identifier, if known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_model: Option<String>,
    /// Input tokens from the most recent main-agent API call.
    pub last_call_input_tokens: u64,
    /// Output tokens from the most recent main-agent API call.
    pub last_call_output_tokens: u64,
}

/// Aggregated code-change metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetricsCodeChanges {
    /// Total lines of code added.
    pub lines_added: u64,
    /// Total lines of code removed.
    pub lines_removed: u64,
    /// Number of distinct files modified.
    pub files_modified_count: u64,
    /// Distinct file paths modified during the session.
    pub files_modified: Vec<String>,
}

/// Per-model usage metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetricsModelMetric {
    /// Request count and cost metrics for the model.
    pub requests: UsageMetricsModelMetricRequests,
    /// Token-usage metrics for the model.
    pub usage: UsageMetricsModelMetricUsage,
    /// Accumulated nano-AI-unit cost for the model, if reported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_nano_aiu: Option<f64>,
    /// Token-count details per token type, if reported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_details: Option<BTreeMap<String, UsageMetricsModelMetricTokenDetail>>,
}

/// Request count and cost metrics for a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetricsModelMetricRequests {
    /// Number of API requests made with the model.
    pub count: u64,
    /// User-initiated premium request cost for the model.
    pub cost: f64,
}

/// Token-usage metrics for a model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetricsModelMetricUsage {
    /// Total input tokens consumed.
    pub input_tokens: u64,
    /// Total output tokens produced.
    pub output_tokens: u64,
    /// Total tokens read from prompt cache.
    pub cache_read_tokens: u64,
    /// Total tokens written to prompt cache.
    pub cache_write_tokens: u64,
    /// Total reasoning tokens produced, if reported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u64>,
}

/// Token-count detail for a model-scoped metric bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetricsModelMetricTokenDetail {
    /// Accumulated token count for the token type.
    pub token_count: u64,
}

/// Session-wide token-count detail for a token bucket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageMetricsTokenDetail {
    /// Accumulated token count for the token type.
    pub token_count: u64,
}

/// Result for `session.instructions.getSources`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionsGetSourcesResult {
    /// Instruction sources loaded for the session.
    pub sources: Vec<InstructionsSource>,
}

/// One loaded instruction source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstructionsSource {
    /// Unique identifier for this source.
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// File path relative to the repository or absolute for home-level files.
    pub source_path: String,
    /// Raw content of the instruction file.
    pub content: String,
    /// Category of instruction source.
    pub r#type: InstructionsSourceType,
    /// Where this source lives.
    pub location: InstructionsSourceLocation,
    /// Glob patterns limiting which files this instruction applies to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_to: Option<Vec<String>>,
    /// Short human-readable description for the source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Whether this source starts disabled by default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_disabled: Option<bool>,
}

/// Merge category for an instruction source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstructionsSourceType {
    /// Instructions loaded from the user's home configuration.
    #[serde(rename = "home")]
    Home,
    /// Instructions loaded from repository-scoped files.
    #[serde(rename = "repo")]
    Repo,
    /// Instructions loaded from model-specific files.
    #[serde(rename = "model")]
    Model,
    /// Instructions loaded from VS Code instruction files.
    #[serde(rename = "vscode")]
    VsCode,
    /// Instructions discovered from nested agent files.
    #[serde(rename = "nested-agents")]
    NestedAgents,
    /// Instructions inherited from child instruction files.
    #[serde(rename = "child-instructions")]
    ChildInstructions,
    /// Instructions supplied by an installed plugin.
    #[serde(rename = "plugin")]
    Plugin,
}

/// UI grouping for an instruction source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstructionsSourceLocation {
    /// Instructions live in user-level configuration.
    #[serde(rename = "user")]
    User,
    /// Instructions live in repository-level configuration.
    #[serde(rename = "repository")]
    Repository,
    /// Instructions live under the current working directory.
    #[serde(rename = "working-directory")]
    WorkingDirectory,
    /// Instructions live in plugin-provided configuration.
    #[serde(rename = "plugin")]
    Plugin,
}

impl Session {
    /// Access session metadata APIs.
    ///
    /// The returned [`SessionMetadataApi`] provides typed bindings for
    /// `session.metadata.*` RPC methods.
    pub fn metadata(&self) -> SessionMetadataApi<'_> {
        SessionMetadataApi { session: self }
    }

    /// Access session naming APIs.
    ///
    /// The returned [`SessionName`] provides typed bindings for
    /// `session.name.*` RPC methods.
    pub fn name(&self) -> SessionName<'_> {
        SessionName { session: self }
    }

    /// Access session usage metrics APIs.
    ///
    /// The returned [`SessionUsage`] provides typed bindings for
    /// `session.usage.*` RPC methods.
    pub fn usage(&self) -> SessionUsage<'_> {
        SessionUsage { session: self }
    }

    /// Access session telemetry APIs.
    ///
    /// The returned [`SessionTelemetry`] provides typed bindings for
    /// `session.telemetry.*` RPC methods.
    pub fn telemetry(&self) -> SessionTelemetry<'_> {
        SessionTelemetry { session: self }
    }

    /// Access session instruction-source APIs.
    ///
    /// The returned [`SessionInstructions`] provides typed bindings for
    /// `session.instructions.*` RPC methods.
    pub fn instructions(&self) -> SessionInstructions<'_> {
        SessionInstructions { session: self }
    }

    /// Suspend the session while preserving persisted state for later resume.
    pub async fn suspend(&self) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session_id,
        });
        (self.invoke_fn)("session.suspend", Some(params)).await?;
        Ok(())
    }
}

impl SessionInstructions<'_> {
    /// Return the instruction sources loaded for the session in merge order.
    pub async fn get_sources(&self) -> Result<InstructionsGetSourcesResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result =
            (self.session.invoke_fn)("session.instructions.getSources", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionMetadataApi<'_> {
    /// Return the current context-window token breakdown for a target model.
    pub async fn context_info(
        &self,
        prompt_token_limit: u64,
        output_token_limit: u64,
        selected_model: Option<&str>,
    ) -> Result<MetadataContextInfoResult> {
        let _request = MetadataContextInfoRequest {
            session_id: self.session.session_id.clone(),
            prompt_token_limit,
            output_token_limit,
            selected_model: selected_model.map(|value| value.to_owned()),
        };
        let mut params = serde_json::json!({
            "sessionId": self.session.session_id,
            "promptTokenLimit": prompt_token_limit,
            "outputTokenLimit": output_token_limit,
        });
        if let Some(selected_model) = selected_model {
            params["selectedModel"] = serde_json::json!(selected_model);
        }
        let result = (self.session.invoke_fn)("session.metadata.contextInfo", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Report whether the local session is currently processing work.
    pub async fn is_processing(&self) -> Result<MetadataIsProcessingResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result =
            (self.session.invoke_fn)("session.metadata.isProcessing", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Re-tokenize the session's existing messages against a model.
    pub async fn recompute_context_tokens(
        &self,
        model_id: &str,
    ) -> Result<MetadataRecomputeContextTokensResult> {
        let _request = MetadataRecomputeContextTokensRequest {
            session_id: self.session.session_id.clone(),
            model_id: model_id.to_owned(),
        };
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "modelId": model_id,
        });
        let result =
            (self.session.invoke_fn)("session.metadata.recomputeContextTokens", Some(params))
                .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Record an updated working-directory and git context on the session.
    pub async fn record_context_change(
        &self,
        context: SessionWorkingDirectoryContext,
    ) -> Result<MetadataRecordContextChangeResult> {
        let _request = MetadataRecordContextChangeRequest {
            session_id: self.session.session_id.clone(),
            context: context.clone(),
        };
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "context": context,
        });
        let result =
            (self.session.invoke_fn)("session.metadata.recordContextChange", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Update the session's recorded working directory.
    pub async fn set_working_directory(
        &self,
        working_directory: &str,
    ) -> Result<MetadataSetWorkingDirectoryResult> {
        let _request = MetadataSetWorkingDirectoryRequest {
            session_id: self.session.session_id.clone(),
            working_directory: working_directory.to_owned(),
        };
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "workingDirectory": working_directory,
        });
        let result =
            (self.session.invoke_fn)("session.metadata.setWorkingDirectory", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Return a snapshot of the session's identifying metadata and state.
    pub async fn snapshot(&self) -> Result<SessionMetadataSnapshot> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result = (self.session.invoke_fn)("session.metadata.snapshot", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionName<'_> {
    /// Get the session's friendly name.
    pub async fn get(&self) -> Result<NameGetResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result = (self.session.invoke_fn)("session.name.get", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Set the session's friendly name.
    pub async fn set(&self, name: &str) -> Result<()> {
        let _request = NameSetRequest {
            session_id: self.session.session_id.clone(),
            name: name.to_owned(),
        };
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "name": name,
        });
        (self.session.invoke_fn)("session.name.set", Some(params)).await?;
        Ok(())
    }

    /// Persist an auto-generated summary as the session's name when allowed.
    pub async fn set_auto(&self, summary: &str) -> Result<NameSetAutoResult> {
        let _request = NameSetAutoRequest {
            session_id: self.session.session_id.clone(),
            summary: summary.to_owned(),
        };
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "summary": summary,
        });
        let result = (self.session.invoke_fn)("session.name.setAuto", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionTelemetry<'_> {
    /// Replace feature override key/value pairs attached to session telemetry.
    pub async fn set_feature_overrides(&self, features: BTreeMap<String, String>) -> Result<()> {
        let _request = TelemetrySetFeatureOverridesRequest {
            session_id: self.session.session_id.clone(),
            features: features.clone(),
        };
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "features": features,
        });
        (self.session.invoke_fn)("session.telemetry.setFeatureOverrides", Some(params)).await?;
        Ok(())
    }
}

impl SessionUsage<'_> {
    /// Get accumulated usage metrics for the session.
    pub async fn get_metrics(&self) -> Result<UsageGetMetricsResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result = (self.session.invoke_fn)("session.usage.getMetrics", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}
