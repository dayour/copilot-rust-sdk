// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Background task RPC bindings.

use crate::{Result, Session};

/// Namespace accessor for `session.tasks.*` RPC methods.
pub struct SessionTasks<'a> {
    session: &'a Session,
}

/// Request payload for cancelling a tracked task.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksCancelParams {
    /// Identifier of the background task to cancel.
    pub id: String,
}

/// Request payload for fetching task progress.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksGetProgressParams {
    /// Identifier of the background task to inspect.
    pub id: String,
}

/// Request payload for promoting a task to background mode.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksPromoteToBackgroundParams {
    /// Identifier of the task to promote.
    pub id: String,
}

/// Request payload for removing a tracked task.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksRemoveParams {
    /// Identifier of the completed or cancelled task to remove.
    pub id: String,
}

/// Request payload for sending a follow-up message to a background agent task.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksSendMessageParams {
    /// Identifier of the target agent task.
    pub id: String,
    /// Message content to deliver to the agent.
    pub message: String,
    /// Optional sender agent ID when relaying on behalf of another agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_agent_id: Option<String>,
}

/// Request payload for starting a background agent task.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksStartAgentParams {
    /// Type of agent to start.
    pub agent_type: String,
    /// Task prompt passed to the agent.
    pub prompt: String,
    /// Short name used to generate a human-readable task ID.
    pub name: String,
    /// Optional short description of the task.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional model override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Lifecycle state of a tracked task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    /// The task is actively executing.
    Running,
    /// The task is waiting for more input.
    Idle,
    /// The task finished successfully.
    Completed,
    /// The task finished with an error.
    Failed,
    /// The task was cancelled before completion.
    Cancelled,
}

/// Execution mode for a tracked task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskExecutionMode {
    /// The task is synchronously awaited.
    Sync,
    /// The task is managed in the background.
    Background,
}

/// Attachment mode for a shell task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskShellInfoAttachmentMode {
    /// The shell runs in a managed PTY session.
    Attached,
    /// The shell runs as an independent background process.
    Detached,
}

/// Display-friendly activity line emitted while a task is running.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskProgressLine {
    /// Rendered message describing the activity event.
    pub message: String,
    /// ISO 8601 timestamp when the event occurred.
    pub timestamp: String,
}

/// Metadata for a tracked background agent task.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskAgentInfo {
    /// Task discriminator.
    #[serde(rename = "type")]
    pub kind: String,
    /// Unique task identifier.
    pub id: String,
    /// Tool call ID associated with this agent task.
    pub tool_call_id: String,
    /// Short task description.
    pub description: String,
    /// Current lifecycle status.
    pub status: TaskStatus,
    /// ISO 8601 timestamp when the task started.
    pub started_at: String,
    /// ISO 8601 timestamp when the task completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// Accumulated active execution time in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_time_ms: Option<u64>,
    /// ISO 8601 timestamp when the current active period began.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_started_at: Option<String>,
    /// Error message when the task failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Agent type used for the task.
    pub agent_type: String,
    /// Prompt originally sent to the agent.
    pub prompt: String,
    /// Final task result text, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    /// Model ID used for the task, when specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Whether the task is sync-waited or backgrounded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_mode: Option<TaskExecutionMode>,
    /// Whether the task can currently be promoted to background mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_promote_to_background: Option<bool>,
    /// Latest response text emitted by the agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_response: Option<String>,
    /// ISO 8601 timestamp when the agent entered idle state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idle_since: Option<String>,
}

/// Metadata for a tracked shell task.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskShellInfo {
    /// Task discriminator.
    #[serde(rename = "type")]
    pub kind: String,
    /// Unique task identifier.
    pub id: String,
    /// Short task description.
    pub description: String,
    /// Current lifecycle status.
    pub status: TaskStatus,
    /// ISO 8601 timestamp when the task started.
    pub started_at: String,
    /// ISO 8601 timestamp when the task completed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<String>,
    /// Command being executed.
    pub command: String,
    /// Whether the shell is attached or detached.
    pub attachment_mode: TaskShellInfoAttachmentMode,
    /// Whether the task is sync-waited or backgrounded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_mode: Option<TaskExecutionMode>,
    /// Whether this shell can currently be promoted to background mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_promote_to_background: Option<bool>,
    /// Detached log path, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_path: Option<String>,
    /// Process ID, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
}

/// A tracked task, discriminated by task kind.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum TaskInfo {
    /// Background agent task metadata.
    Agent(TaskAgentInfo),
    /// Background shell task metadata.
    Shell(TaskShellInfo),
}

/// Progress details for a background agent task.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskAgentProgress {
    /// Progress discriminator.
    #[serde(rename = "type")]
    pub kind: String,
    /// Recent activity lines derived from the agent's tool execution events.
    pub recent_activity: Vec<TaskProgressLine>,
    /// Most recent intent reported by the agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_intent: Option<String>,
}

/// Progress details for a background shell task.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskShellProgress {
    /// Progress discriminator.
    #[serde(rename = "type")]
    pub kind: String,
    /// Recent stdout or stderr content emitted by the shell.
    pub recent_output: String,
    /// Process ID, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
}

/// Progress details for a tracked task.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum TaskProgress {
    /// Agent-task progress details.
    Agent(TaskAgentProgress),
    /// Shell-task progress details.
    Shell(TaskShellProgress),
}

/// Result for `session.tasks.list`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskList {
    /// Tasks currently tracked by the session.
    pub tasks: Vec<TaskInfo>,
}

/// Result for `session.tasks.cancel`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksCancelResult {
    /// Whether the task was successfully cancelled.
    pub cancelled: bool,
}

/// Result for `session.tasks.getCurrentPromotable`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksGetCurrentPromotableResult {
    /// The first promotable sync-waiting task, if one exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<TaskInfo>,
}

/// Result for `session.tasks.getProgress`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksGetProgressResult {
    /// Progress details for the requested task, or `None` if it is not tracked.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<TaskProgress>,
}

/// Result for `session.tasks.promoteCurrentToBackground`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksPromoteCurrentToBackgroundResult {
    /// The promoted task, if any promotable task was waiting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<TaskInfo>,
}

/// Result for `session.tasks.promoteToBackground`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksPromoteToBackgroundResult {
    /// Whether the task was successfully promoted.
    pub promoted: bool,
}

/// Empty result payload for `session.tasks.refresh`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksRefreshResult {}

/// Result for `session.tasks.remove`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksRemoveResult {
    /// Whether the task was removed from tracking.
    pub removed: bool,
}

/// Result for `session.tasks.sendMessage`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksSendMessageResult {
    /// Whether the message was delivered successfully.
    pub sent: bool,
    /// Delivery failure details, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Result for `session.tasks.startAgent`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksStartAgentResult {
    /// Generated agent ID assigned to the new task.
    pub agent_id: String,
}

/// Empty result payload for `session.tasks.waitForPending`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TasksWaitForPendingResult {}

impl Session {
    /// Access background task management APIs.
    ///
    /// The returned [`SessionTasks`] provides typed bindings for
    /// `session.tasks.*` RPC methods that list, inspect, promote, message,
    /// cancel, and drain background tasks associated with this session.
    pub fn tasks(&self) -> SessionTasks<'_> {
        SessionTasks { session: self }
    }
}

impl SessionTasks<'_> {
    /// Starts a background agent task in the session.
    pub async fn start_agent(
        &self,
        agent_type: &str,
        prompt: &str,
        name: &str,
        description: Option<&str>,
        model: Option<&str>,
    ) -> Result<TasksStartAgentResult> {
        let mut params = serde_json::json!({
            "sessionId": self.session.session_id,
            "agentType": agent_type,
            "prompt": prompt,
            "name": name,
        });
        if let Some(description) = description {
            params["description"] = serde_json::json!(description);
        }
        if let Some(model) = model {
            params["model"] = serde_json::json!(model);
        }
        let result = (self.session.invoke_fn)("session.tasks.startAgent", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Lists background tasks currently tracked by the session.
    pub async fn list(&self) -> Result<TaskList> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result = (self.session.invoke_fn)("session.tasks.list", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Refreshes metadata for detached background shells known to the runtime.
    pub async fn refresh(&self) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        (self.session.invoke_fn)("session.tasks.refresh", Some(params)).await?;
        Ok(())
    }

    /// Waits for all tracked background work and follow-up turns to settle.
    pub async fn wait_for_pending(&self) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        (self.session.invoke_fn)("session.tasks.waitForPending", Some(params)).await?;
        Ok(())
    }

    /// Returns progress information for a tracked background task by ID.
    pub async fn get_progress(&self, id: &str) -> Result<TasksGetProgressResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "id": id,
        });
        let result = (self.session.invoke_fn)("session.tasks.getProgress", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Returns the first sync-waiting task that can currently be backgrounded.
    pub async fn get_current_promotable(&self) -> Result<TasksGetCurrentPromotableResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result =
            (self.session.invoke_fn)("session.tasks.getCurrentPromotable", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Promotes an eligible synchronously awaited task into background mode.
    pub async fn promote_to_background(&self, id: &str) -> Result<TasksPromoteToBackgroundResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "id": id,
        });
        let result =
            (self.session.invoke_fn)("session.tasks.promoteToBackground", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Atomically promotes the first promotable sync-waiting task, if any.
    pub async fn promote_current_to_background(
        &self,
    ) -> Result<TasksPromoteCurrentToBackgroundResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result =
            (self.session.invoke_fn)("session.tasks.promoteCurrentToBackground", Some(params))
                .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Cancels a tracked background task.
    pub async fn cancel(&self, id: &str) -> Result<TasksCancelResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "id": id,
        });
        let result = (self.session.invoke_fn)("session.tasks.cancel", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Removes a completed or cancelled task from session tracking.
    pub async fn remove(&self, id: &str) -> Result<TasksRemoveResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "id": id,
        });
        let result = (self.session.invoke_fn)("session.tasks.remove", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Sends a message to a background agent task.
    pub async fn send_message(
        &self,
        id: &str,
        message: &str,
        from_agent_id: Option<&str>,
    ) -> Result<TasksSendMessageResult> {
        let mut params = serde_json::json!({
            "sessionId": self.session.session_id,
            "id": id,
            "message": message,
        });
        if let Some(from_agent_id) = from_agent_id {
            params["fromAgentId"] = serde_json::json!(from_agent_id);
        }
        let result = (self.session.invoke_fn)("session.tasks.sendMessage", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}
