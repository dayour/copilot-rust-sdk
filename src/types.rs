// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Core types for the Copilot SDK.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::error::CopilotError;

fn is_false(value: &bool) -> bool {
    !*value
}

// =============================================================================
// Protocol Version
// =============================================================================

/// Maximum protocol version this SDK supports.
/// This must match the version expected by the copilot-agent-runtime server.
pub const SDK_PROTOCOL_VERSION: u32 = 3;

/// Minimum protocol version this SDK can communicate with.
/// Servers reporting a version below this are rejected.
pub const MIN_PROTOCOL_VERSION: u32 = 2;

// =============================================================================
// Enums
// =============================================================================

/// Connection state of the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error,
}

/// System message mode for session configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SystemMessageMode {
    Append,
    Replace,
    Customize,
}

/// Attachment type for user messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AttachmentType {
    File,
    Directory,
    Selection,
    Blob,
    GithubReference,
}

/// Log level for the CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogLevel {
    None,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
    All,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::None => write!(f, "none"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
            LogLevel::All => write!(f, "all"),
        }
    }
}

// =============================================================================
// Tool Types
// =============================================================================

/// Binary result from a tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolBinaryResult {
    pub data: String,
    pub mime_type: String,
    #[serde(rename = "type")]
    pub result_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Icon theme variant for an external resource link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExternalToolTextResultForLlmContentResourceLinkIconTheme {
    /// Icon intended for light themes.
    Light,
    /// Icon intended for dark themes.
    Dark,
}

/// Icon image metadata for an external resource link.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalToolTextResultForLlmContentResourceLinkIcon {
    /// URL or path to the icon image.
    pub src: String,
    /// MIME type of the icon image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Available icon sizes (for example `16x16` or `32x32`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sizes: Option<Vec<String>>,
    /// Theme variant this icon is intended for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<ExternalToolTextResultForLlmContentResourceLinkIconTheme>,
}

/// Embedded text resource contents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedTextResourceContents {
    /// URI identifying the resource.
    pub uri: String,
    /// MIME type of the text content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Text content of the resource.
    pub text: String,
}

/// Embedded binary resource contents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedBlobResourceContents {
    /// URI identifying the resource.
    pub uri: String,
    /// MIME type of the blob content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Base64-encoded binary content of the resource.
    pub blob: String,
}

/// Embedded resource contents, either inline text or inline binary data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExternalToolTextResultForLlmContentResourceDetails {
    /// Embedded text resource contents.
    Text(EmbeddedTextResourceContents),
    /// Embedded binary resource contents.
    Blob(EmbeddedBlobResourceContents),
}

/// A tool-result content block for LLM-visible output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExternalToolTextResultForLlmContent {
    /// Plain text content block.
    Text {
        /// The text content.
        text: String,
    },
    /// Terminal or shell output content block.
    Terminal {
        /// Terminal or shell output text.
        text: String,
        /// Process exit code, if the command has completed.
        #[serde(rename = "exitCode", skip_serializing_if = "Option::is_none")]
        exit_code: Option<i64>,
        /// Working directory where the command was executed.
        #[serde(skip_serializing_if = "Option::is_none")]
        cwd: Option<String>,
    },
    /// Base64-encoded image content block.
    Image {
        /// Base64-encoded image data.
        data: String,
        /// MIME type of the image.
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    /// Base64-encoded audio content block.
    Audio {
        /// Base64-encoded audio data.
        data: String,
        /// MIME type of the audio.
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    /// External resource-link content block.
    ResourceLink {
        /// Icons associated with this resource.
        #[serde(skip_serializing_if = "Option::is_none")]
        icons: Option<Vec<ExternalToolTextResultForLlmContentResourceLinkIcon>>,
        /// Resource name identifier.
        name: String,
        /// Human-readable display title for the resource.
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        /// URI identifying the resource.
        uri: String,
        /// Human-readable description of the resource.
        #[serde(skip_serializing_if = "Option::is_none")]
        description: Option<String>,
        /// MIME type of the resource content.
        #[serde(rename = "mimeType", skip_serializing_if = "Option::is_none")]
        mime_type: Option<String>,
        /// Size of the resource in bytes.
        #[serde(skip_serializing_if = "Option::is_none")]
        size: Option<u64>,
    },
    /// Embedded resource content block.
    Resource {
        /// The embedded resource contents.
        resource: ExternalToolTextResultForLlmContentResourceDetails,
    },
}

/// Result object returned from tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResultObject {
    pub text_result_for_llm: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binary_results_for_llm: Option<Vec<ToolBinaryResult>>,
    #[serde(default = "default_result_type")]
    pub result_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_log: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_telemetry: Option<HashMap<String, serde_json::Value>>,
}

fn default_result_type() -> String {
    "success".to_string()
}

impl ToolResultObject {
    /// Create a success result with text.
    pub fn text(result: impl Into<String>) -> Self {
        Self {
            text_result_for_llm: result.into(),
            binary_results_for_llm: None,
            result_type: "success".to_string(),
            error: None,
            session_log: None,
            tool_telemetry: None,
        }
    }

    /// Create an error result.
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            text_result_for_llm: String::new(),
            binary_results_for_llm: None,
            result_type: "error".to_string(),
            error: Some(message.into()),
            session_log: None,
            tool_telemetry: None,
        }
    }
}

/// Convenient alias for tool results.
pub type ToolResult = ToolResultObject;

/// Information about a tool invocation from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInvocation {
    pub session_id: String,
    pub tool_call_id: String,
    pub tool_name: String,
    #[serde(default)]
    pub arguments: Option<serde_json::Value>,
}

impl ToolInvocation {
    /// Get an argument by name, deserializing to the specified type.
    pub fn arg<T: serde::de::DeserializeOwned>(&self, name: &str) -> crate::Result<T> {
        let args = self
            .arguments
            .as_ref()
            .ok_or_else(|| crate::CopilotError::ToolError("No arguments provided".into()))?;

        let value = args
            .get(name)
            .ok_or_else(|| crate::CopilotError::ToolError(format!("Missing argument: {}", name)))?;

        serde_json::from_value(value.clone()).map_err(|e| {
            crate::CopilotError::ToolError(format!("Invalid argument '{}': {}", name, e))
        })
    }
}

// =============================================================================
// Permission Types
// =============================================================================

/// Permission request from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRequest {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(flatten)]
    pub extension_data: HashMap<String, serde_json::Value>,
}

/// Result of a permission request (response to CLI).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRequestResult {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<Vec<serde_json::Value>>,
}

impl PermissionRequestResult {
    /// Create an approved permission result.
    pub fn approved() -> Self {
        Self {
            kind: "approved".to_string(),
            rules: None,
        }
    }

    /// Create a denied permission result.
    pub fn denied() -> Self {
        Self {
            kind: "denied-no-approval-rule-and-could-not-request-from-user".to_string(),
            rules: None,
        }
    }

    /// Returns true if the permission was approved.
    pub fn is_approved(&self) -> bool {
        self.kind == "approved"
    }

    /// Returns true if the permission was denied.
    pub fn is_denied(&self) -> bool {
        self.kind.starts_with("denied")
    }

    /// Approve this single request without creating a persistent rule.
    /// Wire kind: `approve-once`.
    pub fn approve_once() -> Self {
        Self {
            kind: "approve-once".to_string(),
            rules: None,
        }
    }

    /// Reject this request. Wire kind: `reject`.
    pub fn reject() -> Self {
        Self {
            kind: "reject".to_string(),
            rules: None,
        }
    }

    /// Decline to decide, deferring to the host's default handling.
    /// Wire kind: `no-result`.
    pub fn no_result() -> Self {
        Self {
            kind: "no-result".to_string(),
            rules: None,
        }
    }
}

/// Permission handler that approves every request once (never persisting a
/// rule). Mirrors the Node.js `approveAll` helper. Convenient for trusted,
/// non-interactive automation.
pub fn approve_all(_request: &PermissionRequest) -> PermissionRequestResult {
    PermissionRequestResult::approve_once()
}

/// Default permission handler used by `join_session`: returns `no-result`,
/// deferring the decision to the host session that owns the permission prompt.
/// Mirrors the Node.js `defaultJoinSessionPermissionHandler`.
pub fn default_join_session_permission_handler(
    _request: &PermissionRequest,
) -> PermissionRequestResult {
    PermissionRequestResult::no_result()
}

// =============================================================================
// Configuration Types
// =============================================================================

/// Known system message section identifiers for the "customize" mode. Each
/// corresponds to a distinct part of the system prompt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SystemMessageSection {
    /// Agent identity preamble and mode statement.
    Identity,
    /// Response style, conciseness rules, output formatting preferences.
    Tone,
    /// Tool usage patterns, parallel calling, batching guidelines.
    ToolEfficiency,
    /// CWD, OS, git root, directory listing, available tools.
    EnvironmentContext,
    /// Coding rules, linting/testing, ecosystem tools, style.
    CodeChangeRules,
    /// Tips, behavioral best practices, behavioral guidelines.
    Guidelines,
    /// Environment limitations, prohibited actions, security policies.
    Safety,
    /// Per-tool usage instructions.
    ToolInstructions,
    /// Repository and organization custom instructions.
    CustomInstructions,
    /// Runtime-provided context and instructions.
    RuntimeInstructions,
    /// End-of-prompt instructions: parallel tool calling, persistence, task completion.
    LastInstructions,
}

impl SystemMessageSection {
    /// All known sections, in system-prompt order.
    pub const ALL: &'static [SystemMessageSection] = &[
        SystemMessageSection::Identity,
        SystemMessageSection::Tone,
        SystemMessageSection::ToolEfficiency,
        SystemMessageSection::EnvironmentContext,
        SystemMessageSection::CodeChangeRules,
        SystemMessageSection::Guidelines,
        SystemMessageSection::Safety,
        SystemMessageSection::ToolInstructions,
        SystemMessageSection::CustomInstructions,
        SystemMessageSection::RuntimeInstructions,
        SystemMessageSection::LastInstructions,
    ];

    /// The wire section id (e.g. `tool_efficiency`).
    pub fn id(self) -> &'static str {
        match self {
            SystemMessageSection::Identity => "identity",
            SystemMessageSection::Tone => "tone",
            SystemMessageSection::ToolEfficiency => "tool_efficiency",
            SystemMessageSection::EnvironmentContext => "environment_context",
            SystemMessageSection::CodeChangeRules => "code_change_rules",
            SystemMessageSection::Guidelines => "guidelines",
            SystemMessageSection::Safety => "safety",
            SystemMessageSection::ToolInstructions => "tool_instructions",
            SystemMessageSection::CustomInstructions => "custom_instructions",
            SystemMessageSection::RuntimeInstructions => "runtime_instructions",
            SystemMessageSection::LastInstructions => "last_instructions",
        }
    }

    /// Human-readable description of the section, for documentation and tooling.
    pub fn description(self) -> &'static str {
        match self {
            SystemMessageSection::Identity => "Agent identity preamble and mode statement",
            SystemMessageSection::Tone => {
                "Response style, conciseness rules, output formatting preferences"
            }
            SystemMessageSection::ToolEfficiency => {
                "Tool usage patterns, parallel calling, batching guidelines"
            }
            SystemMessageSection::EnvironmentContext => {
                "CWD, OS, git root, directory listing, available tools"
            }
            SystemMessageSection::CodeChangeRules => {
                "Coding rules, linting/testing, ecosystem tools, style"
            }
            SystemMessageSection::Guidelines => {
                "Tips, behavioral best practices, behavioral guidelines"
            }
            SystemMessageSection::Safety => {
                "Environment limitations, prohibited actions, security policies"
            }
            SystemMessageSection::ToolInstructions => "Per-tool usage instructions",
            SystemMessageSection::CustomInstructions => {
                "Repository and organization custom instructions"
            }
            SystemMessageSection::RuntimeInstructions => {
                "Runtime-provided context and instructions (e.g. system notifications, memories, workspace context, mode-specific instructions, content-exclusion policy)"
            }
            SystemMessageSection::LastInstructions => {
                "End-of-prompt instructions: parallel tool calling, persistence, task completion"
            }
        }
    }
}

/// Returns section metadata (id + description) for all system message sections.
pub fn system_message_sections() -> Vec<(SystemMessageSection, &'static str)> {
    SystemMessageSection::ALL
        .iter()
        .map(|s| (*s, s.description()))
        .collect()
}

/// The operation applied to a single system message section in "customize" mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SectionOverrideAction {
    /// Replace section content entirely.
    Replace,
    /// Remove the section.
    Remove,
    /// Append to existing section content.
    Append,
    /// Prepend to existing section content.
    Prepend,
    /// Run a client-side callback over the section's rendered content.
    ///
    /// The runtime sends the section back to the client via a
    /// `systemMessage.transform` request; the SDK invokes the callback stored
    /// on [`SectionOverride::transform`] and returns the transformed text.
    Transform,
}

/// Boxed future returned by a [`SectionTransformFn`].
pub type SectionTransformFuture =
    std::pin::Pin<Box<dyn std::future::Future<Output = String> + Send>>;

/// Callback that rewrites a system message section's rendered content.
pub type SectionTransformFn = Arc<dyn Fn(String) -> SectionTransformFuture + Send + Sync>;

/// Override operation for a single system message section.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SectionOverride {
    /// The operation to perform on this section.
    pub action: SectionOverrideAction,
    /// Content for the override. Optional for all actions; ignored for `remove`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Callback invoked when `action` is [`SectionOverrideAction::Transform`].
    ///
    /// Never serialized: only the `"transform"` action marker crosses the wire,
    /// and the runtime calls back into the client to apply it.
    #[serde(skip)]
    pub transform: Option<SectionTransformFn>,
}

impl std::fmt::Debug for SectionOverride {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SectionOverride")
            .field("action", &self.action)
            .field("content", &self.content)
            .field("transform", &self.transform.as_ref().map(|_| "Fn(...)"))
            .finish()
    }
}

impl SectionOverride {
    /// Replace the section content with `content`.
    pub fn replace(content: impl Into<String>) -> Self {
        Self {
            action: SectionOverrideAction::Replace,
            content: Some(content.into()),
            transform: None,
        }
    }

    /// Remove the section.
    pub fn remove() -> Self {
        Self {
            action: SectionOverrideAction::Remove,
            content: None,
            transform: None,
        }
    }

    /// Append `content` to the section.
    pub fn append(content: impl Into<String>) -> Self {
        Self {
            action: SectionOverrideAction::Append,
            content: Some(content.into()),
            transform: None,
        }
    }

    /// Prepend `content` to the section.
    pub fn prepend(content: impl Into<String>) -> Self {
        Self {
            action: SectionOverrideAction::Prepend,
            content: Some(content.into()),
            transform: None,
        }
    }

    /// Transform the rendered section content with a client-side callback.
    ///
    /// # Example
    ///
    /// ```
    /// use copilot_sdk::SectionOverride;
    ///
    /// let override_ = SectionOverride::transform(|content| {
    ///     Box::pin(async move { content.to_uppercase() })
    /// });
    /// ```
    pub fn transform<F>(callback: F) -> Self
    where
        F: Fn(String) -> SectionTransformFuture + Send + Sync + 'static,
    {
        Self {
            action: SectionOverrideAction::Transform,
            content: None,
            transform: Some(Arc::new(callback)),
        }
    }

    /// Returns the transform callback when this override uses
    /// [`SectionOverrideAction::Transform`].
    pub fn transform_fn(&self) -> Option<&SectionTransformFn> {
        if self.action == SectionOverrideAction::Transform {
            self.transform.as_ref()
        } else {
            None
        }
    }
}

/// System message configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemMessageConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<SystemMessageMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Section-level overrides, used with [`SystemMessageMode::Customize`].
    /// Keyed by the section id (e.g. `tool_efficiency`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sections: Option<HashMap<String, SectionOverride>>,
}

impl SystemMessageConfig {
    /// Append-mode config with additional instructions.
    pub fn append(content: impl Into<String>) -> Self {
        Self {
            mode: Some(SystemMessageMode::Append),
            content: Some(content.into()),
            sections: None,
        }
    }

    /// Replace-mode config with a complete system message.
    pub fn replace(content: impl Into<String>) -> Self {
        Self {
            mode: Some(SystemMessageMode::Replace),
            content: Some(content.into()),
            sections: None,
        }
    }

    /// Customize-mode config with per-section overrides.
    pub fn customize(sections: HashMap<String, SectionOverride>) -> Self {
        Self {
            mode: Some(SystemMessageMode::Customize),
            content: None,
            sections: Some(sections),
        }
    }

    /// Add or replace a section override, returning `self` for chaining.
    pub fn with_section(
        mut self,
        section: SystemMessageSection,
        override_: SectionOverride,
    ) -> Self {
        self.mode.get_or_insert(SystemMessageMode::Customize);
        self.sections
            .get_or_insert_with(HashMap::new)
            .insert(section.id().to_string(), override_);
        self
    }
}

/// Stable extension identity for session participants that provide canvases.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInfo {
    /// Extension namespace/source, e.g. `github-app`.
    pub source: String,
    /// Stable provider name within the source namespace.
    pub name: String,
}

/// Configuration for large tool-output handling.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LargeToolOutputConfig {
    /// Whether large output handling is enabled (default `true`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Maximum size in bytes before output is written to a temp file (default `51200`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size_bytes: Option<u64>,
    /// Directory to write temp files to. Defaults to the OS temp directory.
    ///
    /// Serialized as `outputDir` to match the runtime wire contract.
    #[serde(rename = "outputDir", skip_serializing_if = "Option::is_none")]
    pub output_directory: Option<String>,
}

/// Reasoning summary mode for models with configurable reasoning summaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningSummary {
    /// Do not request reasoning summaries from the model.
    None,
    /// Request a concise summary of the model's reasoning.
    Concise,
    /// Request a detailed summary of the model's reasoning.
    Detailed,
}

/// Vision-specific limit overrides.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelCapabilitiesOverrideLimitsVision {
    /// MIME types the model accepts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_media_types: Option<Vec<String>>,
    /// Maximum number of images per prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_prompt_images: Option<u64>,
    /// Maximum image size in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_prompt_image_size: Option<u64>,
}

/// Token-limit overrides for prompts, outputs, and the context window.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelCapabilitiesOverrideLimits {
    /// Maximum number of prompt/input tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_prompt_tokens: Option<u64>,
    /// Maximum number of output/completion tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u64>,
    /// Maximum total context window size in tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_context_window_tokens: Option<u64>,
    /// Vision-specific limits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vision: Option<ModelCapabilitiesOverrideLimitsVision>,
}

/// Feature flags indicating what the model supports.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCapabilitiesOverrideSupports {
    /// Whether this model supports vision/image input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vision: Option<bool>,
    /// Whether this model supports reasoning effort configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<bool>,
}

/// Partial override of a model's advertised capabilities.
///
/// Useful for BYOK providers whose capabilities the runtime cannot discover.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelCapabilitiesOverride {
    /// Feature flags indicating what the model supports.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supports: Option<ModelCapabilitiesOverrideSupports>,
    /// Token limits for prompts, outputs, and context window.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<ModelCapabilitiesOverrideLimits>,
}

/// Configuration applied to the session's default agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultAgentConfig {
    /// Tool names to exclude from the default agent.
    ///
    /// Excluded tools remain available to custom sub-agents that list them in
    /// their `tools` array, which keeps the default agent's context clean.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excluded_tools: Option<Vec<String>>,
}

/// Remote session export and steering mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RemoteSessionMode {
    /// Disable remote session export and steering.
    #[default]
    Off,
    /// Export session events to GitHub without enabling remote steering.
    Export,
    /// Enable both remote session export and remote steering.
    On,
}

/// Where the runtime persists a given class of credentials or caches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StorageMode {
    /// Persist to disk across sessions.
    #[default]
    Persistent,
    /// Keep in memory only; discarded when the session ends.
    InMemory,
}

/// Repository a cloud session is bound to.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudSessionRepository {
    /// Repository owner (user or organization).
    pub owner: String,
    /// Repository name.
    pub name: String,
    /// Branch to work against. Defaults to the repository default branch.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

/// Options for creating a remote session in the cloud.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudSessionOptions {
    /// Repository the cloud session operates on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<CloudSessionRepository>,
}

/// A plugin installed into a session.
///
/// Mirrors the `SessionInstalledPlugin` schema definition, which declares
/// `additionalProperties: false`. The schema spells `installed_at` and
/// `cache_path` in snake_case, so this struct deliberately does **not** use
/// `rename_all = "camelCase"` -- the Rust field names are already the exact
/// wire names.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SessionInstalledPlugin {
    /// Plugin name.
    pub name: String,
    /// Marketplace the plugin came from (empty string for direct repo installs).
    pub marketplace: String,
    /// Installation timestamp (ISO-8601).
    pub installed_at: String,
    /// Whether the plugin is currently enabled.
    pub enabled: bool,
    /// Installed version, if known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Path where the plugin is cached locally.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_path: Option<String>,
    /// Source descriptor for direct repo installs (when `marketplace` is empty).
    ///
    /// The schema marks this `x-opaque-json`: it is a union of an
    /// `owner/repo` string and three object forms, so it is carried verbatim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<serde_json::Value>,
}

/// Patch applied to a live session via `session.options.update`.
///
/// Every field is optional: only the fields you set are sent, and the runtime
/// leaves everything else untouched.
///
/// # Example
///
/// ```no_run
/// use copilot_sdk::SessionUpdateOptions;
///
/// # async fn run(session: &copilot_sdk::Session) -> copilot_sdk::Result<()> {
/// session
///     .update_options(SessionUpdateOptions {
///         coauthor_enabled: Some(false),
///         skip_custom_instructions: Some(true),
///         ..Default::default()
///     })
///     .await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionUpdateOptions {
    /// Model ID to use for assistant turns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Reasoning effort for the selected model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    /// Identifier of the client driving the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,
    /// Identifier sent to LSP-style integrations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lsp_client_name: Option<String>,
    /// Stable integration identifier for analytics and rate-limit attribution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integration_id: Option<String>,
    /// Feature-flag IDs mapped to their enabled state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feature_flags: Option<HashMap<String, bool>>,
    /// Whether experimental capabilities are enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_experimental_mode: Option<bool>,
    /// Custom model-provider configuration (BYOK).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<serde_json::Value>,
    /// Absolute working-directory path for shell tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
    /// Allowlist of tool names available to this session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_tools: Option<Vec<String>>,
    /// Denylist of tool names for this session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excluded_tools: Option<Vec<String>>,
    /// Which filter wins when a tool appears in both lists.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_filter_precedence: Option<String>,
    /// Whether shell-script safety heuristics are enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_script_safety: Option<bool>,
    /// Shell init profile (`None` or `NonInteractive`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell_init_profile: Option<String>,
    /// Per-shell process flags.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell_process_flags: Option<Vec<String>>,
    /// Sandbox configuration; opaque to SDK consumers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_config: Option<serde_json::Value>,
    /// Whether interactive shell sessions are logged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_interactive_shells: Option<bool>,
    /// How environment values are transmitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_value_mode: Option<String>,
    /// Additional directories to search for skills.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_directories: Option<Vec<String>>,
    /// Skill IDs excluded from this session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_skills: Option<Vec<String>>,
    /// Discover custom instructions on demand after successful file views.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_on_demand_instruction_discovery: Option<bool>,
    /// Full set of installed plugins; replaces the existing list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_plugins: Option<Vec<SessionInstalledPlugin>>,
    /// Default custom agents to local-only execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_agents_local_only: Option<bool>,
    /// Skip loading custom instruction sources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_custom_instructions: Option<bool>,
    /// Instruction source IDs excluded from the system prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_instruction_sources: Option<Vec<String>>,
    /// Include the `Co-authored-by` trailer in commit messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coauthor_enabled: Option<bool>,
    /// Path for trajectory output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trajectory_file: Option<String>,
    /// Stream model responses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_streaming: Option<bool>,
    /// Override URL for the Copilot API endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copilot_url: Option<String>,
    /// Disable the `ask_user` tool to encourage autonomous behavior.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask_user_disabled: Option<bool>,
    /// Allow auto-mode continuation across turns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_on_auto_mode: Option<bool>,
    /// Whether the session is running in an interactive UI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub running_in_interactive_mode: Option<bool>,
    /// Surface reasoning-summary events from the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_reasoning_summaries: Option<bool>,
    /// Runtime context discriminator (e.g. `cli`, `actions`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_context: Option<String>,
    /// Override directory for the session-events log.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events_log_directory: Option<String>,
    /// Additional content-exclusion policies merged into the session policy set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_content_exclusion_policies: Option<Vec<serde_json::Value>>,
    /// Expose the `manage_schedule` tool to the agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manage_schedule_enabled: Option<bool>,
    /// Skip embedding-retrieval pipeline initialization and execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_embedding_retrieval: Option<bool>,
    /// Organization-level custom instructions injected into the system prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_custom_instructions: Option<String>,
    /// Enable loading of `.github/hooks/` filesystem hooks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_file_hooks: Option<bool>,
    /// Enable host git operations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_host_git_operations: Option<bool>,
    /// Enable cross-session store reads and writes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_session_store: Option<bool>,
    /// Enable skill directory scanning and loading.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_skills: Option<bool>,
}

impl SessionUpdateOptions {
    /// Returns `true` when no field is set, meaning there is nothing to send.
    pub fn is_empty(&self) -> bool {
        serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_object().map(|o| o.is_empty()))
            .unwrap_or(true)
    }
}

/// Azure-specific provider options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,
}

/// Provider configuration for BYOK (Bring Your Own Key).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderConfig {
    pub base_url: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub provider_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wire_api: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub azure: Option<AzureOptions>,
}

// Environment variable names for BYOK configuration
impl ProviderConfig {
    /// Environment variable for API key
    pub const ENV_API_KEY: &'static str = "COPILOT_SDK_BYOK_API_KEY";
    /// Environment variable for base URL
    pub const ENV_BASE_URL: &'static str = "COPILOT_SDK_BYOK_BASE_URL";
    /// Environment variable for provider type
    pub const ENV_PROVIDER_TYPE: &'static str = "COPILOT_SDK_BYOK_PROVIDER_TYPE";
    /// Environment variable for model
    pub const ENV_MODEL: &'static str = "COPILOT_SDK_BYOK_MODEL";

    /// Check if BYOK environment variables are configured.
    ///
    /// Returns true if `COPILOT_SDK_BYOK_API_KEY` is set and non-empty.
    pub fn is_env_configured() -> bool {
        std::env::var(Self::ENV_API_KEY)
            .map(|v| !v.is_empty())
            .unwrap_or(false)
    }

    /// Load ProviderConfig from `COPILOT_SDK_BYOK_*` environment variables.
    ///
    /// Returns `Some(ProviderConfig)` if API key is set, `None` otherwise.
    ///
    /// Environment variables:
    /// - `COPILOT_SDK_BYOK_API_KEY` (required): API key for the provider
    /// - `COPILOT_SDK_BYOK_BASE_URL` (optional): Base URL (defaults to OpenAI)
    /// - `COPILOT_SDK_BYOK_PROVIDER_TYPE` (optional): Provider type (defaults to "openai")
    pub fn from_env() -> Option<Self> {
        if !Self::is_env_configured() {
            return None;
        }

        let api_key = std::env::var(Self::ENV_API_KEY).ok();
        let base_url = std::env::var(Self::ENV_BASE_URL)
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
        let provider_type = std::env::var(Self::ENV_PROVIDER_TYPE)
            .ok()
            .or_else(|| Some("openai".to_string()));

        Some(Self {
            base_url,
            provider_type,
            api_key,
            wire_api: None,
            bearer_token: None,
            azure: None,
        })
    }

    /// Load model from `COPILOT_SDK_BYOK_MODEL` environment variable.
    ///
    /// Returns `Some(model)` if set and non-empty, `None` otherwise.
    pub fn model_from_env() -> Option<String> {
        std::env::var(Self::ENV_MODEL)
            .ok()
            .filter(|v| !v.is_empty())
    }
}

/// Endpoint URLs returned in a Copilot user snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopilotUserResponseEndpoints {
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopilotOrganization {
    /// Organization login, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<String>,
    /// Organization display name, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Quota snapshot entry embedded in a Copilot user snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CopilotUserQuotaSnapshot {
    /// Total entitlement for this quota bucket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entitlement: Option<f64>,
    /// Number of overage events already consumed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overage_count: Option<f64>,
    /// Whether overage usage is permitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overage_permitted: Option<bool>,
    /// Percentage of quota remaining.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub percent_remaining: Option<f64>,
    /// Quota identifier string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_id: Option<String>,
    /// Remaining quota units for the quota bucket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_remaining: Option<f64>,
    /// Remaining quota units, when reported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remaining: Option<f64>,
    /// Whether the quota is unlimited.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unlimited: Option<bool>,
    /// Timestamp of the snapshot in UTC.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp_utc: Option<String>,
    /// Whether this quota bucket actually has a quota.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_quota: Option<bool>,
    /// Quota reset instant expressed as a numeric timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_reset_at: Option<f64>,
    /// Whether token-based billing applies to this quota bucket.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_based_billing: Option<bool>,
}

/// Snapshot of the authenticated user's Copilot subscription information.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CopilotUserResponse {
    /// Authenticated login name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub login: Option<String>,
    /// Copilot access SKU identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_type_sku: Option<String>,
    /// Analytics tracking identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analytics_tracking_id: Option<String>,
    /// Assigned date, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assigned_date: Option<String>,
    /// Whether the user can sign up for the limited plan.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub can_signup_for_limited: Option<bool>,
    /// Whether chat is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chat_enabled: Option<bool>,
    /// Copilot plan tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copilot_plan: Option<String>,
    /// Whether `.copilotignore` support is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub copilotignore_enabled: Option<bool>,
    /// Endpoint URLs associated with the authenticated user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoints: Option<CopilotUserResponseEndpoints>,
    /// Organization login names associated with the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_login_list: Option<Vec<String>>,
    /// Organization entries associated with the user.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_list: Option<Vec<Option<CopilotOrganization>>>,
    /// Whether Codex agent features are enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codex_agent_enabled: Option<bool>,
    /// Whether MCP features are enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mcp_enabled: Option<bool>,
    /// Quota reset date in local display form.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_reset_date: Option<String>,
    /// Quota snapshot payload keyed by quota type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_snapshots: Option<HashMap<String, Option<CopilotUserQuotaSnapshot>>>,
    /// Whether telemetry is restricted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restricted_telemetry: Option<bool>,
    /// Whether token-based billing is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_based_billing: Option<bool>,
    /// Quota reset date in UTC form.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_reset_date_utc: Option<String>,
    /// Limited-user quota payload keyed by quota name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limited_user_quotas: Option<HashMap<String, f64>>,
    /// Limited-user reset date.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limited_user_reset_date: Option<String>,
    /// Monthly quota payload keyed by quota name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub monthly_quotas: Option<HashMap<String, f64>>,
    /// Whether cloud session storage is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_session_storage_enabled: Option<bool>,
    /// Whether CLI remote control is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cli_remote_control_enabled: Option<bool>,
}

/// Auth credential payload accepted by session-auth operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum AuthInfo {
    /// HMAC-based authentication used by GitHub-internal services.
    Hmac {
        /// Authentication host.
        host: String,
        /// HMAC secret used to sign requests.
        hmac: String,
        /// Snapshot of the authenticated user's Copilot subscription info.
        #[serde(rename = "copilotUser", skip_serializing_if = "Option::is_none")]
        copilot_user: Option<CopilotUserResponse>,
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
        #[serde(rename = "envVar")]
        env_var: String,
        /// Snapshot of the authenticated user's Copilot subscription info.
        #[serde(rename = "copilotUser", skip_serializing_if = "Option::is_none")]
        copilot_user: Option<CopilotUserResponse>,
    },
    /// SDK-side token authentication configured directly by the caller.
    Token {
        /// Authentication host.
        host: String,
        /// The token value itself.
        token: String,
        /// Snapshot of the authenticated user's Copilot subscription info.
        #[serde(rename = "copilotUser", skip_serializing_if = "Option::is_none")]
        copilot_user: Option<CopilotUserResponse>,
    },
    /// Direct Copilot API authentication via environment-provided token settings.
    CopilotApiToken {
        /// Authentication host.
        host: String,
        /// Snapshot of the authenticated user's Copilot subscription info.
        #[serde(rename = "copilotUser", skip_serializing_if = "Option::is_none")]
        copilot_user: Option<CopilotUserResponse>,
    },
    /// OAuth user authentication backed by the runtime's token store.
    User {
        /// Authentication host.
        host: String,
        /// OAuth user login.
        login: String,
        /// Snapshot of the authenticated user's Copilot subscription info.
        #[serde(rename = "copilotUser", skip_serializing_if = "Option::is_none")]
        copilot_user: Option<CopilotUserResponse>,
    },
    /// Authentication delegated to the GitHub CLI.
    GhCli {
        /// Authentication host.
        host: String,
        /// User login reported by `gh auth status`.
        login: String,
        /// Token returned by `gh auth token`.
        token: String,
        /// Snapshot of the authenticated user's Copilot subscription info.
        #[serde(rename = "copilotUser", skip_serializing_if = "Option::is_none")]
        copilot_user: Option<CopilotUserResponse>,
    },
    /// API-key authentication for non-GitHub providers.
    ApiKey {
        /// The API key value.
        #[serde(rename = "apiKey")]
        api_key: String,
        /// Authentication host.
        host: String,
        /// Snapshot of the authenticated user's Copilot subscription info.
        #[serde(rename = "copilotUser", skip_serializing_if = "Option::is_none")]
        copilot_user: Option<CopilotUserResponse>,
    },
}

// =============================================================================
// MCP Server Configuration
// =============================================================================

/// Configuration for a local/stdio MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpLocalServerConfig {
    pub tools: Vec<String>,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub server_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

/// Configuration for a remote MCP server (HTTP or SSE).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpRemoteServerConfig {
    pub tools: Vec<String>,
    pub url: String,
    #[serde(default = "default_mcp_type", rename = "type")]
    pub server_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,
}

fn default_mcp_type() -> String {
    "http".to_string()
}

/// MCP server configuration (either local or remote).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McpServerConfig {
    Local(McpLocalServerConfig),
    Remote(McpRemoteServerConfig),
}

// =============================================================================
// Custom Agent Configuration
// =============================================================================

/// Configuration for a custom agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomAgentConfig {
    pub name: String,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub infer: Option<bool>,
}

// =============================================================================
// Attachment Types
// =============================================================================

/// Optional line range to scope a file attachment to a specific section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendAttachmentFileLineRange {
    /// Start line number (1-based).
    pub start: u64,
    /// End line number (1-based, inclusive).
    pub end: u64,
}

/// Type of GitHub reference attachment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SendAttachmentGithubReferenceType {
    /// GitHub issue reference.
    Issue,
    /// GitHub pull-request reference.
    Pr,
    /// GitHub discussion reference.
    Discussion,
}

/// Position within an editor selection attachment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendAttachmentSelectionPosition {
    /// Line number (0-based).
    pub line: u64,
    /// Character offset within the line (0-based).
    pub character: u64,
}

/// Position range for an editor selection attachment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SendAttachmentSelectionDetails {
    /// Start position of the selection.
    pub start: SendAttachmentSelectionPosition,
    /// End position of the selection.
    pub end: SendAttachmentSelectionPosition,
}

/// A user message attachment payload sent to the runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SendAttachment {
    /// File attachment.
    File {
        /// Absolute file path.
        path: String,
        /// User-facing display name for the attachment.
        #[serde(rename = "displayName")]
        display_name: String,
        /// Optional line range limiting the attached region.
        #[serde(rename = "lineRange", skip_serializing_if = "Option::is_none")]
        line_range: Option<SendAttachmentFileLineRange>,
    },
    /// Directory attachment.
    Directory {
        /// Absolute directory path.
        path: String,
        /// User-facing display name for the attachment.
        #[serde(rename = "displayName")]
        display_name: String,
    },
    /// Code-selection attachment from an editor.
    Selection {
        /// Absolute path to the file containing the selection.
        #[serde(rename = "filePath")]
        file_path: String,
        /// User-facing display name for the selection.
        #[serde(rename = "displayName")]
        display_name: String,
        /// The selected text content.
        text: String,
        /// Position range of the selection within the file.
        selection: SendAttachmentSelectionDetails,
    },
    /// GitHub issue, pull request, or discussion reference.
    GithubReference {
        /// Issue, pull request, or discussion number.
        number: u64,
        /// Title of the referenced item.
        title: String,
        /// Type of GitHub reference.
        #[serde(rename = "referenceType")]
        reference_type: SendAttachmentGithubReferenceType,
        /// Current state of the referenced item.
        state: String,
        /// URL to the referenced item on GitHub.
        url: String,
    },
    /// Inline blob attachment with base64-encoded contents.
    Blob {
        /// Base64-encoded content.
        data: String,
        /// MIME type of the inline data.
        #[serde(rename = "mimeType")]
        mime_type: String,
        /// User-facing display name for the attachment.
        #[serde(rename = "displayName", skip_serializing_if = "Option::is_none")]
        display_name: Option<String>,
    },
}

/// Attachment item for user messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMessageAttachment {
    #[serde(rename = "type")]
    pub attachment_type: AttachmentType,
    pub path: String,
    pub display_name: String,
}

// =============================================================================
// Tool Definition (SDK-side)
// =============================================================================

/// Tool definition for registration with a session.
///
/// Use the builder pattern to create tools:
/// ```no_run
/// use copilot_sdk::{Client, SessionConfig, Tool, ToolHandler, ToolResultObject};
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() -> copilot_sdk::Result<()> {
/// let client = Client::builder().build()?;
/// client.start().await?;
///
/// let tool = Tool::new("get_weather")
///     .description("Get weather for a city")
///     .schema(serde_json::json!({
///         "type": "object",
///         "properties": { "city": { "type": "string" } },
///         "required": ["city"]
///     }));
///
/// let session = client.create_session(SessionConfig {
///     tools: vec![tool.clone()],
///     ..Default::default()
/// }).await?;
///
/// let handler: ToolHandler = Arc::new(|_name, args| {
///     let city = args.get("city").and_then(|v| v.as_str()).unwrap_or("unknown");
///     ToolResultObject::text(format!("Weather in {}: sunny", city))
/// });
/// session.register_tool_with_handler(tool, Some(handler)).await;
/// client.stop().await;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters_schema: serde_json::Value,
    pub overrides_built_in_tool: bool,
    pub skip_permission: bool,
    // Handler is stored separately in Session since it's not Clone-friendly
}

impl Tool {
    /// Create a new tool with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            parameters_schema: serde_json::json!({}),
            overrides_built_in_tool: false,
            skip_permission: false,
        }
    }

    /// Set the tool description.
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set the parameters JSON schema.
    pub fn schema(mut self, schema: serde_json::Value) -> Self {
        self.parameters_schema = schema;
        self
    }

    /// Add a parameter to the tool's JSON schema.
    ///
    /// Builds the schema incrementally using the builder pattern.
    pub fn parameter(
        mut self,
        name: impl Into<String>,
        param_type: impl Into<String>,
        description: impl Into<String>,
        required: bool,
    ) -> Self {
        let name = name.into();

        // Ensure schema has the right shape
        if self.parameters_schema.get("type").is_none() {
            self.parameters_schema["type"] = serde_json::json!("object");
        }
        if self.parameters_schema.get("properties").is_none() {
            self.parameters_schema["properties"] = serde_json::json!({});
        }

        self.parameters_schema["properties"][&name] = serde_json::json!({
            "type": param_type.into(),
            "description": description.into(),
        });

        if required {
            if self.parameters_schema.get("required").is_none() {
                self.parameters_schema["required"] = serde_json::json!([]);
            }
            if let Some(arr) = self.parameters_schema["required"].as_array_mut() {
                arr.push(serde_json::json!(name));
            }
        }

        self
    }

    /// Derive the parameters JSON schema from a Rust type (requires the `schemars` feature).
    #[cfg(feature = "schemars")]
    pub fn typed_schema<T: schemars::JsonSchema>(mut self) -> Self {
        let schema = schemars::schema_for!(T);
        match serde_json::to_value(&schema) {
            Ok(value) => self.parameters_schema = value,
            Err(err) => {
                tracing::warn!("Failed to serialize schemars schema: {err}");
                self.parameters_schema = serde_json::json!({});
            }
        }
        self
    }

    /// Mark this tool as overriding a built-in tool.
    pub fn overrides_built_in_tool(mut self, value: bool) -> Self {
        self.overrides_built_in_tool = value;
        self
    }

    /// Skip permission checks for this tool.
    pub fn skip_permission(mut self, value: bool) -> Self {
        self.skip_permission = value;
        self
    }
}

impl std::fmt::Debug for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tool")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("overrides_built_in_tool", &self.overrides_built_in_tool)
            .field("skip_permission", &self.skip_permission)
            .finish()
    }
}

// Serialization for sending tool definitions to the CLI
impl Serialize for Tool {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let field_count = 3
            + if self.overrides_built_in_tool { 1 } else { 0 }
            + if self.skip_permission { 1 } else { 0 };
        let mut state = serializer.serialize_struct("Tool", field_count)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("parameters", &self.parameters_schema)?;
        if self.overrides_built_in_tool {
            state.serialize_field("overridesBuiltInTool", &self.overrides_built_in_tool)?;
        }
        if self.skip_permission {
            state.serialize_field("skipPermission", &self.skip_permission)?;
        }
        state.end()
    }
}

// =============================================================================
// Infinite Session Configuration
// =============================================================================

/// Configuration for infinite sessions (automatic context compaction).
///
/// When enabled, the SDK will automatically manage conversation context to prevent
/// buffer exhaustion. Thresholds are expressed as fractions (0.0 to 1.0).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InfiniteSessionConfig {
    /// Enable infinite sessions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Threshold for background compaction (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_compaction_threshold: Option<f64>,
    /// Threshold for buffer exhaustion handling (0.0 to 1.0).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buffer_exhaustion_threshold: Option<f64>,
}

impl InfiniteSessionConfig {
    /// Create an enabled infinite session config with default thresholds.
    pub fn enabled() -> Self {
        Self {
            enabled: Some(true),
            background_compaction_threshold: None,
            buffer_exhaustion_threshold: None,
        }
    }

    /// Create an infinite session config with custom thresholds.
    pub fn with_thresholds(background: f64, exhaustion: f64) -> Self {
        Self {
            enabled: Some(true),
            background_compaction_threshold: Some(background),
            buffer_exhaustion_threshold: Some(exhaustion),
        }
    }
}

// =============================================================================
// Session Hooks
// =============================================================================

/// Input for the pre-tool-use hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreToolUseHookInput {
    pub timestamp: i64,
    pub cwd: String,
    pub tool_name: String,
    pub tool_args: serde_json::Value,
}

/// Output for the pre-tool-use hook.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreToolUseHookOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_decision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_decision_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_args: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
}

/// Input for the post-tool-use hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostToolUseHookInput {
    pub timestamp: i64,
    pub cwd: String,
    pub tool_name: String,
    pub tool_args: serde_json::Value,
    pub tool_result: serde_json::Value,
}

/// Output for the post-tool-use hook.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostToolUseHookOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
}

/// Input for the user-prompt-submitted hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPromptSubmittedHookInput {
    pub timestamp: i64,
    pub cwd: String,
    pub prompt: String,
}

/// Output for the user-prompt-submitted hook.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserPromptSubmittedHookOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
}

/// Input for the session-start hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStartHookInput {
    pub timestamp: i64,
    pub cwd: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_prompt: Option<String>,
}

/// Output for the session-start hook.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStartHookOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_config: Option<serde_json::Value>,
}

/// Input for the session-end hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionEndHookInput {
    pub timestamp: i64,
    pub cwd: String,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Output for the session-end hook.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionEndHookOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleanup_actions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_summary: Option<String>,
}

/// Input for the error-occurred hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorOccurredHookInput {
    pub timestamp: i64,
    pub cwd: String,
    pub error: String,
    pub error_context: String,
    pub recoverable: bool,
}

/// Output for the error-occurred hook.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorOccurredHookOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_handling: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_notification: Option<String>,
}

/// Handler types for session hooks.
pub type PreToolUseHandler = Arc<dyn Fn(PreToolUseHookInput) -> PreToolUseHookOutput + Send + Sync>;
pub type PostToolUseHandler =
    Arc<dyn Fn(PostToolUseHookInput) -> PostToolUseHookOutput + Send + Sync>;
pub type UserPromptSubmittedHandler =
    Arc<dyn Fn(UserPromptSubmittedHookInput) -> UserPromptSubmittedHookOutput + Send + Sync>;
pub type SessionStartHandler =
    Arc<dyn Fn(SessionStartHookInput) -> SessionStartHookOutput + Send + Sync>;
pub type SessionEndHandler = Arc<dyn Fn(SessionEndHookInput) -> SessionEndHookOutput + Send + Sync>;
pub type ErrorOccurredHandler =
    Arc<dyn Fn(ErrorOccurredHookInput) -> ErrorOccurredHookOutput + Send + Sync>;

/// Configuration for session hooks.
///
/// Hooks allow intercepting and modifying behavior at key points in the session lifecycle.
#[derive(Clone, Default)]
pub struct SessionHooks {
    pub on_pre_tool_use: Option<PreToolUseHandler>,
    pub on_post_tool_use: Option<PostToolUseHandler>,
    pub on_user_prompt_submitted: Option<UserPromptSubmittedHandler>,
    pub on_session_start: Option<SessionStartHandler>,
    pub on_session_end: Option<SessionEndHandler>,
    pub on_error_occurred: Option<ErrorOccurredHandler>,
}

impl SessionHooks {
    /// Returns true if any hook handler is registered.
    pub fn has_any(&self) -> bool {
        self.on_pre_tool_use.is_some()
            || self.on_post_tool_use.is_some()
            || self.on_user_prompt_submitted.is_some()
            || self.on_session_start.is_some()
            || self.on_session_end.is_some()
            || self.on_error_occurred.is_some()
    }
}

impl std::fmt::Debug for SessionHooks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionHooks")
            .field("on_pre_tool_use", &self.on_pre_tool_use.is_some())
            .field("on_post_tool_use", &self.on_post_tool_use.is_some())
            .field(
                "on_user_prompt_submitted",
                &self.on_user_prompt_submitted.is_some(),
            )
            .field("on_session_start", &self.on_session_start.is_some())
            .field("on_session_end", &self.on_session_end.is_some())
            .field("on_error_occurred", &self.on_error_occurred.is_some())
            .finish()
    }
}

// =============================================================================
// Session Configuration
// =============================================================================

// =============================================================================
// Session Capabilities (host-reported)
// =============================================================================

/// UI capabilities reported by the CLI host for a session.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UiCapabilities {
    /// Whether the host supports interactive elicitation dialogs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elicitation: Option<bool>,
    /// Whether the runtime accepted the session's MCP Apps (SEP-1865) opt-in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_apps: Option<bool>,
    /// Whether the host supports canvas rendering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canvases: Option<bool>,
}

/// Capabilities reported by the CLI host for a session.
///
/// Populated from the `session.create` / `session.resume` response. Check the
/// relevant capability before invoking host-gated APIs such as
/// [`Session::ui`](crate::Session::ui).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionCapabilities {
    /// UI capabilities, when reported by the host.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ui: Option<UiCapabilities>,
}

impl SessionCapabilities {
    /// Returns `true` if the host reported support for interactive elicitation.
    pub fn supports_elicitation(&self) -> bool {
        self.ui
            .as_ref()
            .and_then(|ui| ui.elicitation)
            .unwrap_or(false)
    }

    /// Returns `true` if the host reported support for canvas rendering.
    pub fn supports_canvases(&self) -> bool {
        self.ui.as_ref().and_then(|ui| ui.canvases).unwrap_or(false)
    }
}

// =============================================================================
// UI Elicitation
// =============================================================================

/// Elicitation mode: structured form input or browser redirect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ElicitationMode {
    /// Structured form input.
    Form,
    /// Browser redirect (URL mode).
    Url,
}

/// Accepted string-format hints for free-text elicitation fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UIElicitationSchemaPropertyStringFormat {
    /// Email address string format.
    Email,
    /// URI string format.
    Uri,
    /// Calendar-date string format.
    Date,
    /// Date-time string format.
    DateTime,
}

/// Numeric JSON type accepted by an elicitation field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UIElicitationSchemaPropertyNumberType {
    /// Any JSON number.
    Number,
    /// Integer JSON numbers only.
    Integer,
}

/// Selectable option for a labeled single-select string field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UIElicitationStringOneOfFieldOption {
    /// Value submitted when this option is selected.
    pub r#const: String,
    /// Display label for this option.
    pub title: String,
}

/// Selectable option for a labeled multi-select string field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UIElicitationArrayAnyOfFieldOption {
    /// Value submitted when this option is selected.
    pub r#const: String,
    /// Display label for this option.
    pub title: String,
}

/// Item schema for an inline string-enum array field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UIElicitationArrayEnumFieldItems {
    /// Type discriminator. Always `string`.
    #[serde(rename = "type")]
    pub item_type: String,
    /// Allowed string values for each selected item.
    pub r#enum: Vec<String>,
}

/// Item schema for a labeled multi-select array field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UIElicitationArrayAnyOfFieldItems {
    /// Selectable options, each with a value and a display label.
    pub any_of: Vec<UIElicitationArrayAnyOfFieldOption>,
}

/// Single-select string field whose allowed values are defined inline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UIElicitationStringEnumField {
    /// Type discriminator. Always `string`.
    #[serde(rename = "type")]
    pub field_type: String,
    /// Human-readable label for the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Help text describing the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Allowed string values.
    pub r#enum: Vec<String>,
    /// Optional display labels for each enum value, in the same order as `enum`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_names: Option<Vec<String>>,
    /// Default value selected when the form is first shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// Single-select string field with labeled options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UIElicitationStringOneOfField {
    /// Type discriminator. Always `string`.
    #[serde(rename = "type")]
    pub field_type: String,
    /// Human-readable label for the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Help text describing the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Selectable options, each with a value and a display label.
    pub one_of: Vec<UIElicitationStringOneOfFieldOption>,
    /// Default value selected when the form is first shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// Multi-select string field whose allowed values are defined inline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UIElicitationArrayEnumField {
    /// Type discriminator. Always `array`.
    #[serde(rename = "type")]
    pub field_type: String,
    /// Human-readable label for the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Help text describing the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Minimum number of items the user must select.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<u64>,
    /// Maximum number of items the user may select.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u64>,
    /// Schema applied to each item in the array.
    pub items: UIElicitationArrayEnumFieldItems,
    /// Default values selected when the form is first shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Vec<String>>,
}

/// Multi-select string field with labeled options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UIElicitationArrayAnyOfField {
    /// Type discriminator. Always `array`.
    #[serde(rename = "type")]
    pub field_type: String,
    /// Human-readable label for the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Help text describing the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Minimum number of items the user must select.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<u64>,
    /// Maximum number of items the user may select.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u64>,
    /// Schema applied to each item in the array.
    pub items: UIElicitationArrayAnyOfFieldItems,
    /// Default values selected when the form is first shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<Vec<String>>,
}

/// Boolean elicitation field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UIElicitationSchemaPropertyBoolean {
    /// Type discriminator. Always `boolean`.
    #[serde(rename = "type")]
    pub field_type: String,
    /// Human-readable label for the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Help text describing the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Default value selected when the form is first shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<bool>,
}

/// Free-text string elicitation field.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UIElicitationSchemaPropertyString {
    /// Type discriminator. Always `string`.
    #[serde(rename = "type")]
    pub field_type: String,
    /// Human-readable label for the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Help text describing the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Minimum number of characters required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_length: Option<u64>,
    /// Maximum number of characters allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<u64>,
    /// Optional string-format hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<UIElicitationSchemaPropertyStringFormat>,
    /// Default value populated in the input when the form is first shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
}

/// Numeric elicitation field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UIElicitationSchemaPropertyNumber {
    /// Numeric JSON type accepted by the field.
    #[serde(rename = "type")]
    pub field_type: UIElicitationSchemaPropertyNumberType,
    /// Human-readable label for the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Help text describing the field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Minimum allowed value (inclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    /// Maximum allowed value (inclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    /// Default value populated in the input when the form is first shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<f64>,
}

/// Schema for a single elicitation form field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UIElicitationSchemaProperty {
    /// Inline string-enum field.
    StringEnum(UIElicitationStringEnumField),
    /// Labeled single-select string field.
    StringOneOf(UIElicitationStringOneOfField),
    /// Inline multi-select string field.
    ArrayEnum(UIElicitationArrayEnumField),
    /// Labeled multi-select string field.
    ArrayAnyOf(UIElicitationArrayAnyOfField),
    /// Boolean yes-or-no field.
    Boolean(UIElicitationSchemaPropertyBoolean),
    /// Free-text string field.
    String(UIElicitationSchemaPropertyString),
    /// Numeric field.
    Number(UIElicitationSchemaPropertyNumber),
}

/// Form-schema description for a UI elicitation request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UIElicitationSchema {
    /// Schema type indicator. Always `object`.
    #[serde(rename = "type")]
    pub schema_type: String,
    /// Form field definitions keyed by field name.
    pub properties: HashMap<String, UIElicitationSchemaProperty>,
    /// List of required field names.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

/// One submitted UI elicitation field value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UIElicitationFieldValue {
    /// String field value.
    String(String),
    /// Numeric field value.
    Number(f64),
    /// Boolean field value.
    Boolean(bool),
    /// Multi-select string field value.
    StringArray(Vec<String>),
}

/// The submitted content payload for an elicitation response.
pub type UIElicitationResponseContent = HashMap<String, UIElicitationFieldValue>;

/// Result returned from an elicitation request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ElicitationResult {
    /// User action: `accept` (submitted), `decline` (rejected), or `cancel` (dismissed).
    pub action: String,
    /// Form values submitted by the user (present when action is `accept`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<HashMap<String, serde_json::Value>>,
}

impl ElicitationResult {
    /// Construct a `cancel` result (used as a fallback when a handler fails).
    pub fn cancel() -> Self {
        Self {
            action: "cancel".to_string(),
            content: None,
        }
    }

    /// Returns `true` if the user accepted (submitted) the form.
    pub fn is_accept(&self) -> bool {
        self.action == "accept"
    }
}

/// Parameters for a raw elicitation request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElicitationParams {
    /// Message describing what information is needed from the user.
    pub message: String,
    /// JSON Schema describing the form fields to present.
    pub requested_schema: serde_json::Value,
}

/// Context for an elicitation handler invocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElicitationContext {
    /// Identifier of the session that triggered the elicitation request.
    pub session_id: String,
    /// Message describing what information is needed from the user.
    pub message: String,
    /// JSON Schema describing the form fields to present (form mode only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_schema: Option<serde_json::Value>,
    /// Elicitation mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<ElicitationMode>,
    /// The source that initiated the request (e.g. an MCP server name).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elicitation_source: Option<String>,
    /// URL to open in the user's browser (url mode only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Options for the [`SessionUi::input`](crate::SessionUi::input) convenience method.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiInputOptions {
    /// Title label for the input field.
    pub title: Option<String>,
    /// Descriptive text shown below the field.
    pub description: Option<String>,
    /// Minimum character length.
    pub min_length: Option<u64>,
    /// Maximum character length.
    pub max_length: Option<u64>,
    /// Semantic format hint (`email`, `uri`, `date`, `date-time`).
    pub format: Option<String>,
    /// Default value pre-populated in the field.
    pub default: Option<String>,
}

// =============================================================================
// Exit Plan Mode
// =============================================================================

/// Request to exit plan mode, awaiting the user's approval decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExitPlanModeRequest {
    /// Summary of the plan or proposed next step.
    pub summary: String,
    /// Full plan content, when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_content: Option<String>,
    /// Available actions the user can select.
    #[serde(default)]
    pub actions: Vec<String>,
    /// The action recommended by the runtime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended_action: Option<String>,
}

/// Response to an exit-plan-mode request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExitPlanModeResult {
    /// Whether the user approved exiting plan mode.
    pub approved: bool,
    /// Selected action, if the user chose one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_action: Option<String>,
    /// Optional feedback provided by the user.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feedback: Option<String>,
}

impl Default for ExitPlanModeResult {
    fn default() -> Self {
        Self {
            approved: true,
            selected_action: None,
            feedback: None,
        }
    }
}

// =============================================================================
// Auto Mode Switch
// =============================================================================

/// Request to switch to auto mode after an eligible rate limit.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoModeSwitchRequest {
    /// The rate-limit error code that triggered the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// Seconds until the rate limit resets, when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_after_seconds: Option<i64>,
}

/// Response to an auto-mode-switch request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AutoModeSwitchResponse {
    /// Allow the switch for this turn only.
    Yes,
    /// Allow the switch and persist it as a setting.
    YesAlways,
    /// Decline the switch.
    No,
}

impl Default for AutoModeSwitchResponse {
    fn default() -> Self {
        Self::No
    }
}

// =============================================================================
// Slash Commands
// =============================================================================

/// Context passed to a registered command handler when the user invokes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandContext {
    /// Session ID where the command was invoked.
    pub session_id: String,
    /// The full command text (e.g. `/deploy production`).
    pub command: String,
    /// Command name without the leading `/`.
    pub command_name: String,
    /// Raw argument string after the command name.
    pub args: String,
}

/// A slash-command declaration serialized to the CLI at session create/resume.
///
/// The handler is registered separately via
/// [`Session::register_command`](crate::Session::register_command).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandDeclaration {
    /// Command name (without leading `/`).
    pub name: String,
    /// Human-readable description shown in command completion UI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Configuration for creating a new session.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_dir: Option<PathBuf>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Tool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_message: Option<SystemMessageConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excluded_tools: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderConfig>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub streaming: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_agents: Option<Vec<CustomAgentConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_directories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_skills: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "requestPermission")]
    pub request_permission: Option<bool>,
    /// Infinite session configuration for automatic context compaction.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub infinite_sessions: Option<InfiniteSessionConfig>,

    /// Whether to request user input forwarding from the server.
    /// When true, `userInput.request` callbacks will be sent to the SDK.
    #[serde(skip_serializing_if = "Option::is_none", rename = "requestUserInput")]
    pub request_user_input: Option<bool>,

    /// Reasoning effort level: "low", "medium", "high", or "xhigh".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,

    /// Working directory for the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,

    /// Client name identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,

    /// Agent to use for the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    /// Slash-command declarations advertised to the CLI (TUI completion).
    /// Register the matching handlers via [`Session::register_command`](crate::Session::register_command).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands: Option<Vec<CommandDeclaration>>,

    /// Canvas declarations provided by this session, sent on `session.create`.
    /// Register the dispatch handler via
    /// [`Session::register_canvas_handler`](crate::Session::register_canvas_handler).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canvases: Option<Vec<crate::canvas::CanvasDeclaration>>,

    /// Session hooks for pre/post tool use, session lifecycle, etc.
    #[serde(skip)]
    pub hooks: Option<SessionHooks>,

    /// If true and provider/model not explicitly set, load from `COPILOT_SDK_BYOK_*` env vars.
    ///
    /// Default: false (explicit configuration preferred over environment variables)
    #[serde(skip)]
    pub auto_byok_from_env: bool,

    // =========================================================================
    // Model behaviour
    // =========================================================================
    /// Reasoning summary mode for models that support configurable summaries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_summary: Option<ReasoningSummary>,

    /// Override the runtime's view of the model's capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_capabilities: Option<ModelCapabilitiesOverride>,

    /// Large tool-output handling.
    #[serde(rename = "largeOutput", skip_serializing_if = "Option::is_none")]
    pub large_output: Option<LargeToolOutputConfig>,

    // =========================================================================
    // Reverse-RPC opt-ins
    // =========================================================================
    /// Request `elicitation.request` callbacks from the runtime.
    ///
    /// Register the handler with
    /// [`Session::register_elicitation_handler`](crate::Session::register_elicitation_handler).
    #[serde(rename = "requestElicitation", skip_serializing_if = "Option::is_none")]
    pub request_elicitation: Option<bool>,

    /// Request `exitPlanMode.request` callbacks from the runtime.
    #[serde(
        rename = "requestExitPlanMode",
        skip_serializing_if = "Option::is_none"
    )]
    pub request_exit_plan_mode: Option<bool>,

    /// Request `autoModeSwitch.request` callbacks from the runtime.
    #[serde(
        rename = "requestAutoModeSwitch",
        skip_serializing_if = "Option::is_none"
    )]
    pub request_auto_mode_switch: Option<bool>,

    /// Enable MCP apps support (sent on the wire as `requestMcpApps`).
    #[serde(rename = "requestMcpApps", skip_serializing_if = "Option::is_none")]
    pub enable_mcp_apps: Option<bool>,

    /// Request the canvas renderer capability from the runtime.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_canvas_renderer: Option<bool>,

    /// Request extension participation in this session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_extensions: Option<bool>,

    /// Stable extension identity for sessions that provide canvases.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_info: Option<ExtensionInfo>,

    // =========================================================================
    // Discovery and storage
    // =========================================================================
    /// Enable config-file discovery from the config directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_config_discovery: Option<bool>,

    /// Emit session telemetry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_session_telemetry: Option<bool>,

    /// Include streaming events emitted by sub-agents. Defaults to `true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_sub_agent_streaming_events: Option<bool>,

    /// Where MCP OAuth tokens are stored.
    #[serde(
        rename = "mcpOAuthTokenStorage",
        skip_serializing_if = "Option::is_none"
    )]
    pub mcp_oauth_token_storage: Option<StorageMode>,

    /// Where the embedding cache is stored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_cache_storage: Option<StorageMode>,

    /// Skip embedding-based retrieval entirely.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_embedding_retrieval: Option<bool>,

    /// Configuration applied to the session's default agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_agent: Option<DefaultAgentConfig>,

    /// Additional plugin directories to load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_directories: Option<Vec<String>>,

    /// Additional instruction directories to load.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_directories: Option<Vec<String>>,

    /// Organization-level custom instructions injected into the system message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_custom_instructions: Option<String>,

    /// Discover instructions on demand rather than eagerly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_on_demand_instruction_discovery: Option<bool>,

    /// Allow file-triggered hooks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_file_hooks: Option<bool>,

    /// Allow the runtime to run git operations on the host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_host_git_operations: Option<bool>,

    /// Enable the persistent session store.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_session_store: Option<bool>,

    /// Enable skills discovery and execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_skills: Option<bool>,

    // =========================================================================
    // Auth and remote
    // =========================================================================
    /// GitHub token scoped to this session.
    #[serde(rename = "gitHubToken", skip_serializing_if = "Option::is_none")]
    pub github_token: Option<String>,

    /// Remote session export and steering mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_session: Option<RemoteSessionMode>,

    /// Cloud session options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud: Option<CloudSessionOptions>,

    // =========================================================================
    // Post-create session options (sent via `session.updateOptions`)
    // =========================================================================
    /// Skip loading the user's custom instructions.
    #[serde(skip)]
    pub skip_custom_instructions: Option<bool>,

    /// Restrict custom agents to local definitions only.
    #[serde(skip)]
    pub custom_agents_local_only: Option<bool>,

    /// Add Copilot as a git co-author on commits it makes.
    #[serde(skip)]
    pub coauthor_enabled: Option<bool>,

    /// Allow the agent to manage scheduled prompts.
    #[serde(skip)]
    pub manage_schedule_enabled: Option<bool>,
}

/// Configuration for resuming an existing session.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResumeSessionConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Tool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderConfig>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub streaming: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_agents: Option<Vec<CustomAgentConfig>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_directories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_skills: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "requestPermission")]
    pub request_permission: Option<bool>,

    /// Whether to request user input forwarding from the server.
    #[serde(skip_serializing_if = "Option::is_none", rename = "requestUserInput")]
    pub request_user_input: Option<bool>,

    /// Reasoning effort level: "low", "medium", "high", or "xhigh".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,

    /// Working directory for the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,

    /// Client name identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_name: Option<String>,

    /// Agent to use for the session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,

    /// If true, skip resuming and create a new session instead.
    #[serde(default, skip_serializing_if = "is_false")]
    pub disable_resume: bool,

    /// Suppresses the `session.resumed` event emitted by the runtime.
    ///
    /// Defaults to `true` when resuming via
    /// [`Client::join_session`](crate::client::Client::join_session).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_resume_event: Option<bool>,

    /// System message configuration applied to the resumed session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_message: Option<SystemMessageConfig>,

    /// Infinite session configuration for resumed sessions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub infinite_sessions: Option<InfiniteSessionConfig>,

    /// Slash-command declarations advertised to the CLI (TUI completion).
    /// Register the matching handlers via [`Session::register_command`](crate::Session::register_command).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commands: Option<Vec<CommandDeclaration>>,

    /// Canvas declarations provided by this session, sent on `session.resume`.
    /// Register the dispatch handler via
    /// [`Session::register_canvas_handler`](crate::Session::register_canvas_handler).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canvases: Option<Vec<crate::canvas::CanvasDeclaration>>,

    /// Session hooks for pre/post tool use, session lifecycle, etc.
    #[serde(skip)]
    pub hooks: Option<SessionHooks>,

    // =========================================================================
    // Tool filtering
    // =========================================================================
    /// Allowlist of tool names available to the resumed session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub available_tools: Option<Vec<String>>,

    /// Denylist of tool names for the resumed session.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub excluded_tools: Option<Vec<String>>,

    // =========================================================================
    // Model behavior
    // =========================================================================
    /// Reasoning summary verbosity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_summary: Option<ReasoningSummary>,

    /// Overrides for the runtime's view of the model's capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_capabilities: Option<ModelCapabilitiesOverride>,

    /// Emit streaming events produced by sub-agents. Defaults to `true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_sub_agent_streaming_events: Option<bool>,

    /// Redirect oversized tool output to files instead of the transcript.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub large_output: Option<LargeToolOutputConfig>,

    // =========================================================================
    // Host capability requests
    // =========================================================================
    /// Request elicitation forwarding from the runtime.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_elicitation: Option<bool>,

    /// Request exit-plan-mode forwarding from the runtime.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_exit_plan_mode: Option<bool>,

    /// Request auto-mode-switch forwarding from the runtime.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_auto_mode_switch: Option<bool>,

    /// Advertise this connection as a canvas renderer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_canvas_renderer: Option<bool>,

    /// Request extension registration forwarding.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_extensions: Option<bool>,

    /// Enable MCP-app registration for this session.
    #[serde(rename = "requestMcpApps", skip_serializing_if = "Option::is_none")]
    pub enable_mcp_apps: Option<bool>,

    /// Metadata describing the extension hosting this connection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_info: Option<ExtensionInfo>,

    /// Canvas instances to restore in the host after resuming.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_canvases: Option<Vec<crate::canvas::OpenCanvasInstance>>,

    // =========================================================================
    // Discovery, storage, and instructions
    // =========================================================================
    /// Directory the runtime reads configuration from.
    #[serde(rename = "configDir", skip_serializing_if = "Option::is_none")]
    pub config_directory: Option<String>,

    /// Discover configuration files from the working directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_config_discovery: Option<bool>,

    /// Skip embedding-retrieval initialization and execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_embedding_retrieval: Option<bool>,

    /// Where the embedding cache is stored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_cache_storage: Option<StorageMode>,

    /// Where MCP OAuth tokens are stored.
    #[serde(
        rename = "mcpOAuthTokenStorage",
        skip_serializing_if = "Option::is_none"
    )]
    pub mcp_oauth_token_storage: Option<StorageMode>,

    /// Organization-level custom instructions injected into the system prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_custom_instructions: Option<String>,

    /// Discover custom instructions on demand after successful file views.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_on_demand_instruction_discovery: Option<bool>,

    /// Additional directories scanned for plugins.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_directories: Option<Vec<String>>,

    /// Additional directories scanned for instruction files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction_directories: Option<Vec<String>>,

    // =========================================================================
    // Runtime feature flags
    // =========================================================================
    /// Emit per-session telemetry.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_session_telemetry: Option<bool>,

    /// Load `.github/hooks/` filesystem hooks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_file_hooks: Option<bool>,

    /// Allow the runtime to run git operations on the host.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_host_git_operations: Option<bool>,

    /// Allow cross-session store reads and writes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_session_store: Option<bool>,

    /// Enable skill directory scanning and loading.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_skills: Option<bool>,

    /// Configuration for the built-in default agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_agent: Option<DefaultAgentConfig>,

    // =========================================================================
    // Resume behavior, auth, and remote
    // =========================================================================
    /// Continue any work that was pending when the session was suspended.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_pending_work: Option<bool>,

    /// GitHub token scoped to this session.
    #[serde(rename = "gitHubToken", skip_serializing_if = "Option::is_none")]
    pub github_token: Option<String>,

    /// Remote session export and steering mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote_session: Option<RemoteSessionMode>,

    // =========================================================================
    // Post-resume session options (sent via `session.options.update`)
    // =========================================================================
    /// Skip loading the user's custom instructions.
    #[serde(skip)]
    pub skip_custom_instructions: Option<bool>,

    /// Restrict custom agents to local definitions only.
    #[serde(skip)]
    pub custom_agents_local_only: Option<bool>,

    /// Add Copilot as a git co-author on commits it makes.
    #[serde(skip)]
    pub coauthor_enabled: Option<bool>,

    /// Allow the agent to manage scheduled prompts.
    #[serde(skip)]
    pub manage_schedule_enabled: Option<bool>,

    /// If true and provider not explicitly set, load from `COPILOT_SDK_BYOK_*` env vars.
    ///
    /// Default: false (explicit configuration preferred over environment variables)
    #[serde(skip)]
    pub auto_byok_from_env: bool,
}

/// Options for sending a message.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageOptions {
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<Vec<UserMessageAttachment>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
}

impl From<&str> for MessageOptions {
    fn from(prompt: &str) -> Self {
        Self {
            prompt: prompt.to_string(),
            attachments: None,
            mode: None,
        }
    }
}

impl From<String> for MessageOptions {
    fn from(prompt: String) -> Self {
        Self {
            prompt,
            attachments: None,
            mode: None,
        }
    }
}

// =============================================================================
// Client Options
// =============================================================================

/// Operating mode for a [`Client`](crate::client::Client).
///
/// Mirrors the Node.js `CopilotClientMode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CopilotClientMode {
    /// Standard mode: the CLI owns the workspace and the local filesystem.
    #[default]
    #[serde(rename = "copilot-cli")]
    CopilotCli,
    /// Empty mode: the CLI starts with no implicit workspace. Requires either
    /// [`ClientOptions::cwd`] or [`ClientOptions::session_fs`] to be set so the
    /// runtime knows where the session is rooted.
    Empty,
}

/// How a [`Client`](crate::client::Client) connects to the Copilot runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ConnectionKind {
    /// Spawn (or attach to) a Copilot CLI server and talk to it as a child.
    #[default]
    Child,
    /// Talk to the Copilot CLI over this process's own stdin/stdout, because
    /// the SDK is running as a child process of the CLI. Used by
    /// [`Client::join_session`](crate::client::Client::join_session).
    ParentProcess,
}

/// Options for creating a CopilotClient.
pub struct ClientOptions {
    pub cli_path: Option<PathBuf>,
    pub cli_args: Option<Vec<String>>,
    pub cwd: Option<PathBuf>,
    pub port: u16,
    pub use_stdio: bool,
    pub cli_url: Option<String>,
    pub log_level: LogLevel,
    pub auto_start: bool,
    pub auto_restart: bool,
    pub environment: Option<HashMap<String, String>>,
    /// GitHub personal access token for authentication.
    /// Cannot be used together with `cli_url`.
    pub github_token: Option<String>,
    /// Whether to use the logged-in user for auth.
    /// Defaults to true when github_token is empty. Cannot be used with `cli_url`.
    pub use_logged_in_user: Option<bool>,

    /// Tool specifications to deny (passed as `--deny-tool` arguments to the CLI).
    ///
    /// Each entry follows the CLI's tool specification format:
    /// - `"shell(git push)"` — deny a specific shell command
    /// - `"shell(git)"` — deny all git commands
    /// - `"shell(rm)"` — deny rm commands
    /// - `"shell"` — deny all shell commands
    /// - `"write"` — deny file write operations
    /// - `"MCP_SERVER(tool_name)"` — deny a specific MCP tool
    ///
    /// `--deny-tool` takes precedence over `--allow-tool` and `--allow-all-tools`.
    pub deny_tools: Option<Vec<String>>,

    /// Tool specifications to allow without manual approval
    /// (passed as `--allow-tool` arguments to the CLI).
    ///
    /// Each entry follows the same format as `deny_tools`.
    pub allow_tools: Option<Vec<String>>,

    /// If true, passes `--allow-all-tools` to the CLI.
    ///
    /// This allows Copilot to use any tool without asking for approval.
    /// Use `deny_tools` in combination to create an allowlist with exceptions.
    pub allow_all_tools: bool,

    /// OpenTelemetry configuration for tracing.
    pub telemetry: Option<TelemetryConfig>,

    /// Callback for custom model listing (BYOK).
    #[allow(clippy::type_complexity)]
    pub on_list_models: Option<
        Arc<
            dyn Fn() -> std::pin::Pin<
                    Box<
                        dyn std::future::Future<Output = Result<Vec<ModelInfo>, CopilotError>>
                            + Send,
                    >,
                > + Send
                + Sync,
        >,
    >,

    /// Operating mode.
    ///
    /// [`CopilotClientMode::Empty`] requires either [`ClientOptions::cwd`] or
    /// [`ClientOptions::session_fs`] to be set.
    pub mode: CopilotClientMode,

    /// Enables the client-provided session filesystem.
    ///
    /// When set, the SDK sends `sessionFs.setProvider` after connecting and
    /// every session must register a
    /// [`SessionFsProvider`](crate::session_fs::SessionFsProvider) via
    /// [`Session::register_session_fs_provider`](crate::session::Session::register_session_fs_provider).
    pub session_fs: Option<crate::session_fs::SessionFsConfig>,

    /// Callback returning the current W3C Trace Context.
    ///
    /// When set, `traceparent`/`tracestate` are injected into `session.create`
    /// and `session.resume` requests for distributed trace propagation.
    pub on_get_trace_context: Option<crate::trace::TraceContextProvider>,

    /// How the client connects to the runtime.
    ///
    /// Set to [`ConnectionKind::ParentProcess`] by
    /// [`Client::join_session`](crate::client::Client::join_session); you
    /// normally do not set this yourself.
    pub connection_kind: ConnectionKind,

    /// Idle timeout, in seconds, after which the runtime shuts a session down.
    ///
    /// `0` (the default) disables the timeout.
    pub session_idle_timeout_seconds: u64,

    /// Start the runtime with remote-session support (`--remote`).
    ///
    /// Required for [`SessionConfig::remote_session`] and
    /// [`SessionConfig::cloud`] to have any effect.
    pub enable_remote_sessions: bool,
}

impl Default for ClientOptions {
    fn default() -> Self {
        Self {
            cli_path: None,
            cli_args: None,
            cwd: None,
            port: 0,
            use_stdio: true,
            cli_url: None,
            log_level: LogLevel::Info,
            auto_start: true,
            auto_restart: true,
            environment: None,
            github_token: None,
            use_logged_in_user: None,
            deny_tools: None,
            allow_tools: None,
            allow_all_tools: false,
            telemetry: None,
            on_list_models: None,
            mode: CopilotClientMode::default(),
            session_fs: None,
            on_get_trace_context: None,
            connection_kind: ConnectionKind::default(),
            session_idle_timeout_seconds: 0,
            enable_remote_sessions: false,
        }
    }
}

impl std::fmt::Debug for ClientOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientOptions")
            .field("cli_path", &self.cli_path)
            .field("cli_args", &self.cli_args)
            .field("cwd", &self.cwd)
            .field("port", &self.port)
            .field("use_stdio", &self.use_stdio)
            .field("cli_url", &self.cli_url)
            .field("log_level", &self.log_level)
            .field("auto_start", &self.auto_start)
            .field("auto_restart", &self.auto_restart)
            .field("environment", &self.environment)
            .field("github_token", &self.github_token)
            .field("use_logged_in_user", &self.use_logged_in_user)
            .field("deny_tools", &self.deny_tools)
            .field("allow_tools", &self.allow_tools)
            .field("allow_all_tools", &self.allow_all_tools)
            .field("telemetry", &self.telemetry)
            .field(
                "on_list_models",
                &self.on_list_models.as_ref().map(|_| "Fn(...)"),
            )
            .field("mode", &self.mode)
            .field("session_fs", &self.session_fs)
            .field(
                "on_get_trace_context",
                &self.on_get_trace_context.as_ref().map(|_| "Fn(...)"),
            )
            .field("connection_kind", &self.connection_kind)
            .field(
                "session_idle_timeout_seconds",
                &self.session_idle_timeout_seconds,
            )
            .field("enable_remote_sessions", &self.enable_remote_sessions)
            .finish()
    }
}

impl Clone for ClientOptions {
    fn clone(&self) -> Self {
        Self {
            cli_path: self.cli_path.clone(),
            cli_args: self.cli_args.clone(),
            cwd: self.cwd.clone(),
            port: self.port,
            use_stdio: self.use_stdio,
            cli_url: self.cli_url.clone(),
            log_level: self.log_level,
            auto_start: self.auto_start,
            auto_restart: self.auto_restart,
            environment: self.environment.clone(),
            github_token: self.github_token.clone(),
            use_logged_in_user: self.use_logged_in_user,
            deny_tools: self.deny_tools.clone(),
            allow_tools: self.allow_tools.clone(),
            allow_all_tools: self.allow_all_tools,
            telemetry: self.telemetry.clone(),
            on_list_models: self.on_list_models.clone(),
            mode: self.mode,
            session_fs: self.session_fs.clone(),
            on_get_trace_context: self.on_get_trace_context.clone(),
            connection_kind: self.connection_kind,
            session_idle_timeout_seconds: self.session_idle_timeout_seconds,
            enable_remote_sessions: self.enable_remote_sessions,
        }
    }
}

// =============================================================================
// Response Types
// =============================================================================

/// Metadata about a session.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionMetadata {
    pub session_id: String,
    #[serde(default)]
    pub start_time: Option<String>,
    #[serde(default)]
    pub modified_time: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub is_remote: bool,
    /// Human-friendly name set via `/rename`.
    #[serde(default)]
    pub name: Option<String>,
    /// Runtime client name that created or last resumed this session.
    #[serde(default)]
    pub client_name: Option<String>,
    /// True for detached maintenance sessions hidden from normal resume lists.
    #[serde(default)]
    pub is_detached: bool,
    /// Working-directory and repository context recorded for the session.
    #[serde(default)]
    pub context: Option<SessionContext>,
    /// GitHub task ID, when this local session is bound to one.
    ///
    /// Only present for local sessions exported to remote control.
    #[serde(default)]
    pub mc_task_id: Option<String>,
}

/// Working-directory and repository context recorded for a session.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionContext {
    /// Most recent working directory for this session.
    pub cwd: String,
    /// Git repository root, if the cwd was inside a git repo.
    #[serde(default)]
    pub git_root: Option<String>,
    /// Repository slug in `owner/name` form, when known.
    #[serde(default)]
    pub repository: Option<String>,
    /// Repository host type.
    #[serde(default)]
    pub host_type: Option<String>,
    /// Active git branch.
    #[serde(default)]
    pub branch: Option<String>,
}

/// Response from a ping request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PingResponse {
    pub message: String,
    pub timestamp: i64,
    #[serde(default)]
    pub protocol_version: Option<u32>,
}

/// Response from status.get request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetStatusResponse {
    pub version: String,
    pub protocol_version: u32,
}

/// Response from auth.getStatus request.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAuthStatusResponse {
    pub is_authenticated: bool,
    #[serde(default)]
    pub auth_type: Option<String>,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub login: Option<String>,
    #[serde(default)]
    pub status_message: Option<String>,
}

/// Model capabilities - what the model supports.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCapabilities {
    #[serde(default)]
    pub supports: ModelSupports,
    #[serde(default)]
    pub limits: ModelLimits,
}

/// What features a model supports.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSupports {
    #[serde(default)]
    pub vision: bool,
    #[serde(default)]
    pub reasoning_effort: bool,
}

/// Vision limits for a model.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelVisionLimits {
    #[serde(default)]
    pub supported_media_types: Vec<String>,
    #[serde(default)]
    pub max_prompt_images: u32,
    #[serde(default)]
    pub max_prompt_image_size: u64,
}

/// Model limits.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelLimits {
    #[serde(default)]
    pub max_prompt_tokens: Option<u32>,
    #[serde(default)]
    pub max_context_window_tokens: u32,
    #[serde(default)]
    pub vision: Option<ModelVisionLimits>,
}

/// Model policy state.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelPolicy {
    pub state: String,
    #[serde(default)]
    pub terms: String,
}

/// Model billing information.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelBilling {
    #[serde(default)]
    pub multiplier: f64,
    /// Token-level pricing information for this model.
    #[serde(default)]
    pub token_prices: Option<ModelBillingTokenPrices>,
}

/// Token-level pricing information for a model.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelBillingTokenPrices {
    /// AI Credits cost per billing batch of input tokens.
    #[serde(default)]
    pub input_price: Option<f64>,
    /// AI Credits cost per billing batch of output tokens.
    #[serde(default)]
    pub output_price: Option<f64>,
    /// AI Credits cost per billing batch of cached tokens.
    #[serde(default)]
    pub cache_price: Option<f64>,
    /// Number of tokens per standard billing batch.
    #[serde(default)]
    pub batch_size: Option<u64>,
    /// Maximum context window tokens for the default tier.
    #[serde(default)]
    pub context_max: Option<u64>,
    /// Long context tier pricing, for models with extended context windows.
    #[serde(default)]
    pub long_context: Option<ModelBillingTokenPricesLongContext>,
}

/// Long context tier pricing for a model.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelBillingTokenPricesLongContext {
    /// AI Credits cost per billing batch of input tokens.
    #[serde(default)]
    pub input_price: Option<f64>,
    /// AI Credits cost per billing batch of output tokens.
    #[serde(default)]
    pub output_price: Option<f64>,
    /// AI Credits cost per billing batch of cached tokens.
    #[serde(default)]
    pub cache_price: Option<f64>,
    /// Maximum context window tokens for the long context tier.
    #[serde(default)]
    pub context_max: Option<u64>,
}

/// Information about an available model.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub capabilities: ModelCapabilities,
    #[serde(default)]
    pub policy: Option<ModelPolicy>,
    #[serde(default)]
    pub billing: Option<ModelBilling>,
    #[serde(default)]
    pub supported_reasoning_efforts: Option<Vec<String>>,
    #[serde(default)]
    pub default_reasoning_effort: Option<String>,
}

// =============================================================================
// Selection Attachment Types
// =============================================================================

/// Position in a text document (line + character).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelectionPosition {
    #[serde(default)]
    pub line: f64,
    #[serde(default)]
    pub character: f64,
}

/// Range within a text document.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SelectionRange {
    pub start: SelectionPosition,
    pub end: SelectionPosition,
}

/// Attachment representing a text selection in a file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionAttachment {
    pub file_path: String,
    pub display_name: String,
    pub text: String,
    pub selection: SelectionRange,
}

// =============================================================================
// User Input Types
// =============================================================================

/// Request for user input from the server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInputRequest {
    pub question: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choices: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_freeform: Option<bool>,
}

/// Response to a user input request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInputResponse {
    #[serde(default)]
    pub answer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub was_freeform: Option<bool>,
}

/// Context for a user input invocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInputInvocation {
    pub session_id: String,
}

// =============================================================================
// Session Lifecycle Types
// =============================================================================

/// Session lifecycle event type constants.
pub mod session_lifecycle_event_types {
    pub const CREATED: &str = "session.created";
    pub const DELETED: &str = "session.deleted";
    pub const UPDATED: &str = "session.updated";
    pub const FOREGROUND: &str = "session.foreground";
    pub const BACKGROUND: &str = "session.background";
}

/// Metadata for session lifecycle events.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionLifecycleEventMetadata {
    #[serde(default)]
    pub start_time: Option<String>,
    #[serde(default)]
    pub modified_time: Option<String>,
    #[serde(default)]
    pub summary: Option<String>,
}

/// Session lifecycle event notification.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionLifecycleEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub session_id: String,
    #[serde(default)]
    pub metadata: Option<SessionLifecycleEventMetadata>,
}

/// Response from session.getForeground.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetForegroundSessionResponse {
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub workspace_path: Option<String>,
}

/// Response from session.setForeground.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetForegroundSessionResponse {
    #[serde(default)]
    pub success: bool,
    #[serde(default)]
    pub error: Option<String>,
}

// =============================================================================
// Stop Error
// =============================================================================

/// Error collected during client stop.
#[derive(Debug, Clone)]
pub struct StopError {
    pub message: String,
    pub source: Option<String>,
}

impl std::fmt::Display for StopError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

// =============================================================================
// Session Mode
// =============================================================================

/// Session operation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionMode {
    Interactive,
    Plan,
    Autopilot,
}

// =============================================================================
// Set Model Options
// =============================================================================

/// Options for switching models mid-session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetModelOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<String>,
    /// Reasoning summary mode for models that support configurable summaries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_summary: Option<ReasoningSummary>,
    /// Override the runtime's view of the model's capabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_capabilities: Option<ModelCapabilitiesOverride>,
}

// =============================================================================
// Session Log Types
// =============================================================================

/// Log level for session log entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionLogLevel {
    Error,
    Info,
    Warning,
}

/// Options for adding a log entry to a session.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<SessionLogLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ephemeral: Option<bool>,
}

/// Result from adding a session log entry.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogResult {
    pub event_id: String,
}

/// Options for `session.lsp.initialize` — (re)loading the merged LSP configuration
/// set for the session's working directory.
///
/// This drives the CLI-side LSP configuration loader (the `session.lsp.initialize`
/// RPC). It is distinct from the crate's in-process [`crate::lsp::LspServer`], which is
/// a standalone Rust-native LSP 3.17 server.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LspInitializeOptions {
    /// Force re-initialization even when LSP configs were already loaded for the
    /// working directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force: Option<bool>,
    /// Git root used as the boundary when traversing for project-level LSP configs
    /// (supports monorepos).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_root: Option<String>,
    /// Working directory used to load project-level LSP configs. Defaults to the
    /// session working directory when omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub working_directory: Option<String>,
}

// =============================================================================
// Plan Data
// =============================================================================

/// Plan content data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

// =============================================================================
// Agent Info
// =============================================================================

/// Information about an available agent.
///
/// Mirrors the `AgentInfo` schema definition returned by `session.agent.list`
/// and `session.agent.getCurrent`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentInfo {
    /// Unique identifier of the custom agent.
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Stable identifier used for selection.
    ///
    /// For most agents this equals `name`; for plugin and builtin agents it may
    /// differ. The runtime always populates it, so a missing key only occurs
    /// against older CLIs and yields an empty string.
    #[serde(default)]
    pub id: String,
    /// Absolute local path of the agent definition.
    ///
    /// Only set for file-based agents loaded from disk; remote agents have none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Where the agent definition was loaded from.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<AgentInfoSource>,
    /// Whether the agent can be selected directly by the user.
    ///
    /// Agents marked `false` are subagent-only.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_invocable: Option<bool>,
    /// Allowed tool names for this agent.
    ///
    /// An empty vector means none; `None` means inherit the defaults.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<String>>,
    /// Preferred model id, inheriting the outer agent's model when `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// MCP server configurations attached to this agent, keyed by server name.
    ///
    /// Values mirror the MCP `mcpServers` schema and are carried verbatim.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_servers: Option<HashMap<String, serde_json::Value>>,
    /// Skill names preloaded into this agent's context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<String>>,
}

/// Where an agent definition was loaded from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentInfoSource {
    /// Loaded from the user's personal agent configuration.
    User,
    /// Loaded from the current project's repository configuration.
    Project,
    /// Inherited from a parent project or workspace.
    Inherited,
    /// Provided by a remote runtime or service.
    Remote,
    /// Contributed by an installed plugin.
    Plugin,
    /// Built into the Copilot runtime.
    Builtin,
}

// =============================================================================
// Fleet Start Options
// =============================================================================

/// Options for starting a fleet of parallel agents.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FleetStartOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
}

// =============================================================================
// Tools List Types
// =============================================================================

/// A tool definition as returned by the server.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolInfo {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub parameters: Option<serde_json::Value>,
}

/// Result from listing available tools.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsListResult {
    pub tools: Vec<ToolInfo>,
}

// =============================================================================
// Quota Types
// =============================================================================

/// A snapshot of quota usage.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaSnapshot {
    #[serde(rename = "type")]
    pub quota_type: String,
    #[serde(default)]
    pub limit: Option<u64>,
    #[serde(default)]
    pub used: Option<u64>,
    #[serde(default)]
    pub remaining: Option<u64>,
    #[serde(default)]
    pub resets_at: Option<String>,
}

/// Result from getting account quota.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuotaResult {
    pub quotas: Vec<QuotaSnapshot>,
}

// =============================================================================
// Shell Exec Types
// =============================================================================

/// Signal to send to a shell process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShellSignal {
    SIGINT,
    SIGKILL,
    SIGTERM,
}

/// Options for executing a shell command.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellExecOptions {
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}

/// Result from executing a shell command.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShellExecResult {
    pub process_id: String,
}

// =============================================================================
// Workspace File Types
// =============================================================================

/// Metadata about a workspace file.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFile {
    pub path: String,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub modified_at: Option<String>,
}

// =============================================================================
// Telemetry Config
// =============================================================================

/// OpenTelemetry configuration for the Copilot CLI process.
#[derive(Debug, Clone, Default)]
pub struct TelemetryConfig {
    /// OTLP HTTP endpoint URL.
    pub otlp_endpoint: Option<String>,
    /// File path for JSON-lines trace output.
    pub file_path: Option<String>,
    /// Exporter type: "otlp-http" or "file".
    pub exporter_type: Option<String>,
    /// Instrumentation scope name.
    pub source_name: Option<String>,
    /// Whether to capture message content (prompts/responses).
    pub capture_content: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_type_tag_roundtrip<T>(value: T, expected_tag: &str)
    where
        T: Serialize + serde::de::DeserializeOwned + PartialEq + std::fmt::Debug,
    {
        let json = serde_json::to_value(&value).unwrap();
        assert_eq!(json["type"], expected_tag);
        let decoded: T = serde_json::from_value(json).unwrap();
        assert_eq!(decoded, value);
    }

    #[test]
    fn test_tool_result_text() {
        let result = ToolResult::text("Hello, world!");
        assert_eq!(result.text_result_for_llm, "Hello, world!");
        assert_eq!(result.result_type, "success");
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error("Something went wrong");
        assert_eq!(result.result_type, "error");
        assert_eq!(result.error, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_permission_result() {
        let approved = PermissionRequestResult::approved();
        assert_eq!(approved.kind, "approved");
        assert!(approved.is_approved());
        assert!(!approved.is_denied());

        let denied = PermissionRequestResult::denied();
        assert!(denied.kind.starts_with("denied"));
        assert!(denied.is_denied());
        assert!(!denied.is_approved());
    }

    #[test]
    fn test_message_options_from_str() {
        let opts: MessageOptions = "Hello".into();
        assert_eq!(opts.prompt, "Hello");
    }

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert!(config.model.is_none());
        assert!(config.tools.is_empty());
    }

    #[test]
    fn test_session_config_serialization_with_new_fields() {
        let config = SessionConfig {
            session_id: Some("sess-1".into()),
            model: Some("gpt-4.1".into()),
            config_dir: Some(PathBuf::from("/tmp/copilot")),
            streaming: true,
            skill_directories: Some(vec!["skills".into()]),
            disabled_skills: Some(vec!["legacy_skill".into()]),
            request_permission: Some(true),
            ..Default::default()
        };

        let value = serde_json::to_value(&config).unwrap();
        assert_eq!(value["sessionId"], "sess-1");
        assert_eq!(value["model"], "gpt-4.1");
        assert_eq!(value["configDir"], "/tmp/copilot");
        assert_eq!(value["streaming"], true);
        assert_eq!(value["skillDirectories"][0], "skills");
        assert_eq!(value["disabledSkills"][0], "legacy_skill");
        assert_eq!(value["requestPermission"], true);
    }

    #[test]
    fn test_tool_builder() {
        let tool = Tool::new("my_tool")
            .description("A test tool")
            .schema(serde_json::json!({"type": "object"}));

        assert_eq!(tool.name, "my_tool");
        assert_eq!(tool.description, "A test tool");
    }

    #[test]
    fn test_tool_serializes_parameters_field() {
        let tool = Tool::new("my_tool")
            .description("A test tool")
            .schema(serde_json::json!({"type": "object"}));

        let value = serde_json::to_value(&tool).unwrap();

        assert_eq!(value["parameters"]["type"], "object");
        assert!(value.get("parametersSchema").is_none());
    }

    #[test]
    fn test_user_input_request_roundtrip() {
        let req = UserInputRequest {
            question: "What color?".into(),
            choices: Some(vec!["red".into(), "blue".into()]),
            allow_freeform: Some(true),
        };
        let j = serde_json::to_value(&req).unwrap();
        assert_eq!(j["question"], "What color?");
        assert_eq!(j["choices"][0], "red");
        assert_eq!(j["allowFreeform"], true);

        let req2: UserInputRequest = serde_json::from_value(j).unwrap();
        assert_eq!(req2.question, "What color?");
    }

    #[test]
    fn test_user_input_response_roundtrip() {
        let resp = UserInputResponse {
            answer: "blue".into(),
            was_freeform: Some(true),
        };
        let j = serde_json::to_value(&resp).unwrap();
        assert_eq!(j["answer"], "blue");

        let resp2: UserInputResponse = serde_json::from_value(j).unwrap();
        assert_eq!(resp2.answer, "blue");
        assert_eq!(resp2.was_freeform, Some(true));
    }

    #[test]
    fn test_user_input_request_minimal() {
        let j = serde_json::json!({"question": "Yes or no?"});
        let req: UserInputRequest = serde_json::from_value(j).unwrap();
        assert_eq!(req.question, "Yes or no?");
        assert!(req.choices.is_none());
        assert!(req.allow_freeform.is_none());
    }

    #[test]
    fn test_session_lifecycle_event_from_json() {
        let j = serde_json::json!({
            "type": "session.created",
            "sessionId": "sess_123",
            "metadata": {
                "startTime": "2024-01-15T10:30:00Z",
                "modifiedTime": "2024-01-15T10:30:00Z",
                "summary": "Test session"
            }
        });
        let event: SessionLifecycleEvent = serde_json::from_value(j).unwrap();
        assert_eq!(event.event_type, session_lifecycle_event_types::CREATED);
        assert_eq!(event.session_id, "sess_123");
        assert_eq!(
            event.metadata.as_ref().unwrap().summary,
            Some("Test session".into())
        );
    }

    #[test]
    fn test_get_foreground_session_response() {
        let j = serde_json::json!({"sessionId": "sess_123", "workspacePath": "/tmp"});
        let resp: GetForegroundSessionResponse = serde_json::from_value(j).unwrap();
        assert_eq!(resp.session_id, Some("sess_123".into()));
        assert_eq!(resp.workspace_path, Some("/tmp".into()));
    }

    #[test]
    fn test_set_foreground_session_response() {
        let j = serde_json::json!({"success": true});
        let resp: SetForegroundSessionResponse = serde_json::from_value(j).unwrap();
        assert!(resp.success);
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_set_foreground_session_response_error() {
        let j = serde_json::json!({"success": false, "error": "not found"});
        let resp: SetForegroundSessionResponse = serde_json::from_value(j).unwrap();
        assert!(!resp.success);
        assert_eq!(resp.error, Some("not found".into()));
    }

    #[test]
    fn test_selection_attachment_roundtrip() {
        let att = SelectionAttachment {
            file_path: "src/main.rs".into(),
            display_name: "main.rs".into(),
            text: "fn main()".into(),
            selection: SelectionRange {
                start: SelectionPosition {
                    line: 1.0,
                    character: 0.0,
                },
                end: SelectionPosition {
                    line: 1.0,
                    character: 9.0,
                },
            },
        };
        let j = serde_json::to_value(&att).unwrap();
        assert_eq!(j["filePath"], "src/main.rs");
        assert_eq!(j["selection"]["start"]["line"], 1.0);
    }

    #[test]
    fn test_attachment_type_selection() {
        let j = serde_json::json!("selection");
        let at: AttachmentType = serde_json::from_value(j).unwrap();
        assert_eq!(at, AttachmentType::Selection);
    }

    #[test]
    fn test_stop_error_display() {
        let err = StopError {
            message: "timeout".into(),
            source: Some("rpc".into()),
        };
        assert_eq!(format!("{err}"), "timeout");
    }

    #[test]
    fn test_session_config_reasoning_effort() {
        let config = SessionConfig {
            reasoning_effort: Some("high".into()),
            ..Default::default()
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["reasoningEffort"], "high");
    }

    #[test]
    fn test_session_config_working_directory() {
        let config = SessionConfig {
            working_directory: Some("/home/user/project".into()),
            ..Default::default()
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["workingDirectory"], "/home/user/project");
    }

    #[test]
    fn test_resume_config_disable_resume() {
        let config = ResumeSessionConfig {
            disable_resume: true,
            ..Default::default()
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["disableResume"], true);
    }

    #[test]
    fn test_resume_config_model() {
        let config = ResumeSessionConfig {
            model: Some("gpt-4".into()),
            ..Default::default()
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["model"], "gpt-4");
    }

    #[test]
    fn test_session_hooks_has_any() {
        let hooks = SessionHooks::default();
        assert!(!hooks.has_any());

        let hooks = SessionHooks {
            on_pre_tool_use: Some(Arc::new(|_| PreToolUseHookOutput::default())),
            ..Default::default()
        };
        assert!(hooks.has_any());
    }

    #[test]
    fn test_session_hooks_debug() {
        let hooks = SessionHooks {
            on_pre_tool_use: Some(Arc::new(|_| PreToolUseHookOutput::default())),
            ..Default::default()
        };
        let debug = format!("{:?}", hooks);
        assert!(debug.contains("on_pre_tool_use: true"));
        assert!(debug.contains("on_post_tool_use: false"));
    }

    #[test]
    fn test_pre_tool_use_hook_input_serde() {
        let json = serde_json::json!({
            "timestamp": 1234567890,
            "cwd": "/tmp",
            "toolName": "my_tool",
            "toolArgs": {"key": "value"}
        });
        let input: PreToolUseHookInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.timestamp, 1234567890);
        assert_eq!(input.tool_name, "my_tool");
    }

    #[test]
    fn test_pre_tool_use_hook_output_serde() {
        let output = PreToolUseHookOutput {
            permission_decision: Some("allow".into()),
            additional_context: Some("context".into()),
            ..Default::default()
        };
        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["permissionDecision"], "allow");
        assert_eq!(json["additionalContext"], "context");
        assert!(json.get("suppressOutput").is_none());
    }

    #[test]
    fn test_session_end_hook_input_serde() {
        let json = serde_json::json!({
            "timestamp": 1234567890,
            "cwd": "/tmp",
            "reason": "complete",
            "finalMessage": "Done"
        });
        let input: SessionEndHookInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.reason, "complete");
        assert_eq!(input.final_message, Some("Done".into()));
    }

    #[test]
    fn test_error_occurred_hook_input_serde() {
        let json = serde_json::json!({
            "timestamp": 1234567890,
            "cwd": "/tmp",
            "error": "connection failed",
            "errorContext": "model_call",
            "recoverable": true
        });
        let input: ErrorOccurredHookInput = serde_json::from_value(json).unwrap();
        assert_eq!(input.error_context, "model_call");
        assert!(input.recoverable);
    }

    #[test]
    fn test_hooks_not_serialized_in_config() {
        let config = SessionConfig {
            hooks: Some(SessionHooks {
                on_pre_tool_use: Some(Arc::new(|_| PreToolUseHookOutput::default())),
                ..Default::default()
            }),
            ..Default::default()
        };
        let json = serde_json::to_value(&config).unwrap();
        // hooks field should be skipped from serialization
        assert!(json.get("hooks").is_none());
    }

    #[test]
    fn test_session_mode_serialization() {
        assert_eq!(
            serde_json::to_string(&SessionMode::Interactive).unwrap(),
            "\"interactive\""
        );
        assert_eq!(
            serde_json::to_string(&SessionMode::Plan).unwrap(),
            "\"plan\""
        );
        assert_eq!(
            serde_json::to_string(&SessionMode::Autopilot).unwrap(),
            "\"autopilot\""
        );
    }

    #[test]
    fn test_shell_signal_serialization() {
        assert_eq!(
            serde_json::to_string(&ShellSignal::SIGINT).unwrap(),
            "\"SIGINT\""
        );
        assert_eq!(
            serde_json::to_string(&ShellSignal::SIGKILL).unwrap(),
            "\"SIGKILL\""
        );
    }

    #[test]
    fn test_tool_overrides_fields() {
        let tool = Tool::new("test")
            .overrides_built_in_tool(true)
            .skip_permission(true);
        let value = serde_json::to_value(&tool).unwrap();
        assert_eq!(value["overridesBuiltInTool"], true);
        assert_eq!(value["skipPermission"], true);
    }

    #[test]
    fn test_tool_overrides_fields_default() {
        let tool = Tool::new("test");
        let value = serde_json::to_value(&tool).unwrap();
        // Should not serialize false values
        assert!(value.get("overridesBuiltInTool").is_none());
        assert!(value.get("skipPermission").is_none());
    }

    #[test]
    fn test_session_config_new_fields() {
        let config = SessionConfig {
            client_name: Some("my-app".into()),
            agent: Some("code-reviewer".into()),
            ..Default::default()
        };
        let value = serde_json::to_value(&config).unwrap();
        assert_eq!(value["clientName"], "my-app");
        assert_eq!(value["agent"], "code-reviewer");
    }

    #[test]
    fn test_log_options_serialization() {
        let opts = LogOptions {
            level: Some(SessionLogLevel::Warning),
            ephemeral: Some(true),
        };
        let value = serde_json::to_value(&opts).unwrap();
        assert_eq!(value["level"], "warning");
        assert_eq!(value["ephemeral"], true);
    }

    #[test]
    fn test_plan_data_serialization() {
        let plan = PlanData {
            content: Some("Step 1: Do things".into()),
            title: Some("My Plan".into()),
        };
        let value = serde_json::to_value(&plan).unwrap();
        assert_eq!(value["content"], "Step 1: Do things");
        assert_eq!(value["title"], "My Plan");
    }

    #[test]
    fn test_agent_info_deserialization() {
        let json = serde_json::json!({
            "name": "reviewer",
            "displayName": "Code Reviewer",
            "description": "Reviews code"
        });
        let agent: AgentInfo = serde_json::from_value(json).unwrap();
        assert_eq!(agent.name, "reviewer");
        assert_eq!(agent.display_name, Some("Code Reviewer".into()));
    }

    #[test]
    fn test_telemetry_config_default() {
        let config = TelemetryConfig::default();
        assert!(config.otlp_endpoint.is_none());
        assert!(config.file_path.is_none());
        assert!(config.capture_content.is_none());
    }

    #[test]
    fn test_set_model_options_serialization() {
        let opts = SetModelOptions {
            reasoning_effort: Some("high".into()),
            ..Default::default()
        };
        let value = serde_json::to_value(&opts).unwrap();
        assert_eq!(value["reasoningEffort"], "high");
    }

    #[test]
    fn test_fleet_start_options_serialization() {
        let opts = FleetStartOptions {
            prompt: Some("Build and test the project".into()),
        };
        let value = serde_json::to_value(&opts).unwrap();
        assert_eq!(value["prompt"], "Build and test the project");
    }

    #[test]
    fn test_shell_exec_options_serialization() {
        let opts = ShellExecOptions {
            command: "ls -la".into(),
            cwd: Some("/tmp".into()),
            env: None,
        };
        let value = serde_json::to_value(&opts).unwrap();
        assert_eq!(value["command"], "ls -la");
        assert_eq!(value["cwd"], "/tmp");
    }

    #[test]
    fn test_quota_snapshot_deserialization() {
        let json = serde_json::json!({
            "type": "premium_requests",
            "limit": 1000,
            "used": 42,
            "remaining": 958,
            "resetsAt": "2026-04-01T00:00:00Z"
        });
        let quota: QuotaSnapshot = serde_json::from_value(json).unwrap();
        assert_eq!(quota.quota_type, "premium_requests");
        assert_eq!(quota.limit, Some(1000));
        assert_eq!(quota.used, Some(42));
    }

    #[test]
    fn test_workspace_file_deserialization() {
        let json = serde_json::json!({
            "path": "plan.md",
            "size": 1024,
            "modifiedAt": "2026-03-19T12:00:00Z"
        });
        let file: WorkspaceFile = serde_json::from_value(json).unwrap();
        assert_eq!(file.path, "plan.md");
        assert_eq!(file.size, Some(1024));
    }

    #[test]
    fn test_resume_session_config_new_fields() {
        let config = ResumeSessionConfig {
            client_name: Some("my-cli".into()),
            agent: Some("helper".into()),
            ..Default::default()
        };
        let value = serde_json::to_value(&config).unwrap();
        assert_eq!(value["clientName"], "my-cli");
        assert_eq!(value["agent"], "helper");
    }

    // =========================================================================
    // Client mode / connection kind
    // =========================================================================

    #[test]
    fn test_copilot_client_mode_wire_names() {
        assert_eq!(
            serde_json::to_value(CopilotClientMode::CopilotCli).unwrap(),
            serde_json::json!("copilot-cli")
        );
        assert_eq!(
            serde_json::to_value(CopilotClientMode::Empty).unwrap(),
            serde_json::json!("empty")
        );
        assert_eq!(CopilotClientMode::default(), CopilotClientMode::CopilotCli);
    }

    #[test]
    fn test_connection_kind_default_is_child() {
        assert_eq!(ConnectionKind::default(), ConnectionKind::Child);
    }

    #[test]
    fn test_client_options_defaults_for_new_fields() {
        let options = ClientOptions::default();
        assert_eq!(options.mode, CopilotClientMode::CopilotCli);
        assert_eq!(options.connection_kind, ConnectionKind::Child);
        assert!(options.session_fs.is_none());
        assert!(options.on_get_trace_context.is_none());
    }

    // =========================================================================
    // ResumeSessionConfig extensions
    // =========================================================================

    #[test]
    fn test_resume_session_config_omits_unset_extensions() {
        let value = serde_json::to_value(ResumeSessionConfig::default()).unwrap();
        assert!(value.get("suppressResumeEvent").is_none());
        assert!(value.get("systemMessage").is_none());
    }

    #[test]
    fn test_resume_session_config_serializes_extensions() {
        let config = ResumeSessionConfig {
            suppress_resume_event: Some(true),
            system_message: Some(SystemMessageConfig {
                mode: Some(SystemMessageMode::Customize),
                ..Default::default()
            }),
            ..Default::default()
        };
        let value = serde_json::to_value(&config).unwrap();
        assert_eq!(value["suppressResumeEvent"], true);
        assert_eq!(value["systemMessage"]["mode"], "customize");
    }

    // =========================================================================
    // Section transform overrides
    // =========================================================================

    #[test]
    fn test_section_override_action_transform_wire_name() {
        assert_eq!(
            serde_json::to_value(SectionOverrideAction::Transform).unwrap(),
            serde_json::json!("transform")
        );
    }

    #[tokio::test]
    async fn test_section_override_transform_fn_is_callable() {
        let over =
            SectionOverride::transform(|content| Box::pin(async move { content.to_uppercase() }));
        let f = over.transform_fn().expect("transform fn");
        assert_eq!(f("abc".to_string()).await, "ABC");
    }

    #[test]
    fn test_section_override_debug_hides_callback() {
        let over = SectionOverride::transform(|content| Box::pin(async move { content }));
        let debug = format!("{over:?}");
        assert!(debug.contains("Transform"));
    }
    // ---- Wave 3: session config feature flags / capability overrides ----

    #[test]
    fn test_reasoning_summary_wire_names() {
        for (v, w) in [
            (ReasoningSummary::None, "none"),
            (ReasoningSummary::Concise, "concise"),
            (ReasoningSummary::Detailed, "detailed"),
        ] {
            assert_eq!(serde_json::to_value(v).unwrap(), serde_json::json!(w));
        }
    }

    #[test]
    fn test_remote_session_mode_wire_names() {
        for (v, w) in [
            (RemoteSessionMode::Off, "off"),
            (RemoteSessionMode::Export, "export"),
            (RemoteSessionMode::On, "on"),
        ] {
            assert_eq!(serde_json::to_value(v).unwrap(), serde_json::json!(w));
        }
    }

    #[test]
    fn test_storage_mode_wire_names() {
        assert_eq!(
            serde_json::to_value(StorageMode::Persistent).unwrap(),
            serde_json::json!("persistent")
        );
        assert_eq!(
            serde_json::to_value(StorageMode::InMemory).unwrap(),
            serde_json::json!("in-memory")
        );
    }

    #[test]
    fn test_model_capabilities_override_limits_use_snake_case() {
        let caps = ModelCapabilitiesOverride {
            limits: Some(ModelCapabilitiesOverrideLimits {
                max_prompt_tokens: Some(1000),
                max_output_tokens: Some(200),
                max_context_window_tokens: Some(1200),
                vision: Some(ModelCapabilitiesOverrideLimitsVision {
                    supported_media_types: Some(vec!["image/png".into()]),
                    max_prompt_images: Some(3),
                    max_prompt_image_size: Some(4096),
                }),
            }),
            supports: Some(ModelCapabilitiesOverrideSupports {
                vision: Some(true),
                reasoning_effort: Some(true),
            }),
        };
        let v = serde_json::to_value(&caps).unwrap();
        assert_eq!(v["limits"]["max_prompt_tokens"], 1000);
        assert_eq!(v["limits"]["max_output_tokens"], 200);
        assert_eq!(v["limits"]["max_context_window_tokens"], 1200);
        assert_eq!(
            v["limits"]["vision"]["supported_media_types"][0],
            "image/png"
        );
        assert_eq!(v["limits"]["vision"]["max_prompt_images"], 3);
        assert_eq!(v["limits"]["vision"]["max_prompt_image_size"], 4096);
        // supports uses camelCase, unlike limits
        assert_eq!(v["supports"]["reasoningEffort"], true);
        assert_eq!(v["supports"]["vision"], true);
    }

    #[test]
    fn test_large_tool_output_config_uses_output_dir() {
        let cfg = LargeToolOutputConfig {
            output_directory: Some("/tmp/out".into()),
            ..Default::default()
        };
        let v = serde_json::to_value(&cfg).unwrap();
        assert_eq!(v["outputDir"], "/tmp/out");
        assert!(v.get("outputDirectory").is_none());
    }

    #[test]
    fn test_session_config_wave3_wire_names() {
        let cfg = SessionConfig {
            github_token: Some("tok".into()),
            enable_mcp_apps: Some(true),
            reasoning_summary: Some(ReasoningSummary::Concise),
            default_agent: Some(DefaultAgentConfig {
                excluded_tools: Some(vec!["shell".into()]),
            }),
            remote_session: Some(RemoteSessionMode::Export),
            mcp_oauth_token_storage: Some(StorageMode::InMemory),
            embedding_cache_storage: Some(StorageMode::Persistent),
            cloud: Some(CloudSessionOptions {
                repository: Some(CloudSessionRepository {
                    owner: "octo".into(),
                    name: "repo".into(),
                    branch: None,
                }),
            }),
            ..Default::default()
        };
        let v = serde_json::to_value(&cfg).unwrap();
        assert_eq!(v["gitHubToken"], "tok");
        assert_eq!(v["requestMcpApps"], true);
        assert_eq!(v["reasoningSummary"], "concise");
        assert_eq!(v["defaultAgent"]["excludedTools"][0], "shell");
        assert_eq!(v["remoteSession"], "export");
        assert_eq!(v["mcpOAuthTokenStorage"], "in-memory");
        assert_eq!(v["embeddingCacheStorage"], "persistent");
        assert_eq!(v["cloud"]["repository"]["owner"], "octo");
    }

    #[test]
    fn test_session_update_options_is_empty() {
        assert!(SessionUpdateOptions::default().is_empty());
        let patch = SessionUpdateOptions {
            coauthor_enabled: Some(false),
            ..Default::default()
        };
        assert!(!patch.is_empty());
        let v = serde_json::to_value(&patch).unwrap();
        assert_eq!(v.as_object().unwrap().len(), 1);
        assert_eq!(v["coauthorEnabled"], false);
    }

    #[test]
    fn test_session_update_options_wire_names() {
        let patch = SessionUpdateOptions {
            skip_custom_instructions: Some(true),
            custom_agents_local_only: Some(true),
            manage_schedule_enabled: Some(false),
            installed_plugins: Some(Vec::new()),
            enable_skills: Some(false),
            ..Default::default()
        };
        let v = serde_json::to_value(&patch).unwrap();
        assert_eq!(v["skipCustomInstructions"], true);
        assert_eq!(v["customAgentsLocalOnly"], true);
        assert_eq!(v["manageScheduleEnabled"], false);
        assert_eq!(v["installedPlugins"], serde_json::json!([]));
        assert_eq!(v["enableSkills"], false);
    }

    #[test]
    fn test_set_model_options_serializes_overrides() {
        let opts = SetModelOptions {
            reasoning_effort: Some("high".into()),
            reasoning_summary: Some(ReasoningSummary::Detailed),
            model_capabilities: Some(ModelCapabilitiesOverride {
                supports: Some(ModelCapabilitiesOverrideSupports {
                    vision: Some(false),
                    ..Default::default()
                }),
                ..Default::default()
            }),
        };
        let v = serde_json::to_value(&opts).unwrap();
        assert_eq!(v["reasoningEffort"], "high");
        assert_eq!(v["reasoningSummary"], "detailed");
        assert_eq!(v["modelCapabilities"]["supports"]["vision"], false);
    }

    #[test]
    fn test_set_model_options_default_is_empty_object() {
        let v = serde_json::to_value(SetModelOptions::default()).unwrap();
        assert_eq!(v, serde_json::json!({}));
    }

    #[test]
    fn test_send_attachment_directory_roundtrip() {
        let attachment = SendAttachment::Directory {
            path: "C:/repo/src".into(),
            display_name: "src".into(),
        };
        assert_type_tag_roundtrip(attachment, "directory");
    }

    #[test]
    fn test_send_attachment_github_reference_roundtrip() {
        let attachment = SendAttachment::GithubReference {
            number: 42,
            title: "Fix type coverage".into(),
            reference_type: SendAttachmentGithubReferenceType::Pr,
            state: "open".into(),
            url: "https://github.com/copilot-community-sdk/copilot-sdk-rust/pull/42".into(),
        };
        assert_type_tag_roundtrip(attachment, "github_reference");
    }

    #[test]
    fn test_send_attachment_blob_roundtrip() {
        let attachment = SendAttachment::Blob {
            data: "aGVsbG8=".into(),
            mime_type: "text/plain".into(),
            display_name: Some("hello.txt".into()),
        };
        assert_type_tag_roundtrip(attachment, "blob");
    }

    #[test]
    fn test_external_tool_terminal_roundtrip() {
        let content = ExternalToolTextResultForLlmContent::Terminal {
            text: "cargo test".into(),
            exit_code: Some(0),
            cwd: Some("E:/copilot-sdk-rust".into()),
        };
        assert_type_tag_roundtrip(content, "terminal");
    }

    #[test]
    fn test_external_tool_image_roundtrip() {
        let content = ExternalToolTextResultForLlmContent::Image {
            data: "iVBORw0KGgo=".into(),
            mime_type: "image/png".into(),
        };
        assert_type_tag_roundtrip(content, "image");
    }

    #[test]
    fn test_external_tool_audio_roundtrip() {
        let content = ExternalToolTextResultForLlmContent::Audio {
            data: "UklGRg==".into(),
            mime_type: "audio/wav".into(),
        };
        assert_type_tag_roundtrip(content, "audio");
    }

    #[test]
    fn test_external_tool_resource_link_roundtrip() {
        let content = ExternalToolTextResultForLlmContent::ResourceLink {
            icons: Some(vec![ExternalToolTextResultForLlmContentResourceLinkIcon {
                src: "https://example.com/icon.png".into(),
                mime_type: Some("image/png".into()),
                sizes: Some(vec!["16x16".into()]),
                theme: Some(ExternalToolTextResultForLlmContentResourceLinkIconTheme::Dark),
            }]),
            name: "artifact".into(),
            title: Some("Build Artifact".into()),
            uri: "artifact://build/123".into(),
            description: Some("Compiled binary".into()),
            mime_type: Some("application/octet-stream".into()),
            size: Some(512),
        };
        assert_type_tag_roundtrip(content, "resource_link");
    }

    #[test]
    fn test_external_tool_resource_roundtrip() {
        let content = ExternalToolTextResultForLlmContent::Resource {
            resource: ExternalToolTextResultForLlmContentResourceDetails::Text(
                EmbeddedTextResourceContents {
                    uri: "file:///tmp/output.txt".into(),
                    mime_type: Some("text/plain".into()),
                    text: "done".into(),
                },
            ),
        };
        assert_type_tag_roundtrip(content, "resource");
    }

    #[test]
    fn test_auth_info_hmac_roundtrip() {
        let auth = AuthInfo::Hmac {
            host: "https://github.com".into(),
            hmac: "secret".into(),
            copilot_user: None,
        };
        assert_type_tag_roundtrip(auth, "hmac");
    }

    #[test]
    fn test_auth_info_env_roundtrip() {
        let auth = AuthInfo::Env {
            host: "https://github.example.com".into(),
            login: Some("octocat".into()),
            token: "ghp_example".into(),
            env_var: "GITHUB_TOKEN".into(),
            copilot_user: None,
        };
        assert_type_tag_roundtrip(auth, "env");
    }

    #[test]
    fn test_auth_info_token_roundtrip() {
        let auth = AuthInfo::Token {
            host: "https://github.com".into(),
            token: "ghp_direct".into(),
            copilot_user: None,
        };
        assert_type_tag_roundtrip(auth, "token");
    }

    #[test]
    fn test_ui_elicitation_array_any_of_items_roundtrip() {
        let schema = UIElicitationSchema {
            schema_type: "object".into(),
            properties: HashMap::from([(
                "targets".into(),
                UIElicitationSchemaProperty::ArrayAnyOf(UIElicitationArrayAnyOfField {
                    field_type: "array".into(),
                    title: Some("Targets".into()),
                    description: Some("Choose one or more targets.".into()),
                    min_items: Some(1),
                    max_items: Some(2),
                    items: UIElicitationArrayAnyOfFieldItems {
                        any_of: vec![
                            UIElicitationArrayAnyOfFieldOption {
                                r#const: "linux".into(),
                                title: "Linux".into(),
                            },
                            UIElicitationArrayAnyOfFieldOption {
                                r#const: "windows".into(),
                                title: "Windows".into(),
                            },
                        ],
                    },
                    default: Some(vec!["linux".into()]),
                }),
            )]),
            required: Some(vec!["targets".into()]),
        };

        let json = serde_json::to_value(&schema).unwrap();
        assert_eq!(json["properties"]["targets"]["type"], "array");
        assert_eq!(
            json["properties"]["targets"]["items"]["anyOf"][0]["const"],
            "linux"
        );
        assert_eq!(
            json["properties"]["targets"]["items"]["anyOf"][0]["title"],
            "Linux"
        );

        let decoded: UIElicitationSchema = serde_json::from_value(json).unwrap();
        assert_eq!(decoded, schema);
    }
}
