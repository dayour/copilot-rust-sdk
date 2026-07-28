// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Client-surface MCP config, skills, secrets, and account RPC bindings.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::events::{McpServerSource, McpServerTransport, SkillSource};
use crate::{Client, Result};

/// Accessor for `agentRegistry.*` RPC methods exposed on [`Client`].
pub struct ClientAgentRegistry<'a> {
    client: &'a Client,
}

/// Accessor for `connect` RPC methods exposed on [`Client`].
pub struct ClientConnect<'a> {
    client: &'a Client,
}

/// Accessor for `mcp.config.*` RPC methods exposed on [`Client`].
pub struct ClientMcpConfig<'a> {
    client: &'a Client,
}

/// Accessor for `mcp.*` RPC methods exposed on [`Client`].
pub struct ClientMcp<'a> {
    client: &'a Client,
}

/// Accessor for `secrets.*` RPC methods exposed on [`Client`].
pub struct ClientSecrets<'a> {
    client: &'a Client,
}

/// Accessor for `skills.*` RPC methods exposed on [`Client`].
pub struct ClientSkills<'a> {
    client: &'a Client,
}

/// Accessor for `skills.config.*` RPC methods exposed on [`Client`].
pub struct ClientSkillsConfig<'a> {
    client: &'a Client,
}

/// Accessor for `user.settings.*` RPC methods exposed on [`Client`].
pub struct ClientUserSettings<'a> {
    client: &'a Client,
}

impl Client {
    /// Access managed child-session registry APIs.
    pub fn agent_registry(&self) -> ClientAgentRegistry<'_> {
        ClientAgentRegistry { client: self }
    }

    /// Access the server connection handshake API.
    pub fn connect(&self) -> ClientConnect<'_> {
        ClientConnect { client: self }
    }

    /// Access global MCP server configuration APIs.
    pub fn mcp_config(&self) -> ClientMcpConfig<'_> {
        ClientMcpConfig { client: self }
    }

    /// Access server-surface MCP discovery APIs.
    pub fn mcp(&self) -> ClientMcp<'_> {
        ClientMcp { client: self }
    }

    /// Access secret redaction APIs.
    pub fn secrets(&self) -> ClientSecrets<'_> {
        ClientSecrets { client: self }
    }

    /// Access skill discovery APIs.
    pub fn skills(&self) -> ClientSkills<'_> {
        ClientSkills { client: self }
    }

    /// Access global skill configuration APIs.
    pub fn skills_config(&self) -> ClientSkillsConfig<'_> {
        ClientSkillsConfig { client: self }
    }

    /// Access user settings cache-control APIs.
    pub fn user_settings(&self) -> ClientUserSettings<'_> {
        ClientUserSettings { client: self }
    }
}

impl ClientAgentRegistry<'_> {
    /// Spawn a managed child server through the controller-local spawn delegate.
    pub async fn spawn(
        &self,
        request: AgentRegistrySpawnRequest,
    ) -> Result<AgentRegistrySpawnResult> {
        let params = serde_json::to_value(request)?;
        let result = self
            .client
            .invoke("agentRegistry.spawn", Some(params))
            .await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl ClientConnect<'_> {
    /// Perform the initial SDK handshake with the Copilot CLI server.
    pub async fn connect(&self, request: ConnectRequest) -> Result<ConnectResult> {
        let params = serde_json::to_value(request)?;
        let result = self.client.invoke("connect", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl ClientMcpConfig<'_> {
    /// Add a new MCP server definition to user configuration.
    pub async fn add(&self, request: McpConfigAddRequest) -> Result<()> {
        let params = serde_json::to_value(request)?;
        self.client.invoke("mcp.config.add", Some(params)).await?;
        Ok(())
    }

    /// Disable MCP servers in global configuration for future sessions.
    pub async fn disable(&self, request: McpConfigDisableRequest) -> Result<()> {
        let params = serde_json::to_value(request)?;
        self.client
            .invoke("mcp.config.disable", Some(params))
            .await?;
        Ok(())
    }

    /// Enable MCP servers in global configuration for future sessions.
    pub async fn enable(&self, request: McpConfigEnableRequest) -> Result<()> {
        let params = serde_json::to_value(request)?;
        self.client
            .invoke("mcp.config.enable", Some(params))
            .await?;
        Ok(())
    }

    /// List MCP servers from user configuration.
    pub async fn list(&self) -> Result<McpConfigList> {
        let result = self.client.invoke("mcp.config.list", None).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Drop the in-memory MCP config cache for this runtime process.
    pub async fn reload(&self) -> Result<()> {
        self.client.invoke("mcp.config.reload", None).await?;
        Ok(())
    }

    /// Remove an MCP server definition from user configuration.
    pub async fn remove(&self, request: McpConfigRemoveRequest) -> Result<()> {
        let params = serde_json::to_value(request)?;
        self.client
            .invoke("mcp.config.remove", Some(params))
            .await?;
        Ok(())
    }

    /// Replace an existing MCP server definition in user configuration.
    pub async fn update(&self, request: McpConfigUpdateRequest) -> Result<()> {
        let params = serde_json::to_value(request)?;
        self.client
            .invoke("mcp.config.update", Some(params))
            .await?;
        Ok(())
    }
}

impl ClientMcp<'_> {
    /// Discover MCP servers from user, workspace, plugin, and built-in sources.
    pub async fn discover(&self, request: McpDiscoverRequest) -> Result<McpDiscoverResult> {
        let params = serde_json::to_value(request)?;
        let result = self.client.invoke("mcp.discover", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl ClientSecrets<'_> {
    /// Register secret values for redaction in logs and exported session data.
    pub async fn add_filter_values(
        &self,
        request: SecretsAddFilterValuesRequest,
    ) -> Result<SecretsAddFilterValuesResult> {
        let params = serde_json::to_value(request)?;
        let result = self
            .client
            .invoke("secrets.addFilterValues", Some(params))
            .await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl ClientSkills<'_> {
    /// Discover skills across global and project sources.
    pub async fn discover(&self, request: SkillsDiscoverRequest) -> Result<ServerSkillList> {
        let params = serde_json::to_value(request)?;
        let result = self.client.invoke("skills.discover", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl ClientSkillsConfig<'_> {
    /// Replace the globally disabled skill list.
    pub async fn set_disabled_skills(
        &self,
        request: SkillsConfigSetDisabledSkillsRequest,
    ) -> Result<()> {
        let params = serde_json::to_value(request)?;
        self.client
            .invoke("skills.config.setDisabledSkills", Some(params))
            .await?;
        Ok(())
    }
}

impl ClientUserSettings<'_> {
    /// Drop the in-memory user settings cache for this runtime process.
    pub async fn reload(&self) -> Result<()> {
        self.client.invoke("user.settings.reload", None).await?;
        Ok(())
    }
}

/// Request payload for `agentRegistry.spawn`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRegistrySpawnRequest {
    /// Working directory for the spawned child.
    pub cwd: String,
    /// Optional built-in or custom agent name to run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_name: Option<String>,
    /// Optional model identifier to select for the child session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Optional friendly session name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Permission posture to apply to the child session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<AgentRegistrySpawnPermissionMode>,
    /// Optional first prompt to send after the controller attaches.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_prompt: Option<String>,
}

/// Permission posture for a spawned managed child session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentRegistrySpawnPermissionMode {
    /// Use the standard permission posture.
    Default,
    /// Use allow-all mode when the controller is permitted to do so.
    Yolo,
}

/// Outcome of an `agentRegistry.spawn` request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AgentRegistrySpawnResult {
    /// Managed child server spawned and registered successfully.
    ///
    /// Boxed because this variant is substantially larger than the error
    /// variants; boxing keeps the enum compact.
    Spawned(Box<AgentRegistrySpawnSpawned>),
    /// Child process creation failed before registration.
    SpawnError(AgentRegistrySpawnError),
    /// Child process started but never registered in time.
    RegistryTimeout(AgentRegistrySpawnRegistryTimeout),
    /// Synchronous validation rejected the request.
    ValidationError(AgentRegistrySpawnValidationError),
}

/// Successful `agentRegistry.spawn` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRegistrySpawnSpawned {
    /// Discriminator for the successful result shape.
    pub kind: String,
    /// Full registry entry published by the spawned child.
    pub entry: AgentRegistryLiveTargetEntry,
    /// Whether the spawn delegate already sent the initial prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_prompt_sent: Option<bool>,
    /// Categorized error message when sending the initial prompt failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_prompt_error: Option<String>,
    /// Optional per-spawn log capture outcome.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_capture: Option<AgentRegistryLogCapture>,
}

/// Failed `agentRegistry.spawn` response caused by child-process startup failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRegistrySpawnError {
    /// Discriminator for the failure result shape.
    pub kind: String,
    /// Human-readable failure message.
    pub message: String,
    /// Optional platform error code such as `ENOENT` or `EACCES`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

/// Failed `agentRegistry.spawn` response caused by child registration timeout.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRegistrySpawnRegistryTimeout {
    /// Discriminator for the timeout result shape.
    pub kind: String,
    /// Process identifier of the spawned child that never registered.
    pub child_pid: u64,
    /// Optional per-spawn log capture outcome.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_capture: Option<AgentRegistryLogCapture>,
}

/// Failed `agentRegistry.spawn` response caused by synchronous input validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRegistrySpawnValidationError {
    /// Discriminator for the validation result shape.
    pub kind: String,
    /// Stable reason code describing why validation failed.
    pub reason: AgentRegistrySpawnValidationErrorReason,
    /// Specific request field that failed validation, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<AgentRegistrySpawnValidationErrorField>,
    /// Human-readable validation message suitable for UI display.
    pub message: String,
}

/// Request field that failed spawn validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AgentRegistrySpawnValidationErrorField {
    /// The `cwd` parameter was invalid.
    Cwd,
    /// The `name` parameter was invalid.
    Name,
    /// The `agentName` parameter was invalid.
    AgentName,
    /// The `model` parameter was invalid.
    Model,
    /// The `permissionMode` parameter was invalid.
    PermissionMode,
}

/// Stable reason code for spawn-request validation failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentRegistrySpawnValidationErrorReason {
    /// The supplied working directory does not exist.
    CwdNotFound,
    /// The supplied working directory exists but is not a directory.
    CwdNotDirectory,
    /// The friendly session name failed validation.
    InvalidName,
    /// The requested agent name is unknown.
    UnknownAgent,
    /// The requested model identifier is unknown.
    UnknownModel,
    /// `yolo` permission mode is not currently allowed.
    YoloNotAllowed,
}

/// Live managed-target registry entry returned by `agentRegistry.spawn`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRegistryLiveTargetEntry {
    /// Registry entry schema version.
    pub schema_version: u64,
    /// Process kind for the registered target.
    pub kind: AgentRegistryLiveTargetEntryKind,
    /// Operating-system process identifier.
    pub pid: u64,
    /// Bind host for the target JSON-RPC server.
    pub host: String,
    /// TCP port for the target JSON-RPC server.
    pub port: u64,
    /// Optional connection token required by the target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// Optional foreground session identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Optional human-friendly session name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_name: Option<String>,
    /// Optional working directory for the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Optional Git branch name for the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Optional selected model identifier for the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Optional coarse lifecycle status for the foreground session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<AgentRegistryLiveTargetEntryStatus>,
    /// Optional attention subtype when `status` is `attention`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attention_kind: Option<AgentRegistryLiveTargetEntryAttentionKind>,
    /// Optional monotonic revision number for status updates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_revision: Option<u64>,
    /// Optional terminal-event marker for the most recent turn.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_terminal_event: Option<AgentRegistryLiveTargetEntryLastTerminalEvent>,
    /// ISO 8601 registration timestamp.
    pub started_at: String,
    /// Copilot CLI version that published the entry.
    pub copilot_version: String,
    /// Milliseconds since the watcher last observed the entry.
    pub last_seen_ms: u64,
}

/// Process kind reported by a live registry entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentRegistryLiveTargetEntryKind {
    /// Interactive CLI process exposing a UI server.
    UiServer,
    /// Headless managed child process spawned by a controller.
    ManagedServer,
}

/// Coarse lifecycle state reported by a live registry entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentRegistryLiveTargetEntryStatus {
    /// The session is currently processing a turn.
    Working,
    /// The session is idle and waiting for input.
    Waiting,
    /// The most recent turn completed successfully.
    Done,
    /// The session needs user attention.
    Attention,
}

/// Specific reason a live registry entry requires user attention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRegistryLiveTargetEntryAttentionKind {
    /// The session is blocked on an error.
    Error,
    /// The session is waiting on a permission decision.
    Permission,
    /// The session is waiting for plan approval.
    ExitPlan,
    /// The session is waiting on an elicitation prompt.
    Elicitation,
    /// The session is waiting on free-form user input.
    UserInput,
}

/// How the most recent turn ended for a live registry entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRegistryLiveTargetEntryLastTerminalEvent {
    /// The turn ended cleanly.
    TurnEnd,
    /// The turn was aborted.
    Abort,
}

/// Per-spawn log capture status returned by `agentRegistry.spawn`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRegistryLogCapture {
    /// Whether log capture is enabled for this spawn.
    pub enabled: bool,
    /// Optional absolute path to the captured log file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Optional human-readable log-open failure message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_error: Option<String>,
    /// Optional categorized reason for log-open failure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_error_reason: Option<AgentRegistryLogCaptureOpenErrorReason>,
}

/// Stable reason code for log-capture initialization failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentRegistryLogCaptureOpenErrorReason {
    /// Filesystem permissions prevented opening the log file.
    Permission,
    /// The target filesystem had no remaining capacity.
    DiskFull,
    /// Another uncategorized error occurred.
    Other,
}

/// Request payload for `connect`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectRequest {
    /// Optional connection token presented during the handshake.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

/// Successful response payload for `connect`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResult {
    /// Success marker for the completed handshake.
    pub ok: bool,
    /// Supported server protocol version.
    pub protocol_version: u64,
    /// Copilot CLI package version.
    pub version: String,
}

/// Request payload for `mcp.config.add`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpConfigAddRequest {
    /// Unique configuration key for the MCP server.
    pub name: String,
    /// MCP server configuration to persist.
    pub config: McpServerConfig,
}

/// Request payload for `mcp.config.disable`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpConfigDisableRequest {
    /// MCP server names to add to the disabled list.
    pub names: Vec<String>,
}

/// Request payload for `mcp.config.enable`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpConfigEnableRequest {
    /// MCP server names to remove from the disabled list.
    pub names: Vec<String>,
}

/// Result payload for `mcp.config.list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpConfigList {
    /// User-configured MCP servers keyed by server name.
    pub servers: BTreeMap<String, McpServerConfig>,
}

/// Request payload for `mcp.config.remove`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpConfigRemoveRequest {
    /// MCP server name to delete from user configuration.
    pub name: String,
}

/// Request payload for `mcp.config.update`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpConfigUpdateRequest {
    /// MCP server name to replace in user configuration.
    pub name: String,
    /// Replacement MCP server configuration to persist.
    pub config: McpServerConfig,
}

/// Request payload for `mcp.discover`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpDiscoverRequest {
    /// Optional working directory used to scope discovery.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
}

/// Result payload for `mcp.discover`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpDiscoverResult {
    /// MCP servers discovered from all supported sources.
    pub servers: Vec<DiscoveredMcpServer>,
}

/// Discovered MCP server metadata returned by `mcp.discover`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredMcpServer {
    /// MCP server configuration key.
    pub name: String,
    /// Optional transport type used by the server.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub server_type: Option<McpServerTransport>,
    /// Where the server definition came from.
    pub source: McpServerSource,
    /// Whether the server is currently enabled.
    pub enabled: bool,
}

/// Serialized MCP server configuration written by `mcp.config.*` RPC methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpServerConfig {
    /// Local stdio MCP server launched as a child process.
    Stdio(McpServerConfigStdio),
    /// Remote HTTP or SSE MCP server.
    Http(McpServerConfigHttp),
}

/// Configuration for a remote HTTP or SSE MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfigHttp {
    /// Optional subset of tools to expose from the remote server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    /// Optional remote transport type.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub server_type: Option<McpServerConfigHttpType>,
    /// Whether this entry is the built-in default server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default_server: Option<bool>,
    /// Optional content-filtering mode or per-tool map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_mapping: Option<FilterMapping>,
    /// Optional tool-call timeout in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// Optional OIDC configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oidc: Option<McpServerAuthConfig>,
    /// Optional authentication configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<McpServerAuthConfig>,
    /// Remote server endpoint URL.
    pub url: String,
    /// Optional HTTP headers to include on requests.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, String>>,
    /// Optional pre-registered OAuth client identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_client_id: Option<String>,
    /// Optional marker indicating the OAuth client is public.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_public_client: Option<bool>,
    /// Optional OAuth grant type for remote-server authentication.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_grant_type: Option<McpServerConfigHttpOauthGrantType>,
}

/// Configuration for a local stdio MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerConfigStdio {
    /// Optional subset of tools to expose from the local server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    /// Whether this entry is the built-in default server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default_server: Option<bool>,
    /// Optional content-filtering mode or per-tool map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_mapping: Option<FilterMapping>,
    /// Optional tool-call timeout in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// Optional OIDC configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oidc: Option<McpServerAuthConfig>,
    /// Optional authentication configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth: Option<McpServerAuthConfig>,
    /// Executable command used to launch the stdio server.
    pub command: String,
    /// Optional command-line arguments passed to the server process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    /// Optional working directory used to launch the server process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    /// Optional environment variables passed to the server process.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<BTreeMap<String, String>>,
}

/// Remote transport type for an HTTP-configured MCP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McpServerConfigHttpType {
    /// Streamable HTTP transport.
    Http,
    /// Server-Sent Events transport.
    Sse,
}

/// OAuth grant type for a remote HTTP MCP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpServerConfigHttpOauthGrantType {
    /// Interactive browser-based authorization code flow with PKCE.
    AuthorizationCode,
    /// Headless client credentials flow.
    ClientCredentials,
}

/// Authentication configuration accepted by MCP server definitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpServerAuthConfig {
    /// Boolean opt-in using default authentication settings.
    Enabled(bool),
    /// Object form with optional redirect port and other provider-specific keys.
    RedirectPort(McpServerAuthConfigRedirectPort),
}

/// Object-shaped MCP authentication settings with optional extra properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerAuthConfigRedirectPort {
    /// Fixed local port for the OAuth redirect callback server.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_port: Option<u16>,
    /// Additional provider-specific authentication settings.
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

/// Content-filtering mapping accepted by MCP server definitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterMapping {
    /// Per-tool content-filtering configuration.
    ByTool(BTreeMap<String, ContentFilterMode>),
    /// Single content-filtering mode applied to every tool.
    Mode(ContentFilterMode),
}

/// Content-filtering mode for MCP tool results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentFilterMode {
    /// Leave MCP tool content unchanged.
    None,
    /// Sanitize HTML while preserving Markdown-friendly output.
    Markdown,
    /// Strip characters that can hide directives.
    HiddenCharacters,
}

/// Request payload for `secrets.addFilterValues`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretsAddFilterValuesRequest {
    /// Raw secret values to register for redaction.
    pub values: Vec<String>,
}

/// Result payload for `secrets.addFilterValues`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecretsAddFilterValuesResult {
    /// Whether the secret values were accepted for redaction.
    pub ok: bool,
}

/// Request payload for `skills.config.setDisabledSkills`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsConfigSetDisabledSkillsRequest {
    /// Skill names that should be globally disabled.
    pub disabled_skills: Vec<String>,
}

/// Request payload for `skills.discover`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillsDiscoverRequest {
    /// Optional project roots to scan for project-scoped skills.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_paths: Option<Vec<String>>,
    /// Optional extra directories to scan for skills.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_directories: Option<Vec<String>>,
}

/// Result payload for `skills.discover`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerSkillList {
    /// All discovered skills across all configured sources.
    pub skills: Vec<ServerSkill>,
}

/// Discovered skill metadata returned by `skills.discover`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerSkill {
    /// Unique identifier for the skill.
    pub name: String,
    /// Human-readable summary of what the skill does.
    pub description: String,
    /// Source location type for the skill.
    pub source: SkillSource,
    /// Whether the user can invoke the skill directly.
    pub user_invocable: bool,
    /// Whether the skill is currently enabled.
    pub enabled: bool,
    /// Optional absolute path to the skill file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Optional project path that owns the skill.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_path: Option<String>,
}
