// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Session-scoped MCP server, app, and OAuth RPC bindings.

use crate::events::{McpServerSource, McpServerStatus};
use crate::{Result, Session};
use serde_json::Value;
use std::collections::BTreeMap;

/// Namespace accessor for `session.mcp.*` RPC methods.
pub struct SessionMcp<'a> {
    session: &'a Session,
}

/// Namespace accessor for `session.mcp.apps.*` RPC methods.
pub struct SessionMcpApps<'a> {
    session: &'a Session,
}

/// Namespace accessor for `session.mcp.oauth.*` RPC methods.
pub struct SessionMcpOauth<'a> {
    session: &'a Session,
}

/// Request payload for `session.mcp.apps.callTool`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsCallToolRequest {
    /// Target session identifier.
    pub session_id: String,
    /// MCP server hosting the tool.
    pub server_name: String,
    /// MCP tool name.
    pub tool_name: String,
    /// Tool arguments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<Value>,
    /// Server whose `ui://` view issued the request.
    pub origin_server_name: String,
}

/// Standard MCP `CallToolResult`.
pub type McpAppsCallToolResult = Value;

/// Request payload for `session.mcp.apps.diagnose`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsDiagnoseRequest {
    /// Target session identifier.
    pub session_id: String,
    /// MCP server to probe.
    pub server_name: String,
}

/// Request payload for `session.mcp.apps.getHostContext`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsGetHostContextRequest {
    /// Target session identifier.
    pub session_id: String,
}

/// Request payload for `session.mcp.apps.listTools`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsListToolsRequest {
    /// Target session identifier.
    pub session_id: String,
    /// MCP server hosting the app.
    pub server_name: String,
    /// Server whose `ui://` view issued the request.
    pub origin_server_name: String,
}

/// Request payload for `session.mcp.apps.readResource`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsReadResourceRequest {
    /// Target session identifier.
    pub session_id: String,
    /// Name of the MCP server hosting the resource.
    pub server_name: String,
    /// Resource URI to fetch.
    pub uri: String,
}

/// Request payload for `session.mcp.apps.setHostContext`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsSetHostContextRequest {
    /// Target session identifier.
    pub session_id: String,
    /// Host context advertised to MCP App guests.
    pub context: McpAppsSetHostContextDetails,
}

/// Request payload for `session.mcp.cancelSamplingExecution`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpCancelSamplingExecutionParams {
    /// Target session identifier.
    pub session_id: String,
    /// Sampling request identifier to cancel.
    pub request_id: String,
}

/// Request payload for `session.mcp.disable`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpDisableRequest {
    /// Target session identifier.
    pub session_id: String,
    /// Name of the MCP server to disable.
    pub server_name: String,
}

/// Request payload for `session.mcp.enable`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpEnableRequest {
    /// Target session identifier.
    pub session_id: String,
    /// Name of the MCP server to enable.
    pub server_name: String,
}

/// Request payload for `session.mcp.executeSampling`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpExecuteSamplingParams {
    /// Target session identifier.
    pub session_id: String,
    /// Caller-provided unique sampling execution identifier.
    pub request_id: String,
    /// Name of the MCP server that initiated the sampling request.
    pub server_name: String,
    /// Original MCP JSON-RPC request identifier.
    pub mcp_request_id: Value,
    /// Raw MCP `sampling/createMessage` request payload.
    pub request: McpExecuteSamplingRequest,
}

/// Request payload for `session.mcp.list`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpListRequest {
    /// Target session identifier.
    pub session_id: String,
}

/// Request payload for `session.mcp.oauth.login`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpOauthLoginRequest {
    /// Target session identifier.
    pub session_id: String,
    /// Name of the remote MCP server to authenticate.
    pub server_name: String,
    /// Whether to force a brand-new authorization flow.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_reauth: Option<bool>,
    /// Optional OAuth client display name override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,
    /// Optional callback success-page message override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub callback_success_message: Option<String>,
}

/// Request payload for `session.mcp.reload`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpReloadRequest {
    /// Target session identifier.
    pub session_id: String,
}

/// Request payload for `session.mcp.removeGitHub`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpRemoveGitHubRequest {
    /// Target session identifier.
    pub session_id: String,
}

/// Request payload for `session.mcp.setEnvValueMode`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpSetEnvValueModeParams {
    /// Target session identifier.
    pub session_id: String,
    /// How environment-variable values should be resolved for MCP servers.
    pub mode: McpSetEnvValueModeDetails,
}

/// Capability-negotiation state used by `session.mcp.apps.diagnose`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsDiagnoseCapability {
    /// Whether the session exposes the `mcp-apps` capability.
    pub session_has_mcp_apps: bool,
    /// Whether the MCP Apps feature flag is enabled.
    pub feature_flag_enabled: bool,
    /// Whether the runtime advertises the MCP Apps extension capability.
    pub advertised: bool,
}

/// Server-specific diagnostic details returned by `session.mcp.apps.diagnose`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsDiagnoseServer {
    /// Whether the named server is currently connected.
    pub connected: bool,
    /// Total tool count returned by the server.
    pub tool_count: f64,
    /// Count of tools that include `_meta.ui`.
    pub tools_with_ui_meta: f64,
    /// Up to five example tool names that advertise UI metadata.
    pub sample_tool_names: Vec<String>,
}

/// Result payload for `session.mcp.apps.diagnose`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsDiagnoseResult {
    /// Capability-negotiation snapshot for the current session.
    pub capability: McpAppsDiagnoseCapability,
    /// Diagnostic snapshot for the requested server.
    pub server: McpAppsDiagnoseServer,
}

/// Result payload for `session.mcp.apps.getHostContext`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsHostContext {
    /// Current host context advertised to MCP App guests.
    pub context: McpAppsHostContextDetails,
}

/// UI display mode currently used by the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum McpAppsHostContextDetailsDisplayMode {
    /// Render inline within the host conversation surface.
    #[serde(rename = "inline")]
    Inline,
    /// Render as a fullscreen overlay.
    #[serde(rename = "fullscreen")]
    Fullscreen,
    /// Render as a picture-in-picture floating panel.
    #[serde(rename = "pip")]
    Pip,
}

/// Display mode supported by the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum McpAppsHostContextDetailsAvailableDisplayMode {
    /// Render inline within the host conversation surface.
    #[serde(rename = "inline")]
    Inline,
    /// Render as a fullscreen overlay.
    #[serde(rename = "fullscreen")]
    Fullscreen,
    /// Render as a picture-in-picture floating panel.
    #[serde(rename = "pip")]
    Pip,
}

/// Platform type used for responsive MCP App layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum McpAppsHostContextDetailsPlatform {
    /// Host runs in a web browser.
    #[serde(rename = "web")]
    Web,
    /// Host runs as a desktop application.
    #[serde(rename = "desktop")]
    Desktop,
    /// Host runs on a mobile device.
    #[serde(rename = "mobile")]
    Mobile,
}

/// Theme preference advertised to MCP App guests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum McpAppsHostContextDetailsTheme {
    /// Light UI theme.
    #[serde(rename = "light")]
    Light,
    /// Dark UI theme.
    #[serde(rename = "dark")]
    Dark,
}

/// Host metadata advertised to an embedded MCP App guest.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsHostContextDetails {
    /// UI theme preference per SEP-1865.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<McpAppsHostContextDetailsTheme>,
    /// BCP-47 locale, such as `en-US`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// IANA timezone, such as `America/New_York`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
    /// Current display mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_mode: Option<McpAppsHostContextDetailsDisplayMode>,
    /// Display modes supported by the host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_display_modes: Option<Vec<McpAppsHostContextDetailsAvailableDisplayMode>>,
    /// Platform type used for responsive design.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<McpAppsHostContextDetailsPlatform>,
    /// Host application identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// Additional host-defined context properties.
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Result payload for `session.mcp.apps.listTools`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsListToolsResult {
    /// App-callable tools returned by the server.
    pub tools: Vec<Value>,
}

/// Result payload for `session.mcp.apps.readResource`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsReadResourceResult {
    /// Resource payloads returned by the server.
    pub contents: Vec<McpAppsResourceContent>,
}

/// One content item returned by `session.mcp.apps.readResource`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsResourceContent {
    /// Resource URI, typically `ui://...`.
    pub uri: String,
    /// MIME type of the content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Text content, such as HTML.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Base64-encoded binary content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
    /// Resource-level metadata such as CSP or permissions.
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<Value>,
}

/// UI display mode to advertise to guest MCP Apps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum McpAppsSetHostContextDetailsDisplayMode {
    /// Render inline within the host conversation surface.
    #[serde(rename = "inline")]
    Inline,
    /// Render as a fullscreen overlay.
    #[serde(rename = "fullscreen")]
    Fullscreen,
    /// Render as a picture-in-picture floating panel.
    #[serde(rename = "pip")]
    Pip,
}

/// Supported UI display mode to advertise to guest MCP Apps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum McpAppsSetHostContextDetailsAvailableDisplayMode {
    /// Render inline within the host conversation surface.
    #[serde(rename = "inline")]
    Inline,
    /// Render as a fullscreen overlay.
    #[serde(rename = "fullscreen")]
    Fullscreen,
    /// Render as a picture-in-picture floating panel.
    #[serde(rename = "pip")]
    Pip,
}

/// Platform type to advertise to guest MCP Apps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum McpAppsSetHostContextDetailsPlatform {
    /// Host runs in a web browser.
    #[serde(rename = "web")]
    Web,
    /// Host runs as a desktop application.
    #[serde(rename = "desktop")]
    Desktop,
    /// Host runs on a mobile device.
    #[serde(rename = "mobile")]
    Mobile,
}

/// Theme preference to advertise to guest MCP Apps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum McpAppsSetHostContextDetailsTheme {
    /// Light UI theme.
    #[serde(rename = "light")]
    Light,
    /// Dark UI theme.
    #[serde(rename = "dark")]
    Dark,
}

/// Host context payload sent to `session.mcp.apps.setHostContext`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpAppsSetHostContextDetails {
    /// UI theme preference per SEP-1865.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<McpAppsSetHostContextDetailsTheme>,
    /// BCP-47 locale, such as `en-US`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// IANA timezone, such as `America/New_York`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
    /// Current display mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_mode: Option<McpAppsSetHostContextDetailsDisplayMode>,
    /// Display modes supported by the host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_display_modes: Option<Vec<McpAppsSetHostContextDetailsAvailableDisplayMode>>,
    /// Platform type used for responsive design.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<McpAppsSetHostContextDetailsPlatform>,
    /// Host application identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<String>,
    /// Additional host-defined context properties.
    #[serde(default, flatten, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Result payload for `session.mcp.cancelSamplingExecution`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpCancelSamplingExecutionResult {
    /// Whether an in-flight sampling execution was found and cancelled.
    pub cancelled: bool,
}

/// Raw MCP `sampling/createMessage` request payload.
pub type McpExecuteSamplingRequest = Value;

/// MCP `sampling/createMessage` result payload.
pub type McpExecuteSamplingResult = Value;

/// Result payload for `session.mcp.oauth.login`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpOauthLoginResult {
    /// URL to open in a browser to continue OAuth, when interactive auth is needed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_url: Option<String>,
}

/// Result payload for `session.mcp.removeGitHub`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpRemoveGitHubResult {
    /// Whether the auto-managed `github` MCP server was removed.
    pub removed: bool,
}

/// Outcome of an MCP sampling execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum McpSamplingExecutionAction {
    /// The sampling inference completed successfully.
    #[serde(rename = "success")]
    Success,
    /// The sampling inference failed or was rejected.
    #[serde(rename = "failure")]
    Failure,
    /// The sampling inference was cancelled.
    #[serde(rename = "cancelled")]
    Cancelled,
}

/// Result payload for `session.mcp.executeSampling`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpSamplingExecutionResult {
    /// Outcome of the sampling execution.
    pub action: McpSamplingExecutionAction,
    /// Successful MCP sampling result payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<McpExecuteSamplingResult>,
    /// Error description when the execution failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// One session-configured MCP server.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    /// Server name, matching the MCP configuration key.
    pub name: String,
    /// Current connection status.
    pub status: McpServerStatus,
    /// Configuration source for this server, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<McpServerSource>,
    /// Connection error message, when the server failed to connect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Result payload for `session.mcp.list`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpServerList {
    /// MCP servers configured for the session.
    pub servers: Vec<McpServer>,
}

/// Mode controlling how MCP server environment values are resolved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum McpSetEnvValueModeDetails {
    /// Treat provided values as literal strings.
    #[serde(rename = "direct")]
    Direct,
    /// Treat provided values as host-side references that must be resolved.
    #[serde(rename = "indirect")]
    Indirect,
}

/// Result payload for `session.mcp.setEnvValueMode`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpSetEnvValueModeResult {
    /// Mode recorded on the session after the update.
    pub mode: McpSetEnvValueModeDetails,
}

impl Session {
    /// Access session-scoped MCP server APIs.
    ///
    /// The returned [`SessionMcp`] exposes typed bindings for `session.mcp.*`
    /// RPC methods, including server lifecycle, sampling, OAuth, and MCP App
    /// helpers.
    pub fn mcp(&self) -> SessionMcp<'_> {
        SessionMcp { session: self }
    }
}

impl SessionMcp<'_> {
    /// Access MCP App APIs.
    pub fn apps(&self) -> SessionMcpApps<'_> {
        SessionMcpApps {
            session: self.session,
        }
    }

    /// Access MCP OAuth APIs.
    pub fn oauth(&self) -> SessionMcpOauth<'_> {
        SessionMcpOauth {
            session: self.session,
        }
    }

    /// Lists MCP servers configured for the session and their connection status.
    pub async fn list(&self) -> Result<McpServerList> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result = (self.session.invoke_fn)("session.mcp.list", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Enables an MCP server for the current session.
    pub async fn enable(&self, server_name: &str) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "serverName": server_name,
        });
        (self.session.invoke_fn)("session.mcp.enable", Some(params)).await?;
        Ok(())
    }

    /// Disables an MCP server for the current session.
    pub async fn disable(&self, server_name: &str) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "serverName": server_name,
        });
        (self.session.invoke_fn)("session.mcp.disable", Some(params)).await?;
        Ok(())
    }

    /// Reloads MCP server connections for the session.
    pub async fn reload(&self) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        (self.session.invoke_fn)("session.mcp.reload", Some(params)).await?;
        Ok(())
    }

    /// Removes the auto-managed `github` MCP server when present.
    pub async fn remove_github(&self) -> Result<McpRemoveGitHubResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result = (self.session.invoke_fn)("session.mcp.removeGitHub", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Sets how environment-variable values supplied to MCP servers are resolved.
    pub async fn set_env_value_mode(
        &self,
        mode: McpSetEnvValueModeDetails,
    ) -> Result<McpSetEnvValueModeResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "mode": mode,
        });
        let result = (self.session.invoke_fn)("session.mcp.setEnvValueMode", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Runs an MCP sampling inference on behalf of an MCP server.
    pub async fn execute_sampling(
        &self,
        request_id: &str,
        server_name: &str,
        mcp_request_id: Value,
        request: McpExecuteSamplingRequest,
    ) -> Result<McpSamplingExecutionResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "requestId": request_id,
            "serverName": server_name,
            "mcpRequestId": mcp_request_id,
            "request": request,
        });
        let result = (self.session.invoke_fn)("session.mcp.executeSampling", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Cancels an in-flight MCP sampling execution by request ID.
    pub async fn cancel_sampling_execution(
        &self,
        request_id: &str,
    ) -> Result<McpCancelSamplingExecutionResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "requestId": request_id,
        });
        let result =
            (self.session.invoke_fn)("session.mcp.cancelSamplingExecution", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionMcpApps<'_> {
    /// Lists tools that an MCP App view is allowed to invoke.
    pub async fn list_tools(
        &self,
        server_name: &str,
        origin_server_name: &str,
    ) -> Result<McpAppsListToolsResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "serverName": server_name,
            "originServerName": origin_server_name,
        });
        let result = (self.session.invoke_fn)("session.mcp.apps.listTools", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Calls an MCP tool from an MCP App view.
    pub async fn call_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: Option<Value>,
        origin_server_name: &str,
    ) -> Result<McpAppsCallToolResult> {
        let mut params = serde_json::json!({
            "sessionId": self.session.session_id,
            "serverName": server_name,
            "toolName": tool_name,
            "originServerName": origin_server_name,
        });
        if let Some(arguments) = arguments {
            params["arguments"] = arguments;
        }
        let result = (self.session.invoke_fn)("session.mcp.apps.callTool", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Diagnoses MCP Apps wiring for a specific MCP server.
    pub async fn diagnose(&self, server_name: &str) -> Result<McpAppsDiagnoseResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "serverName": server_name,
        });
        let result = (self.session.invoke_fn)("session.mcp.apps.diagnose", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Reads the current host context advertised to MCP App guests.
    pub async fn get_host_context(&self) -> Result<McpAppsHostContext> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result =
            (self.session.invoke_fn)("session.mcp.apps.getHostContext", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Fetches an MCP resource from a connected server.
    pub async fn read_resource(
        &self,
        server_name: &str,
        uri: &str,
    ) -> Result<McpAppsReadResourceResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "serverName": server_name,
            "uri": uri,
        });
        let result =
            (self.session.invoke_fn)("session.mcp.apps.readResource", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Replaces the host context returned to MCP App guests during initialization.
    pub async fn set_host_context(&self, context: McpAppsSetHostContextDetails) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "context": context,
        });
        (self.session.invoke_fn)("session.mcp.apps.setHostContext", Some(params)).await?;
        Ok(())
    }
}

impl SessionMcpOauth<'_> {
    /// Starts OAuth authentication for a remote MCP server.
    pub async fn login(
        &self,
        server_name: &str,
        force_reauth: Option<bool>,
        client_name: Option<&str>,
        callback_success_message: Option<&str>,
    ) -> Result<McpOauthLoginResult> {
        let mut params = serde_json::json!({
            "sessionId": self.session.session_id,
            "serverName": server_name,
        });
        if let Some(force_reauth) = force_reauth {
            params["forceReauth"] = serde_json::json!(force_reauth);
        }
        if let Some(client_name) = client_name {
            params["clientName"] = serde_json::json!(client_name);
        }
        if let Some(callback_success_message) = callback_success_message {
            params["callbackSuccessMessage"] = serde_json::json!(callback_success_message);
        }
        let result = (self.session.invoke_fn)("session.mcp.oauth.login", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}
