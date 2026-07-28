// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Permission, path, location, folder-trust, and URL RPC bindings.

use crate::{Result, Session};
use serde_json::Value;
use std::collections::BTreeMap;

/// Accessor for session-scoped permission management APIs.
pub struct SessionPermissions<'a> {
    session: &'a Session,
}

/// Accessor for path-scoped permission APIs.
pub struct SessionPermissionPaths<'a> {
    session: &'a Session,
}

/// Accessor for location-scoped permission APIs.
pub struct SessionPermissionLocations<'a> {
    session: &'a Session,
}

/// Accessor for folder-trust APIs.
pub struct SessionPermissionFolderTrust<'a> {
    session: &'a Session,
}

/// Accessor for URL permission APIs.
pub struct SessionPermissionUrls<'a> {
    session: &'a Session,
}

/// Patch of permission policy fields to apply to a session.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsConfigureParams {
    /// Whether tool permission requests should be auto-approved without prompting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approve_all_tool_permission_requests: Option<bool>,
    /// Whether path and URL read permission requests should be auto-approved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approve_all_read_permission_requests: Option<bool>,
    /// Replacement approved and denied permission rules for the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<PermissionRulesSet>,
    /// Replacement path-permission policy for the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paths: Option<PermissionPathsConfig>,
    /// Replacement URL-permission policy for the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub urls: Option<PermissionUrlsConfig>,
    /// Replacement host-supplied content exclusion policies for the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_content_exclusion_policies:
        Option<Vec<PermissionsConfigureAdditionalContentExclusionPolicy>>,
}

/// Result for `session.permissions.configure`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsConfigureResult {
    /// Whether the operation succeeded.
    pub success: bool,
}

/// Approved and denied permission rules.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRulesSet {
    /// Rules that auto-approve matching requests.
    pub approved: Vec<PermissionRule>,
    /// Rules that auto-deny matching requests.
    pub denied: Vec<PermissionRule>,
}

/// A single permission rule.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRule {
    /// The rule kind, such as `Shell` or `GitHubMCP`.
    pub kind: String,
    /// The argument matched against the request, when the rule kind accepts one.
    pub argument: Option<String>,
}

/// Path-permission policy configuration.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionPathsConfig {
    /// Whether all filesystem paths should be allowed without prompting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unrestricted: Option<bool>,
    /// Extra directories to allow in addition to the session working directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_directories: Option<Vec<String>>,
    /// Whether the system temp directory should be included in the allow-list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_temp_directory: Option<bool>,
    /// Workspace root path to expose as the primary working directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_path: Option<String>,
}

/// Snapshot of the session's allow-listed directories and primary working directory.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionPathsList {
    /// All directories currently allowed for tool access.
    pub directories: Vec<String>,
    /// The primary working directory for the session.
    pub primary: String,
}

/// URL-permission policy configuration.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionUrlsConfig {
    /// Whether all URLs should be allowed without prompting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unrestricted: Option<bool>,
    /// Initial list of allowed URL or domain patterns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_allowed: Option<Vec<String>>,
}

/// Host-supplied content exclusion policy.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsConfigureAdditionalContentExclusionPolicy {
    /// Rules that define which paths are excluded.
    pub rules: Vec<PermissionsConfigureAdditionalContentExclusionPolicyRule>,
    /// Opaque last-updated marker supplied by the host.
    pub last_updated_at: Value,
    /// Scope to which this exclusion policy applies.
    pub scope: PermissionsConfigureAdditionalContentExclusionPolicyScope,
    /// Additional host-defined fields preserved during round-tripping.
    #[serde(default, flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Rule within an additional content exclusion policy.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsConfigureAdditionalContentExclusionPolicyRule {
    /// Paths governed by this rule.
    pub paths: Vec<String>,
    /// Optional glob-like matchers; at least one must match for the rule to apply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_any_match: Option<Vec<String>>,
    /// Optional glob-like matchers; none may match for the rule to apply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_none_match: Option<Vec<String>>,
    /// Source metadata for this rule.
    pub source: PermissionsConfigureAdditionalContentExclusionPolicyRuleSource,
    /// Additional host-defined fields preserved during round-tripping.
    #[serde(default, flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Source metadata for an additional content exclusion rule.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsConfigureAdditionalContentExclusionPolicyRuleSource {
    /// Source name.
    pub name: String,
    /// Source type.
    pub r#type: String,
}

/// Scope for an additional content exclusion policy.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionsConfigureAdditionalContentExclusionPolicyScope {
    /// The policy applies only to the current repository.
    Repo,
    /// The policy applies across all repositories.
    All,
}

/// Result for `session.permissions.getAllowAll`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowAllPermissionState {
    /// Whether full allow-all permissions are currently active.
    pub enabled: bool,
}

/// Parameters for `session.permissions.modifyRules`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsModifyRulesParams {
    /// Whether the change applies to session-scoped or persisted location-scoped rules.
    pub scope: PermissionsModifyRulesScope,
    /// Rules to add before removals are processed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add: Option<Vec<PermissionRule>>,
    /// Specific rules to remove from the scope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<Vec<PermissionRule>>,
    /// Whether every rule in the scope should be removed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_all: Option<bool>,
}

/// Scope for permission rule modifications.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionsModifyRulesScope {
    /// Apply the rule change only to the active session.
    Session,
    /// Persist the rule change for the current project location.
    Location,
}

/// Result for `session.permissions.modifyRules`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsModifyRulesResult {
    /// Whether the operation succeeded.
    pub success: bool,
}

/// Result for `session.permissions.notifyPromptShown`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsNotifyPromptShownResult {
    /// Whether the operation succeeded.
    pub success: bool,
}

/// A pending permission request reconstructed from session history.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingPermissionRequest {
    /// Unique identifier for the pending permission request.
    pub request_id: String,
    /// User-facing permission prompt details.
    pub request: Value,
}

/// List of pending permission requests.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingPermissionRequestList {
    /// Pending permission prompts reconstructed from event history.
    pub items: Vec<PendingPermissionRequest>,
}

/// Result for `session.permissions.resetSessionApprovals`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsResetSessionApprovalsResult {
    /// Whether the operation succeeded.
    pub success: bool,
}

/// Source for allow-all telemetry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PermissionsSetAllowAllSource {
    /// Allow-all was enabled from a CLI command-line flag.
    #[serde(rename = "cli_flag")]
    CliFlag,
    /// Allow-all was enabled by a slash command.
    #[serde(rename = "slash_command")]
    SlashCommand,
    /// Allow-all was enabled after the user confirmed autopilot behavior.
    #[serde(rename = "autopilot_confirmation")]
    AutopilotConfirmation,
    /// Allow-all was enabled through an RPC caller.
    #[serde(rename = "rpc")]
    Rpc,
}

/// Result for `session.permissions.setAllowAll`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowAllPermissionSetResult {
    /// Whether the operation succeeded.
    pub success: bool,
    /// The authoritative allow-all state after the mutation.
    pub enabled: bool,
}

/// Source for approve-all telemetry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PermissionsSetApproveAllSource {
    /// Allow-all was enabled from a CLI command-line flag.
    #[serde(rename = "cli_flag")]
    CliFlag,
    /// Allow-all was enabled by a slash command.
    #[serde(rename = "slash_command")]
    SlashCommand,
    /// Allow-all was enabled after the user confirmed autopilot behavior.
    #[serde(rename = "autopilot_confirmation")]
    AutopilotConfirmation,
    /// Allow-all was enabled through an RPC caller.
    #[serde(rename = "rpc")]
    Rpc,
}

/// Result for `session.permissions.setApproveAll`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsSetApproveAllResult {
    /// Whether the operation succeeded.
    pub success: bool,
}

/// Result for `session.permissions.setRequired`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsSetRequiredResult {
    /// Whether the operation succeeded.
    pub success: bool,
}

/// Parameters for adding a location-scoped tool approval.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsLocationsAddToolApprovalParams {
    /// Location key to persist the approval to.
    pub location_key: String,
    /// Tool approval to persist and apply.
    pub approval: PermissionsLocationsAddToolApprovalDetails,
}

/// Tool approval to persist for a location.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum PermissionsLocationsAddToolApprovalDetails {
    /// Approval scoped to specific command identifiers.
    Commands {
        /// Command identifiers covered by this approval.
        #[serde(rename = "commandIdentifiers")]
        command_identifiers: Vec<String>,
    },
    /// Approval covering read-only filesystem operations.
    Read,
    /// Approval covering filesystem write operations.
    Write,
    /// Approval covering an MCP tool.
    Mcp {
        /// MCP server name.
        #[serde(rename = "serverName")]
        server_name: String,
        /// MCP tool name, or `None` to cover every tool on the server.
        #[serde(rename = "toolName")]
        tool_name: Option<String>,
    },
    /// Approval covering MCP sampling requests for a server.
    McpSampling {
        /// MCP server name.
        #[serde(rename = "serverName")]
        server_name: String,
    },
    /// Approval covering writes to long-term memory.
    Memory,
    /// Approval covering a custom tool.
    CustomTool {
        /// Custom tool name.
        #[serde(rename = "toolName")]
        tool_name: String,
    },
    /// Approval covering extension lifecycle operations.
    ExtensionManagement {
        /// Optional operation identifier.
        #[serde(rename = "operation", skip_serializing_if = "Option::is_none")]
        operation: Option<String>,
    },
    /// Approval covering an extension's permission-gated capability access.
    ExtensionPermissionAccess {
        /// Extension name.
        #[serde(rename = "extensionName")]
        extension_name: String,
    },
}

/// Result for `session.permissions.locations.addToolApproval`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsLocationsAddToolApprovalResult {
    /// Whether the operation succeeded.
    pub success: bool,
}

/// Result for `session.permissions.locations.resolve`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionLocationResolveResult {
    /// Location key used in the location-permissions store.
    pub location_key: String,
    /// Whether the location is a git repository or a directory.
    pub location_type: PermissionLocationType,
}

/// Result for `session.permissions.locations.apply`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionLocationApplyResult {
    /// Location key used in the location-permissions store.
    pub location_key: String,
    /// Whether the location is a git repository or a directory.
    pub location_type: PermissionLocationType,
    /// Whether a different location was applied since the previous apply call.
    pub changed: bool,
    /// Number of location-scoped rules added to the live permission service.
    pub applied_rule_count: u64,
    /// Number of persisted allowed directories added to the live path manager.
    pub applied_directory_count: u64,
    /// Location-scoped rules applied to the live permission service.
    pub applied_rules: Vec<PermissionRule>,
}

/// Whether a location permission entry is rooted at a repository or directory.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionLocationType {
    /// The permission location is persisted at the git repository root.
    Repo,
    /// The permission location is persisted at the working directory.
    Dir,
}

/// Result for `session.permissions.paths.add`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsPathsAddResult {
    /// Whether the operation succeeded.
    pub success: bool,
}

/// Result for checking whether a path is within allowed directories.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionPathsAllowedCheckResult {
    /// Whether the path is within the session's allowed directories.
    pub allowed: bool,
}

/// Result for checking whether a path is within the session workspace.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionPathsWorkspaceCheckResult {
    /// Whether the path is within the session workspace directory.
    pub allowed: bool,
}

/// Result for `session.permissions.paths.updatePrimary`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsPathsUpdatePrimaryResult {
    /// Whether the operation succeeded.
    pub success: bool,
}

/// Result for `session.permissions.folderTrust.addTrusted`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsFolderTrustAddTrustedResult {
    /// Whether the operation succeeded.
    pub success: bool,
}

/// Result for `session.permissions.folderTrust.isTrusted`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderTrustCheckResult {
    /// Whether the folder is trusted.
    pub trusted: bool,
}

/// Result for `session.permissions.urls.setUnrestrictedMode`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionsUrlsSetUnrestrictedModeResult {
    /// Whether the operation succeeded.
    pub success: bool,
}

impl Session {
    /// Access permission management APIs.
    pub fn permissions(&self) -> SessionPermissions<'_> {
        SessionPermissions { session: self }
    }
}

impl SessionPermissions<'_> {
    /// Access path-scoped permission APIs.
    pub fn paths(&self) -> SessionPermissionPaths<'_> {
        SessionPermissionPaths {
            session: self.session,
        }
    }

    /// Access location-scoped permission APIs.
    pub fn locations(&self) -> SessionPermissionLocations<'_> {
        SessionPermissionLocations {
            session: self.session,
        }
    }

    /// Access folder-trust APIs.
    pub fn folder_trust(&self) -> SessionPermissionFolderTrust<'_> {
        SessionPermissionFolderTrust {
            session: self.session,
        }
    }

    /// Access URL permission APIs.
    pub fn urls(&self) -> SessionPermissionUrls<'_> {
        SessionPermissionUrls {
            session: self.session,
        }
    }

    /// Replace selected permission policy fields on the session.
    ///
    /// Omitted fields are left unchanged by the runtime.
    pub async fn configure(
        &self,
        configuration: PermissionsConfigureParams,
    ) -> Result<PermissionsConfigureResult> {
        let mut params = serde_json::to_value(&configuration)?;
        if let Some(obj) = params.as_object_mut() {
            obj.insert(
                "sessionId".into(),
                serde_json::json!(self.session.session_id),
            );
        }
        let result =
            (self.session.invoke_fn)("session.permissions.configure", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Return whether full allow-all permissions are active for the session.
    pub async fn get_allow_all(&self) -> Result<AllowAllPermissionState> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result =
            (self.session.invoke_fn)("session.permissions.getAllowAll", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Add or remove session-scoped or location-scoped permission rules.
    pub async fn modify_rules(
        &self,
        changes: PermissionsModifyRulesParams,
    ) -> Result<PermissionsModifyRulesResult> {
        let mut params = serde_json::to_value(&changes)?;
        if let Some(obj) = params.as_object_mut() {
            obj.insert(
                "sessionId".into(),
                serde_json::json!(self.session.session_id),
            );
        }
        let result =
            (self.session.invoke_fn)("session.permissions.modifyRules", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Notify the runtime that a permission prompt UI has been shown to the user.
    pub async fn notify_prompt_shown(
        &self,
        message: &str,
    ) -> Result<PermissionsNotifyPromptShownResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "message": message,
        });
        let result =
            (self.session.invoke_fn)("session.permissions.notifyPromptShown", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Reconstruct pending permission requests from the session event history.
    pub async fn pending_requests(&self) -> Result<PendingPermissionRequestList> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result =
            (self.session.invoke_fn)("session.permissions.pendingRequests", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Clear session-scoped tool permission approvals.
    pub async fn reset_session_approvals(&self) -> Result<PermissionsResetSessionApprovalsResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result =
            (self.session.invoke_fn)("session.permissions.resetSessionApprovals", Some(params))
                .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Enable or disable full allow-all permissions for the session.
    pub async fn set_allow_all(
        &self,
        enabled: bool,
        source: Option<PermissionsSetAllowAllSource>,
    ) -> Result<AllowAllPermissionSetResult> {
        let mut params = serde_json::json!({
            "sessionId": self.session.session_id,
            "enabled": enabled,
        });
        if let Some(source) = source {
            params["source"] = serde_json::to_value(source)?;
        }
        let result =
            (self.session.invoke_fn)("session.permissions.setAllowAll", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Enable or disable automatic approval of tool permission requests.
    pub async fn set_approve_all(
        &self,
        enabled: bool,
        source: Option<PermissionsSetApproveAllSource>,
    ) -> Result<PermissionsSetApproveAllResult> {
        let mut params = serde_json::json!({
            "sessionId": self.session.session_id,
            "enabled": enabled,
        });
        if let Some(source) = source {
            params["source"] = serde_json::to_value(source)?;
        }
        let result =
            (self.session.invoke_fn)("session.permissions.setApproveAll", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Set whether permission prompts should be bridged into session events.
    pub async fn set_required(&self, required: bool) -> Result<PermissionsSetRequiredResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "required": required,
        });
        let result =
            (self.session.invoke_fn)("session.permissions.setRequired", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionPermissionPaths<'_> {
    /// Add a directory to the session allow-list.
    pub async fn add(&self, path: &str) -> Result<PermissionsPathsAddResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "path": path,
        });
        let result =
            (self.session.invoke_fn)("session.permissions.paths.add", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Check whether a path falls within one of the session's allowed directories.
    pub async fn is_path_within_allowed_directories(
        &self,
        path: &str,
    ) -> Result<PermissionPathsAllowedCheckResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "path": path,
        });
        let result = (self.session.invoke_fn)(
            "session.permissions.paths.isPathWithinAllowedDirectories",
            Some(params),
        )
        .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Check whether a path falls within the session workspace directory.
    pub async fn is_path_within_workspace(
        &self,
        path: &str,
    ) -> Result<PermissionPathsWorkspaceCheckResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "path": path,
        });
        let result = (self.session.invoke_fn)(
            "session.permissions.paths.isPathWithinWorkspace",
            Some(params),
        )
        .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Return the session's allowed directories and primary working directory.
    pub async fn list(&self) -> Result<PermissionPathsList> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result =
            (self.session.invoke_fn)("session.permissions.paths.list", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Update the session's primary working directory used by the permission policy.
    pub async fn update_primary(&self, path: &str) -> Result<PermissionsPathsUpdatePrimaryResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "path": path,
        });
        let result =
            (self.session.invoke_fn)("session.permissions.paths.updatePrimary", Some(params))
                .await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionPermissionLocations<'_> {
    /// Persist a tool approval for a permission location and apply it to the live session.
    pub async fn add_tool_approval(
        &self,
        location_key: &str,
        approval: PermissionsLocationsAddToolApprovalDetails,
    ) -> Result<PermissionsLocationsAddToolApprovalResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "locationKey": location_key,
            "approval": approval,
        });
        let result = (self.session.invoke_fn)(
            "session.permissions.locations.addToolApproval",
            Some(params),
        )
        .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Apply persisted location-scoped permissions for a working directory to the session.
    pub async fn apply(&self, working_directory: &str) -> Result<PermissionLocationApplyResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "workingDirectory": working_directory,
        });
        let result =
            (self.session.invoke_fn)("session.permissions.locations.apply", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Resolve the location-permissions key and type for a working directory.
    pub async fn resolve(
        &self,
        working_directory: &str,
    ) -> Result<PermissionLocationResolveResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "workingDirectory": working_directory,
        });
        let result =
            (self.session.invoke_fn)("session.permissions.locations.resolve", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionPermissionFolderTrust<'_> {
    /// Add a folder to the user's trusted folders list.
    pub async fn add_trusted(&self, path: &str) -> Result<PermissionsFolderTrustAddTrustedResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "path": path,
        });
        let result =
            (self.session.invoke_fn)("session.permissions.folderTrust.addTrusted", Some(params))
                .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Report whether a folder is trusted according to the user's folder-trust state.
    pub async fn is_trusted(&self, path: &str) -> Result<FolderTrustCheckResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "path": path,
        });
        let result =
            (self.session.invoke_fn)("session.permissions.folderTrust.isTrusted", Some(params))
                .await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionPermissionUrls<'_> {
    /// Toggle the runtime URL-permission policy between restricted and unrestricted modes.
    pub async fn set_unrestricted_mode(
        &self,
        enabled: bool,
    ) -> Result<PermissionsUrlsSetUnrestrictedModeResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "enabled": enabled,
        });
        let result =
            (self.session.invoke_fn)("session.permissions.urls.setUnrestrictedMode", Some(params))
                .await?;
        Ok(serde_json::from_value(result)?)
    }
}
