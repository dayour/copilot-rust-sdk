// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Skill, extension, plugin, and command RPC bindings.

use crate::{Result, Session, SessionMode};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Accessor for `session.skills.*` RPC methods.
pub struct SessionSkills<'a> {
    session: &'a Session,
}

/// Accessor for `session.extensions.*` RPC methods.
pub struct SessionExtensions<'a> {
    session: &'a Session,
}

/// Accessor for `session.plugins.*` RPC methods.
pub struct SessionPlugins<'a> {
    session: &'a Session,
}

/// Accessor for `session.commands.*` RPC methods.
pub struct SessionCommands<'a> {
    session: &'a Session,
}

/// Optional filters for `session.commands.list`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommandsListRequest {
    /// Include runtime built-in commands in the listing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_builtins: Option<bool>,
    /// Include enabled user-invocable skills and commands in the listing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_skills: Option<bool>,
    /// Include commands registered by SDK clients and extensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_client_commands: Option<bool>,
}

/// Result for `session.commands.enqueue`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EnqueueCommandResult {
    /// Whether the command was accepted into the local execution queue.
    pub queued: bool,
}

/// Result for `session.commands.execute`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExecuteCommandResult {
    /// Error message produced while executing the command, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Result for `session.commands.list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommandList {
    /// Commands available in this session.
    pub commands: Vec<SlashCommandInfo>,
}

/// Coarse slash-command category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SlashCommandKind {
    /// Command implemented by the runtime.
    Builtin,
    /// Command backed by a skill.
    Skill,
    /// Command registered by an SDK client or extension.
    Client,
}

/// Completion hint for a slash-command input field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SlashCommandInputCompletion {
    /// Input should complete filesystem directories.
    Directory,
}

/// Optional unstructured input hint for a slash command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SlashCommandInput {
    /// Hint to display when command input has not been provided.
    pub hint: String,
    /// Whether the command requires non-empty input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    /// Optional completion hint for the input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<SlashCommandInputCompletion>,
    /// Whether clients should preserve multiline input as a single argument.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preserve_multiline_input: Option<bool>,
}

/// Slash-command metadata returned by `session.commands.list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SlashCommandInfo {
    /// Canonical command name without a leading slash.
    pub name: String,
    /// Canonical aliases without leading slashes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,
    /// Human-readable command description.
    pub description: String,
    /// Coarse command category for grouping and behavior.
    pub kind: SlashCommandKind,
    /// Optional unstructured input hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<SlashCommandInput>,
    /// Whether the command may run while an agent turn is active.
    pub allow_during_agent_execution: bool,
    /// Whether the command is experimental.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<bool>,
}

/// A subcommand option returned by `select-subcommand` slash-command results.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SlashCommandSelectSubcommandOption {
    /// Subcommand name to invoke.
    pub name: String,
    /// Human-readable description of the subcommand.
    pub description: String,
    /// Optional group label for organizing options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,
}

/// Result of invoking a slash command.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum SlashCommandInvocationResult {
    /// Text output for the client to render.
    #[serde(rename = "text")]
    Text {
        /// Text output for the client to render.
        text: String,
        /// Whether the text contains Markdown.
        #[serde(skip_serializing_if = "Option::is_none")]
        markdown: Option<bool>,
        /// Whether ANSI sequences should be preserved.
        #[serde(rename = "preserveAnsi", skip_serializing_if = "Option::is_none")]
        preserve_ansi: Option<bool>,
        /// Whether the invocation changed runtime settings.
        #[serde(
            rename = "runtimeSettingsChanged",
            skip_serializing_if = "Option::is_none"
        )]
        runtime_settings_changed: Option<bool>,
    },
    /// Prompt that should be sent to the agent.
    #[serde(rename = "agent-prompt")]
    AgentPrompt {
        /// Prompt to submit to the agent.
        prompt: String,
        /// Prompt text to display to the user.
        #[serde(rename = "displayPrompt")]
        display_prompt: String,
        /// Optional target session mode for the agent prompt.
        #[serde(skip_serializing_if = "Option::is_none")]
        mode: Option<SessionMode>,
        /// Whether the invocation changed runtime settings.
        #[serde(
            rename = "runtimeSettingsChanged",
            skip_serializing_if = "Option::is_none"
        )]
        runtime_settings_changed: Option<bool>,
    },
    /// Command completion result with an optional user-facing message.
    #[serde(rename = "completed")]
    Completed {
        /// Optional message describing the completed command.
        #[serde(skip_serializing_if = "Option::is_none")]
        message: Option<String>,
        /// Whether the invocation changed runtime settings.
        #[serde(
            rename = "runtimeSettingsChanged",
            skip_serializing_if = "Option::is_none"
        )]
        runtime_settings_changed: Option<bool>,
    },
    /// Request for the client to present subcommand choices.
    #[serde(rename = "select-subcommand")]
    SelectSubcommand {
        /// Parent command name that requires subcommand selection.
        command: String,
        /// Human-readable title for the selection UI.
        title: String,
        /// Available subcommand options for the client to present.
        options: Vec<SlashCommandSelectSubcommandOption>,
        /// Whether the invocation changed runtime settings.
        #[serde(
            rename = "runtimeSettingsChanged",
            skip_serializing_if = "Option::is_none"
        )]
        runtime_settings_changed: Option<bool>,
    },
}

/// Result of queued-command execution reported back to the runtime.
#[derive(Debug, Clone)]
pub enum QueuedCommandResult {
    /// The host executed the queued command.
    Handled {
        /// When true, the runtime stops processing subsequent queued commands.
        stop_processing_queue: Option<bool>,
    },
    /// The host did not execute the queued command.
    NotHandled,
}

impl QueuedCommandResult {
    /// Construct a handled queued-command result.
    pub fn handled(stop_processing_queue: Option<bool>) -> Self {
        Self::Handled {
            stop_processing_queue,
        }
    }

    /// Construct a not-handled queued-command result.
    pub fn not_handled() -> Self {
        Self::NotHandled
    }
}

impl Serialize for QueuedCommandResult {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;

        match self {
            Self::Handled {
                stop_processing_queue,
            } => {
                let mut state = serializer.serialize_struct(
                    "QueuedCommandHandled",
                    if stop_processing_queue.is_some() {
                        2
                    } else {
                        1
                    },
                )?;
                state.serialize_field("handled", &true)?;
                if let Some(stop_processing_queue) = stop_processing_queue {
                    state.serialize_field("stopProcessingQueue", stop_processing_queue)?;
                }
                state.end()
            }
            Self::NotHandled => {
                let mut state = serializer.serialize_struct("QueuedCommandNotHandled", 1)?;
                state.serialize_field("handled", &false)?;
                state.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for QueuedCommandResult {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase", deny_unknown_fields)]
        struct QueuedCommandResultWire {
            handled: bool,
            #[serde(skip_serializing_if = "Option::is_none")]
            stop_processing_queue: Option<bool>,
        }

        let wire = QueuedCommandResultWire::deserialize(deserializer)?;
        if wire.handled {
            Ok(Self::Handled {
                stop_processing_queue: wire.stop_processing_queue,
            })
        } else if wire.stop_processing_queue.is_some() {
            Err(serde::de::Error::custom(
                "stopProcessingQueue is only valid when handled is true",
            ))
        } else {
            Ok(Self::NotHandled)
        }
    }
}

/// Result for `session.commands.respondToQueuedCommand`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommandsRespondToQueuedCommandResult {
    /// Whether the queued-command response matched and resolved a pending request.
    pub success: bool,
}

/// Where a skill was loaded from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SkillSource {
    /// Skill defined in the current project's skill directories.
    Project,
    /// Skill inherited from a parent directory in the workspace tree.
    Inherited,
    /// Skill defined in the user's Copilot skill directory.
    PersonalCopilot,
    /// Skill defined in the user's personal agents skill directory.
    PersonalAgents,
    /// Skill provided by an installed plugin.
    Plugin,
    /// Skill loaded from a configured custom skill directory.
    Custom,
    /// Skill bundled with the runtime.
    Builtin,
}

/// A skill discovered for the active session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Skill {
    /// Unique identifier for the skill.
    pub name: String,
    /// Description of what the skill does.
    pub description: String,
    /// Source location type for the skill.
    pub source: SkillSource,
    /// Whether the skill can be invoked by the user as a slash command.
    pub user_invocable: bool,
    /// Whether the skill is currently enabled.
    pub enabled: bool,
    /// Absolute path to the skill file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Name of the plugin that provides the skill, when applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_name: Option<String>,
}

/// Result for `session.skills.list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillList {
    /// Skills available to the session.
    pub skills: Vec<Skill>,
}

/// A skill that has been invoked during the active session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillsInvokedSkill {
    /// Unique identifier for the skill.
    pub name: String,
    /// Path to the `SKILL.md` file.
    pub path: String,
    /// Full content of the skill file.
    pub content: String,
    /// Tools auto-approved while the skill was active, if any.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<Vec<String>>,
    /// Turn number when the skill was invoked.
    pub invoked_at_turn: u64,
}

/// Result for `session.skills.getInvoked`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillsGetInvokedResult {
    /// Skills invoked during this session, ordered by invocation time.
    pub skills: Vec<SkillsInvokedSkill>,
}

/// Diagnostics returned by `session.skills.reload`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillsLoadDiagnostics {
    /// Warnings emitted while loading skills.
    pub warnings: Vec<String>,
    /// Errors emitted while loading skills.
    pub errors: Vec<String>,
}

/// Where an extension was discovered from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtensionSource {
    /// Extension discovered from the current project's `.github/extensions` directory.
    Project,
    /// Extension discovered from the user's `~/.copilot/extensions` directory.
    User,
}

/// Runtime status of an extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtensionStatus {
    /// The extension process is running.
    Running,
    /// The extension is installed but disabled.
    Disabled,
    /// The extension failed to start or crashed.
    Failed,
    /// The extension process is starting.
    Starting,
}

/// An extension discovered for the active session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Extension {
    /// Source-qualified extension identifier.
    pub id: String,
    /// Extension name.
    pub name: String,
    /// Discovery source for the extension.
    pub source: ExtensionSource,
    /// Current runtime status.
    pub status: ExtensionStatus,
    /// Process ID if the extension is running.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
}

/// Result for `session.extensions.list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtensionList {
    /// Discovered extensions and their current status.
    pub extensions: Vec<Extension>,
}

/// A plugin installed for the active session.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Plugin {
    /// Plugin name.
    pub name: String,
    /// Marketplace the plugin came from.
    pub marketplace: String,
    /// Installed version, when reported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Whether the plugin is currently enabled.
    pub enabled: bool,
}

/// Result for `session.plugins.list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PluginList {
    /// Installed plugins.
    pub plugins: Vec<Plugin>,
}

impl Session {
    /// Access skill discovery and management APIs.
    pub fn skills(&self) -> SessionSkills<'_> {
        SessionSkills { session: self }
    }

    /// Access extension discovery and management APIs.
    pub fn extensions(&self) -> SessionExtensions<'_> {
        SessionExtensions { session: self }
    }

    /// Access plugin discovery APIs.
    pub fn plugins(&self) -> SessionPlugins<'_> {
        SessionPlugins { session: self }
    }

    /// Access slash-command discovery and invocation APIs.
    pub fn commands(&self) -> SessionCommands<'_> {
        SessionCommands { session: self }
    }
}

impl SessionSkills<'_> {
    /// List skills available to the session.
    pub async fn list(&self) -> Result<SkillList> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result = (self.session.invoke_fn)("session.skills.list", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Enable a skill for the session.
    pub async fn enable(&self, name: &str) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "name": name,
        });
        (self.session.invoke_fn)("session.skills.enable", Some(params)).await?;
        Ok(())
    }

    /// Disable a skill for the session.
    pub async fn disable(&self, name: &str) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "name": name,
        });
        (self.session.invoke_fn)("session.skills.disable", Some(params)).await?;
        Ok(())
    }

    /// Ensure the session's skill definitions have been loaded from disk.
    pub async fn ensure_loaded(&self) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        (self.session.invoke_fn)("session.skills.ensureLoaded", Some(params)).await?;
        Ok(())
    }

    /// Return the skills invoked during this session.
    pub async fn get_invoked(&self) -> Result<SkillsGetInvokedResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result = (self.session.invoke_fn)("session.skills.getInvoked", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Reload skill definitions for the session.
    pub async fn reload(&self) -> Result<SkillsLoadDiagnostics> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result = (self.session.invoke_fn)("session.skills.reload", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionExtensions<'_> {
    /// List extensions discovered for the session.
    pub async fn list(&self) -> Result<ExtensionList> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result = (self.session.invoke_fn)("session.extensions.list", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Enable an extension for the session.
    pub async fn enable(&self, id: &str) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "id": id,
        });
        (self.session.invoke_fn)("session.extensions.enable", Some(params)).await?;
        Ok(())
    }

    /// Disable an extension for the session.
    pub async fn disable(&self, id: &str) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "id": id,
        });
        (self.session.invoke_fn)("session.extensions.disable", Some(params)).await?;
        Ok(())
    }

    /// Reload extension definitions and processes for the session.
    pub async fn reload(&self) -> Result<()> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        (self.session.invoke_fn)("session.extensions.reload", Some(params)).await?;
        Ok(())
    }
}

impl SessionPlugins<'_> {
    /// List plugins installed for the session.
    pub async fn list(&self) -> Result<PluginList> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        let result = (self.session.invoke_fn)("session.plugins.list", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }
}

impl SessionCommands<'_> {
    /// Enqueue a slash command for FIFO processing on the local session.
    pub async fn enqueue(&self, command: &str) -> Result<EnqueueCommandResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "command": command,
        });
        let result = (self.session.invoke_fn)("session.commands.enqueue", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Execute a slash command synchronously.
    pub async fn execute(&self, command_name: &str, args: &str) -> Result<ExecuteCommandResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "commandName": command_name,
            "args": args,
        });
        let result = (self.session.invoke_fn)("session.commands.execute", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Invoke a slash command in the session.
    pub async fn invoke(
        &self,
        name: &str,
        input: Option<&str>,
    ) -> Result<SlashCommandInvocationResult> {
        let mut params = serde_json::json!({
            "sessionId": self.session.session_id,
            "name": name,
        });
        if let Some(input) = input {
            params["input"] = serde_json::json!(input);
        }
        let result = (self.session.invoke_fn)("session.commands.invoke", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// List slash commands available in the session.
    pub async fn list(&self, request: Option<CommandsListRequest>) -> Result<CommandList> {
        let mut params = serde_json::json!({
            "sessionId": self.session.session_id,
        });
        if let Some(request) = request {
            if let Some(include_builtins) = request.include_builtins {
                params["includeBuiltins"] = serde_json::json!(include_builtins);
            }
            if let Some(include_skills) = request.include_skills {
                params["includeSkills"] = serde_json::json!(include_skills);
            }
            if let Some(include_client_commands) = request.include_client_commands {
                params["includeClientCommands"] = serde_json::json!(include_client_commands);
            }
        }
        let result = (self.session.invoke_fn)("session.commands.list", Some(params)).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Report whether the host executed a queued command and whether processing should continue.
    pub async fn respond_to_queued_command(
        &self,
        request_id: &str,
        result: QueuedCommandResult,
    ) -> Result<CommandsRespondToQueuedCommandResult> {
        let params = serde_json::json!({
            "sessionId": self.session.session_id,
            "requestId": request_id,
            "result": result,
        });
        let result =
            (self.session.invoke_fn)("session.commands.respondToQueuedCommand", Some(params))
                .await?;
        Ok(serde_json::from_value(result)?)
    }
}
