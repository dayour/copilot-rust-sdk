// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Server-surface multi-session management RPC bindings.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{Client, Result};

/// Accessor for server-surface `sessions.*` RPC methods.
///
/// Acquired via [`Client::sessions`].
pub struct ClientSessions<'a> {
    client: &'a Client,
}

impl Client {
    /// Access multi-session management APIs.
    pub fn sessions(&self) -> ClientSessions<'_> {
        ClientSessions { client: self }
    }
}

impl ClientSessions<'_> {
    /// Close, deactivate, and delete a set of sessions from disk.
    pub async fn bulk_delete(
        &self,
        request: SessionsBulkDeleteRequest,
    ) -> Result<SessionBulkDeleteResult> {
        let params = serde_json::to_value(request)?;
        let result = self
            .client
            .invoke("sessions.bulkDelete", Some(params))
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Return the subset of supplied session IDs that are currently in use.
    pub async fn check_in_use(
        &self,
        request: SessionsCheckInUseRequest,
    ) -> Result<SessionsCheckInUseResult> {
        let params = serde_json::to_value(request)?;
        let result = self
            .client
            .invoke("sessions.checkInUse", Some(params))
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Close a session, flush its pending events, and release its in-use lock.
    pub async fn close(&self, request: SessionsCloseRequest) -> Result<()> {
        let params = serde_json::to_value(request)?;
        self.client.invoke("sessions.close", Some(params)).await?;
        Ok(())
    }

    /// Connect to an existing remote session and expose it through the SDK.
    pub async fn connect(
        &self,
        request: ConnectRemoteSessionParams,
    ) -> Result<RemoteSessionConnectionResult> {
        let params = serde_json::to_value(request)?;
        let result = self.client.invoke("sessions.connect", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Backfill missing summary and context fields on session metadata records.
    pub async fn enrich_metadata(
        &self,
        request: SessionsEnrichMetadataRequest,
    ) -> Result<SessionEnrichMetadataResult> {
        let params = serde_json::to_value(request)?;
        let result = self
            .client
            .invoke("sessions.enrichMetadata", Some(params))
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Resolve a UUID prefix to a unique session ID when exactly one match exists.
    pub async fn find_by_prefix(
        &self,
        request: SessionsFindByPrefixRequest,
    ) -> Result<SessionsFindByPrefixResult> {
        let params = serde_json::to_value(request)?;
        let result = self
            .client
            .invoke("sessions.findByPrefix", Some(params))
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Find the local session bound to a GitHub task ID, if one exists.
    pub async fn find_by_task_id(
        &self,
        request: SessionsFindByTaskIdRequest,
    ) -> Result<SessionsFindByTaskIdResult> {
        let params = serde_json::to_value(request)?;
        let result = self
            .client
            .invoke("sessions.findByTaskId", Some(params))
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Create a new session by forking persisted history from an existing session.
    pub async fn fork(&self, request: SessionsForkRequest) -> Result<SessionsForkResult> {
        let params = serde_json::to_value(request)?;
        let result = self.client.invoke("sessions.fork", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Compute the absolute path to a session's persisted `events.jsonl` file.
    pub async fn get_event_file_path(
        &self,
        request: SessionsGetEventFilePathRequest,
    ) -> Result<SessionsGetEventFilePathResult> {
        let params = serde_json::to_value(request)?;
        let result = self
            .client
            .invoke("sessions.getEventFilePath", Some(params))
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Return the most-relevant prior session for a working-directory context.
    pub async fn get_last_for_context(
        &self,
        request: SessionsGetLastForContextRequest,
    ) -> Result<SessionsGetLastForContextResult> {
        let params = serde_json::to_value(request)?;
        let result = self
            .client
            .invoke("sessions.getLastForContext", Some(params))
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Return a session's persisted remote-steerable flag, if one was recorded.
    pub async fn get_persisted_remote_steerable(
        &self,
        request: SessionsGetPersistedRemoteSteerableRequest,
    ) -> Result<SessionsGetPersistedRemoteSteerableResult> {
        let params = serde_json::to_value(request)?;
        let result = self
            .client
            .invoke("sessions.getPersistedRemoteSteerable", Some(params))
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Return the on-disk byte size of each session workspace directory.
    pub async fn get_sizes(&self) -> Result<SessionSizes> {
        let result = self.client.invoke("sessions.getSizes", None).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// List persisted sessions, optionally filtered by working-directory context.
    pub async fn list(&self, request: Option<SessionsListRequest>) -> Result<SessionList> {
        let result = if let Some(request) = request {
            let params = serde_json::to_value(request)?;
            self.client.invoke("sessions.list", Some(params)).await?
        } else {
            self.client.invoke("sessions.list", None).await?
        };
        Ok(serde_json::from_value(result)?)
    }

    /// Load previously deferred repo-level hooks for the active session.
    pub async fn load_deferred_repo_hooks(
        &self,
        request: SessionsLoadDeferredRepoHooksRequest,
    ) -> Result<SessionLoadDeferredRepoHooksResult> {
        let params = serde_json::to_value(request)?;
        let result = self
            .client
            .invoke("sessions.loadDeferredRepoHooks", Some(params))
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Delete or simulate deletion of old persisted sessions.
    pub async fn prune_old(&self, request: SessionsPruneOldRequest) -> Result<SessionPruneResult> {
        let params = serde_json::to_value(request)?;
        let result = self
            .client
            .invoke("sessions.pruneOld", Some(params))
            .await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Release the in-use lock held by this process for a session.
    pub async fn release_lock(&self, request: SessionsReleaseLockRequest) -> Result<()> {
        let params = serde_json::to_value(request)?;
        self.client
            .invoke("sessions.releaseLock", Some(params))
            .await?;
        Ok(())
    }

    /// Reload user, plugin, and optionally repo hooks for the active session.
    pub async fn reload_plugin_hooks(
        &self,
        request: SessionsReloadPluginHooksRequest,
    ) -> Result<()> {
        let params = serde_json::to_value(request)?;
        self.client
            .invoke("sessions.reloadPluginHooks", Some(params))
            .await?;
        Ok(())
    }

    /// Flush a session's pending events to disk.
    pub async fn save(&self, request: SessionsSaveRequest) -> Result<()> {
        let params = serde_json::to_value(request)?;
        self.client.invoke("sessions.save", Some(params)).await?;
        Ok(())
    }

    /// Replace the manager-wide additional plugin set.
    pub async fn set_additional_plugins(
        &self,
        request: SessionsSetAdditionalPluginsRequest,
    ) -> Result<()> {
        let params = serde_json::to_value(request)?;
        self.client
            .invoke("sessions.setAdditionalPlugins", Some(params))
            .await?;
        Ok(())
    }
}

/// Parameters for `sessions.connect`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectRemoteSessionParams {
    /// Session ID to connect to.
    pub session_id: String,
}

/// Metadata for a connected remote session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedRemoteSessionMetadata {
    /// SDK session ID for the connected remote session.
    pub session_id: String,
    /// Optional friendly session name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional session summary.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Session start time as an ISO 8601 string.
    pub start_time: String,
    /// Last session update time as an ISO 8601 string.
    pub modified_time: String,
    /// Repository associated with the connected remote session.
    pub repository: ConnectedRemoteSessionMetadataRepository,
    /// Pull request number associated with the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pull_request_number: Option<u64>,
    /// Original remote resource identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    /// Neutral SDK discriminator for the connected remote session kind.
    pub kind: ConnectedRemoteSessionMetadataKind,
    /// Remote session staleness deadline as an ISO 8601 string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stale_at: Option<String>,
    /// Remote session state returned by the backing service.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

/// Neutral SDK discriminator for the connected remote session kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectedRemoteSessionMetadataKind {
    /// Remote CLI session.
    #[serde(rename = "remote-session")]
    RemoteSession,
    /// GitHub Copilot coding agent session.
    #[serde(rename = "coding-agent")]
    CodingAgent,
}

/// Repository associated with the connected remote session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectedRemoteSessionMetadataRepository {
    /// Repository owner or organization login.
    pub owner: String,
    /// Repository name.
    pub name: String,
    /// Branch associated with the remote session.
    pub branch: String,
}

/// Plugin metadata for manager-wide additional plugins.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledPlugin {
    /// Plugin name.
    pub name: String,
    /// Marketplace the plugin came from, or an empty string for direct repo installs.
    pub marketplace: String,
    /// Version installed, if available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Installation timestamp.
    #[serde(rename = "installed_at")]
    pub installed_at: String,
    /// Whether the plugin is currently enabled.
    pub enabled: bool,
    /// Path where the plugin is cached locally.
    #[serde(rename = "cache_path", skip_serializing_if = "Option::is_none")]
    pub cache_path: Option<String>,
    /// Source for direct repo installs when `marketplace` is empty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Value>,
}

/// Result returned by `sessions.connect`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSessionConnectionResult {
    /// SDK session ID for the connected remote session.
    pub session_id: String,
    /// Metadata for the connected remote session.
    pub metadata: ConnectedRemoteSessionMetadata,
}

/// Result returned by `sessions.bulkDelete`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionBulkDeleteResult {
    /// Map of session ID to bytes freed by removing the session workspace directory.
    pub freed_bytes: BTreeMap<String, u64>,
}

/// Session working-directory context metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionContext {
    /// Most recent working directory for this session.
    pub cwd: String,
    /// Git repository root, if the working directory was inside a Git repository.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_root: Option<String>,
    /// Repository slug in `owner/name` form, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// Repository host type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_type: Option<SessionContextHostType>,
    /// Active Git branch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

/// Repository host type for a session context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionContextHostType {
    /// Session repository is hosted on GitHub.
    #[serde(rename = "github")]
    Github,
    /// Session repository is hosted on Azure DevOps.
    #[serde(rename = "ado")]
    Ado,
}

/// Result returned by `sessions.enrichMetadata`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionEnrichMetadataResult {
    /// Enriched session metadata records.
    pub sessions: Vec<SessionMetadata>,
}

/// Result returned by `sessions.list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionList {
    /// Sessions ordered most-recently-modified first.
    pub sessions: Vec<SessionMetadata>,
}

/// Optional filter applied by `sessions.list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionListFilter {
    /// Match sessions whose `context.cwd` equals this value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Match sessions whose `context.gitRoot` equals this value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_root: Option<String>,
    /// Match sessions whose `context.repository` equals this value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// Match sessions whose `context.branch` equals this value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

/// Result returned by `sessions.loadDeferredRepoHooks`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionLoadDeferredRepoHooksResult {
    /// Repo-level startup prompts queued from repository hook configs.
    pub startup_prompts: Vec<String>,
    /// Total hook command count loaded for the session by this call.
    pub hook_count: u64,
}

/// Persisted metadata for a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMetadata {
    /// Stable session identifier.
    pub session_id: String,
    /// Session creation time as an ISO 8601 timestamp.
    pub start_time: String,
    /// Last-modified time of the session's persisted state as ISO 8601.
    pub modified_time: String,
    /// Short summary of the session, when one has been derived.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// Optional human-friendly name set via `/rename`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Runtime client name that created or last resumed this session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,
    /// Whether the session is remote.
    pub is_remote: bool,
    /// Whether the session is a detached maintenance session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_detached: Option<bool>,
    /// Most recent working-directory context for the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<SessionContext>,
    /// GitHub task ID bound to this local session, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mc_task_id: Option<String>,
}

/// Result returned by `sessions.pruneOld`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionPruneResult {
    /// Session IDs that were deleted.
    pub deleted: Vec<String>,
    /// Session IDs that would be deleted in dry-run mode.
    pub candidates: Vec<String>,
    /// Session IDs that were skipped.
    pub skipped: Vec<String>,
    /// Total bytes freed or projected to be freed.
    pub freed_bytes: u64,
    /// Whether the operation ran in dry-run mode.
    pub dry_run: bool,
}

/// Result returned by `sessions.getSizes`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSizes {
    /// Map of session ID to workspace-directory size in bytes.
    pub sizes: BTreeMap<String, u64>,
}

/// Parameters for `sessions.bulkDelete`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsBulkDeleteRequest {
    /// Session IDs to close, deactivate, and delete from disk.
    pub session_ids: Vec<String>,
}

/// Parameters for `sessions.checkInUse`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCheckInUseRequest {
    /// Session IDs to test for live in-use locks.
    pub session_ids: Vec<String>,
}

/// Result returned by `sessions.checkInUse`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCheckInUseResult {
    /// Session IDs from the input set that are currently held by another process.
    pub in_use: Vec<String>,
}

/// Parameters for `sessions.close`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsCloseRequest {
    /// Session ID to close.
    pub session_id: String,
}

/// Parameters for `sessions.enrichMetadata`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsEnrichMetadataRequest {
    /// Session metadata records to enrich.
    pub sessions: Vec<SessionMetadata>,
}

/// Parameters for `sessions.findByPrefix`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsFindByPrefixRequest {
    /// UUID prefix to resolve to a unique session ID.
    pub prefix: String,
}

/// Result returned by `sessions.findByPrefix`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsFindByPrefixResult {
    /// Unique session ID matching the prefix, when one exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// Parameters for `sessions.findByTaskId`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsFindByTaskIdRequest {
    /// GitHub task ID to look up.
    pub task_id: String,
}

/// Result returned by `sessions.findByTaskId`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsFindByTaskIdResult {
    /// Local session ID bound to the task, when one exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// Parameters for `sessions.fork`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsForkRequest {
    /// Source session ID to fork from.
    pub session_id: String,
    /// Optional event ID boundary to fork up to, exclusive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_event_id: Option<String>,
    /// Optional friendly name for the forked session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Result returned by `sessions.fork`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsForkResult {
    /// The new forked session ID.
    pub session_id: String,
    /// Friendly name assigned to the forked session, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Parameters for `sessions.getEventFilePath`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsGetEventFilePathRequest {
    /// Session ID whose event-log file path to compute.
    pub session_id: String,
}

/// Result returned by `sessions.getEventFilePath`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsGetEventFilePathResult {
    /// Absolute path to the session's `events.jsonl` file.
    pub file_path: String,
}

/// Parameters for `sessions.getLastForContext`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsGetLastForContextRequest {
    /// Optional working-directory context used to score session relevance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<SessionContext>,
}

/// Result returned by `sessions.getLastForContext`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsGetLastForContextResult {
    /// Most-relevant session ID for the supplied context, when one exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// Parameters for `sessions.getPersistedRemoteSteerable`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsGetPersistedRemoteSteerableRequest {
    /// Session ID whose persisted remote-steerable flag to read.
    pub session_id: String,
}

/// Result returned by `sessions.getPersistedRemoteSteerable`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsGetPersistedRemoteSteerableResult {
    /// Persisted remote-steerable flag, when one has been recorded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_steerable: Option<bool>,
}

/// Parameters for `sessions.list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsListRequest {
    /// Number of newest sessions that should load full metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_limit: Option<u64>,
    /// Optional filter applied to the returned sessions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<SessionListFilter>,
    /// Whether detached maintenance sessions should be included.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_detached: Option<bool>,
}

/// Parameters for `sessions.loadDeferredRepoHooks`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsLoadDeferredRepoHooksRequest {
    /// Active session ID whose deferred repository hooks should be loaded.
    pub session_id: String,
}

/// Parameters for `sessions.pruneOld`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsPruneOldRequest {
    /// Delete sessions whose `modifiedTime` is at least this many days old.
    pub older_than_days: u64,
    /// Whether the prune should be reported without performing deletions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
    /// Whether named sessions are eligible for pruning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_named: Option<bool>,
    /// Session IDs that should never be considered for pruning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_session_ids: Option<Vec<String>>,
}

/// Parameters for `sessions.releaseLock`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsReleaseLockRequest {
    /// Session ID whose in-use lock should be released.
    pub session_id: String,
}

/// Parameters for `sessions.reloadPluginHooks`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsReloadPluginHooksRequest {
    /// Active session ID to reload hooks for.
    pub session_id: String,
    /// Whether repository hooks should be deferred until folder trust.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub defer_repo_hooks: Option<bool>,
}

/// Parameters for `sessions.save`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsSaveRequest {
    /// Session ID whose pending events should be flushed to disk.
    pub session_id: String,
}

/// Parameters for `sessions.setAdditionalPlugins`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionsSetAdditionalPluginsRequest {
    /// Manager-wide additional plugins to register.
    pub plugins: Vec<InstalledPlugin>,
}
