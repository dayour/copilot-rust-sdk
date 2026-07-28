// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Canvas, interactive UI, and workspace file RPC bindings.

use crate::{
    canvas::{CanvasActionDeclaration, CanvasJsonSchema, OpenCanvasInstance},
    Result, Session,
};
use serde_json::Value;

/// Accessor for session-scoped canvas APIs.
pub struct SessionCanvas<'a> {
    session: &'a Session,
}

/// Accessor for canvas-action APIs on an open canvas instance.
pub struct SessionCanvasAction<'a> {
    session: &'a Session,
}

/// Canvas action that the agent or host can invoke on an open instance.
pub type CanvasAction = CanvasActionDeclaration;

/// Canvas available in the current session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredCanvas {
    /// Human-readable canvas name.
    pub display_name: String,
    /// Short, single-sentence description shown to the agent in canvas catalogs.
    pub description: String,
    /// JSON Schema for canvas open input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<CanvasJsonSchema>,
    /// Actions the agent or host may invoke on an open instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<CanvasAction>>,
    /// Owning provider identifier.
    pub extension_id: String,
    /// Owning extension display name, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_name: Option<String>,
    /// Provider-local canvas identifier.
    pub canvas_id: String,
}

/// Declared canvases available in this session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasList {
    /// Declared canvases available in this session.
    pub canvases: Vec<DiscoveredCanvas>,
}

/// Live snapshot of open canvas instances for the session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasListOpenResult {
    /// Currently open canvas instances.
    pub open_canvases: Vec<OpenCanvasInstance>,
}

/// Result returned from `session.canvas.action.invoke`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasActionInvokeResult {
    /// Provider-supplied action result, when the action returned one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
}

/// Result indicating whether a pending UI request was resolved by this call.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiHandlePendingResult {
    /// True if the request was still pending and was resolved by this call.
    pub success: bool,
}

/// Opaque sampling result payload for `session.ui.handlePendingSampling`.
pub type UiHandlePendingSamplingResponse = Value;

/// User response payload for `session.ui.handlePendingUserInput`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiUserInputResponse {
    /// The user's answer text.
    pub answer: String,
    /// Whether the user typed a freeform response instead of selecting a choice.
    pub was_freeform: bool,
}

/// Result returned when registering a direct auto-mode-switch handler.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiRegisterDirectAutoModeSwitchHandlerResult {
    /// Opaque handle representing the registration.
    pub handle: String,
}

/// Result returned when unregistering a direct auto-mode-switch handler.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiUnregisterDirectAutoModeSwitchHandlerResult {
    /// Whether the handle was active and successfully decremented.
    pub unregistered: bool,
}

/// Diff mode requested by the client.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkspaceDiffMode {
    /// Return staged, unstaged, and untracked working tree changes.
    Unstaged,
    /// Return changes compared with the default branch.
    Branch,
}

/// Type of change represented by a file diff.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkspaceDiffFileChangeType {
    /// The file was added.
    Added,
    /// The file was modified.
    Modified,
    /// The file was deleted.
    Deleted,
    /// The file was renamed.
    Renamed,
}

/// A single changed file and its unified diff.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceDiffFileChange {
    /// Path to the changed file, relative to the workspace root.
    pub path: String,
    /// Unified diff content for the file.
    pub diff: String,
    /// Type of change represented by this file diff.
    pub change_type: WorkspaceDiffFileChangeType,
    /// Original file path for renamed files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_path: Option<String>,
    /// Whether the diff content was omitted because it exceeded the size limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_truncated: Option<bool>,
}

/// Workspace diff result for the requested mode.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceDiffResult {
    /// Diff mode requested by the client.
    pub requested_mode: WorkspaceDiffMode,
    /// Effective mode used for the returned changes.
    pub mode: WorkspaceDiffMode,
    /// Changed files and their unified diffs.
    pub changes: Vec<WorkspaceDiffFileChange>,
    /// Default branch used for a branch diff, when branch mode was requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_branch: Option<String>,
    /// Whether a requested branch diff fell back to unstaged changes.
    pub is_fallback: bool,
}

/// A single workspace checkpoint.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacesCheckpoint {
    /// Checkpoint number assigned by the workspace manager.
    pub number: u64,
    /// Human-readable checkpoint title.
    pub title: String,
    /// Filename of the checkpoint within the workspace checkpoints directory.
    pub filename: String,
}

/// Repository host type for a workspace.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WorkspacesWorkspaceDetailsHostType {
    /// Workspace repository is hosted on GitHub.
    Github,
    /// Workspace repository is hosted on Azure DevOps.
    Ado,
}

/// Metadata for the current session workspace.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacesWorkspaceDetails {
    /// Stable workspace identifier.
    pub id: String,
    /// Current working directory tracked for the workspace.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Git repository root, when known.
    #[serde(rename = "git_root", skip_serializing_if = "Option::is_none")]
    pub git_root: Option<String>,
    /// Repository slug in `owner/name` form, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// Repository host type, when known.
    #[serde(rename = "host_type", skip_serializing_if = "Option::is_none")]
    pub host_type: Option<WorkspacesWorkspaceDetailsHostType>,
    /// Active Git branch, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// User-visible workspace name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Name of the client that created the workspace, when available.
    #[serde(rename = "client_name", skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,
    /// Whether the workspace name was explicitly chosen by the user.
    #[serde(rename = "user_named", skip_serializing_if = "Option::is_none")]
    pub user_named: Option<bool>,
    /// Number of workspace summaries recorded for this workspace.
    #[serde(rename = "summary_count", skip_serializing_if = "Option::is_none")]
    pub summary_count: Option<u64>,
    /// Workspace creation timestamp.
    #[serde(rename = "created_at", skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    /// Workspace last-update timestamp.
    #[serde(rename = "updated_at", skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    /// Whether the workspace is currently remote-steerable.
    #[serde(rename = "remote_steerable", skip_serializing_if = "Option::is_none")]
    pub remote_steerable: Option<bool>,
    /// Associated multi-client task identifier, when present.
    #[serde(rename = "mc_task_id", skip_serializing_if = "Option::is_none")]
    pub mc_task_id: Option<String>,
    /// Associated multi-client session identifier, when present.
    #[serde(rename = "mc_session_id", skip_serializing_if = "Option::is_none")]
    pub mc_session_id: Option<String>,
    /// Last known multi-client event identifier, when present.
    #[serde(rename = "mc_last_event_id", skip_serializing_if = "Option::is_none")]
    pub mc_last_event_id: Option<String>,
    /// Whether chronicle sync dismissal has been recorded.
    #[serde(
        rename = "chronicle_sync_dismissed",
        skip_serializing_if = "Option::is_none"
    )]
    pub chronicle_sync_dismissed: Option<bool>,
}

/// Current workspace metadata for the session.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacesGetWorkspaceResult {
    /// Current workspace metadata, or `None` when no workspace is available.
    pub workspace: Option<WorkspacesWorkspaceDetails>,
    /// Absolute filesystem path to the workspace directory, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Workspace checkpoints in chronological order.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacesListCheckpointsResult {
    /// Workspace checkpoints in chronological order.
    pub checkpoints: Vec<WorkspacesCheckpoint>,
}

/// Content of a workspace checkpoint.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacesReadCheckpointResult {
    /// Checkpoint content as a UTF-8 string, or `None` when unavailable.
    pub content: Option<String>,
}

/// Descriptor for a saved large-paste file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacesSavedLargePasteFile {
    /// Absolute filesystem path to the saved paste file.
    pub file_path: String,
    /// Filename within the workspace files directory.
    pub filename: String,
    /// Size of the saved file in bytes.
    pub size_bytes: u64,
}

/// Result returned from saving a large paste into the workspace.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspacesSaveLargePasteResult {
    /// Descriptor for the saved paste file, or `None` when the workspace is unavailable.
    pub saved: Option<WorkspacesSavedLargePasteFile>,
}

impl Session {
    /// Access canvas APIs for this session.
    pub fn canvas(&self) -> SessionCanvas<'_> {
        SessionCanvas { session: self }
    }

    /// Resolve a pending `sampling.requested` event with an optional sampling result payload.
    pub async fn ui_handle_pending_sampling(
        &self,
        request_id: &str,
        response: Option<UiHandlePendingSamplingResponse>,
    ) -> Result<UiHandlePendingResult> {
        let mut params = serde_json::json!({
            "sessionId": self.session_id,
            "requestId": request_id,
        });
        if let Some(response) = response {
            params["response"] = response;
        }
        let result = (self.invoke_fn)("session.ui.handlePendingSampling", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Resolve a pending `user_input.requested` event with the user's response.
    pub async fn ui_handle_pending_user_input(
        &self,
        request_id: &str,
        response: UiUserInputResponse,
    ) -> Result<UiHandlePendingResult> {
        let params = serde_json::json!({
            "sessionId": self.session_id,
            "requestId": request_id,
            "response": response,
        });
        let result = (self.invoke_fn)("session.ui.handlePendingUserInput", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Register an in-process handler for `auto_mode_switch.requested` events.
    pub async fn ui_register_direct_auto_mode_switch_handler(
        &self,
    ) -> Result<UiRegisterDirectAutoModeSwitchHandlerResult> {
        let params = serde_json::json!({
            "sessionId": self.session_id,
        });
        let result = (self.invoke_fn)(
            "session.ui.registerDirectAutoModeSwitchHandler",
            Some(params),
        )
        .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Unregister a previously-registered direct auto-mode-switch handler.
    pub async fn ui_unregister_direct_auto_mode_switch_handler(
        &self,
        handle: &str,
    ) -> Result<UiUnregisterDirectAutoModeSwitchHandlerResult> {
        let params = serde_json::json!({
            "sessionId": self.session_id,
            "handle": handle,
        });
        let result = (self.invoke_fn)(
            "session.ui.unregisterDirectAutoModeSwitchHandler",
            Some(params),
        )
        .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Get current workspace metadata for the session.
    pub async fn workspace_get_workspace(&self) -> Result<WorkspacesGetWorkspaceResult> {
        let params = serde_json::json!({
            "sessionId": self.session_id,
        });
        let result = (self.invoke_fn)("session.workspaces.getWorkspace", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// List workspace checkpoints in chronological order.
    pub async fn workspace_list_checkpoints(&self) -> Result<WorkspacesListCheckpointsResult> {
        let params = serde_json::json!({
            "sessionId": self.session_id,
        });
        let result = (self.invoke_fn)("session.workspaces.listCheckpoints", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Read the content of a workspace checkpoint by number.
    pub async fn workspace_read_checkpoint(
        &self,
        number: u64,
    ) -> Result<WorkspacesReadCheckpointResult> {
        let params = serde_json::json!({
            "sessionId": self.session_id,
            "number": number,
        });
        let result = (self.invoke_fn)("session.workspaces.readCheckpoint", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Save pasted content as a UTF-8 file in the session workspace.
    pub async fn workspace_save_large_paste(
        &self,
        content: &str,
    ) -> Result<WorkspacesSaveLargePasteResult> {
        let params = serde_json::json!({
            "sessionId": self.session_id,
            "content": content,
        });
        let result = (self.invoke_fn)("session.workspaces.saveLargePaste", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Compute a diff for the session workspace.
    pub async fn workspace_diff(&self, mode: WorkspaceDiffMode) -> Result<WorkspaceDiffResult> {
        let params = serde_json::json!({
            "sessionId": self.session_id,
            "mode": mode,
        });
        let result = (self.invoke_fn)("session.workspaces.diff", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionCanvas<'_> {
    /// Access canvas-action APIs for this session.
    pub fn action(&self) -> SessionCanvasAction<'_> {
        SessionCanvasAction {
            session: self.session,
        }
    }

    /// List canvases declared for the session.
    pub async fn list(&self) -> Result<CanvasList> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result = (self.session.invoke_fn)("session.canvas.list", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// List open canvas instances for the live session.
    pub async fn list_open(&self) -> Result<CanvasListOpenResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result = (self.session.invoke_fn)("session.canvas.listOpen", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Open or focus a canvas instance.
    pub async fn open(
        &self,
        canvas_id: &str,
        instance_id: &str,
        extension_id: Option<&str>,
        input: Option<Value>,
    ) -> Result<OpenCanvasInstance> {
        let mut params = serde_json::json!({
            "sessionId": self.session.session_id,
            "canvasId": canvas_id,
            "instanceId": instance_id,
        });
        if let Some(extension_id) = extension_id {
            params["extensionId"] = serde_json::json!(extension_id);
        }
        if let Some(input) = input {
            params["input"] = input;
        }
        let result = (self.session.invoke_fn)("session.canvas.open", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Close an open canvas instance.
    pub async fn close(&self, instance_id: &str) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "instanceId": instance_id,
        });
        (self.session.invoke_fn)("session.canvas.close", Some(params)).await?;
        Ok(())
    }
}

impl SessionCanvasAction<'_> {
    /// Invoke an action on an open canvas instance.
    pub async fn invoke(
        &self,
        instance_id: &str,
        action_name: &str,
        input: Option<Value>,
    ) -> Result<CanvasActionInvokeResult> {
        let mut params = serde_json::json!({
            "sessionId": self.session.session_id,
            "instanceId": instance_id,
            "actionName": action_name,
        });
        if let Some(input) = input {
            params["input"] = input;
        }
        let result = (self.session.invoke_fn)("session.canvas.action.invoke", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}
