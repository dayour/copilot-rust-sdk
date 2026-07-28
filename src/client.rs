// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Copilot client for managing connections and sessions.
//!
//! The `Client` is the main entry point for the SDK.

use crate::error::{CopilotError, Result};
use crate::events::SessionEvent;
use crate::jsonrpc::{JsonRpcClient, StdioJsonRpcClient, TcpJsonRpcClient};
use crate::process::{CopilotProcess, ProcessOptions, ResolvedCopilotCli};
use crate::session::Session;
use crate::transport::ParentStdioTransport;
use crate::types::{
    ClientOptions, ConnectionKind, ConnectionState, CopilotClientMode, GetAuthStatusResponse,
    GetForegroundSessionResponse, GetStatusResponse, LogLevel, ModelInfo, PingResponse,
    ProviderConfig, QuotaResult, ResumeSessionConfig, SessionConfig, SessionHooks,
    SessionLifecycleEvent, SessionMetadata, SessionUpdateOptions, SetForegroundSessionResponse,
    StopError, TelemetryConfig, ToolsListResult, MIN_PROTOCOL_VERSION, SDK_PROTOCOL_VERSION,
};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::{Mutex, RwLock};

// =============================================================================
// Helper Functions
// =============================================================================

/// Resolve CLI command for the current platform.
///
/// On Windows, .cmd/.bat files are npm wrappers that need special handling.
/// We resolve them to their underlying node.js scripts for proper pipe handling.
fn resolve_native_cli_command(cli_path: &Path, args: &[String]) -> (PathBuf, Vec<String>) {
    let path = cli_path.to_path_buf();
    let args_owned = args.to_vec();

    // Check if it's a Node.js script - run directly via node
    if crate::process::is_node_script(&path) {
        if let Some(node_path) = crate::process::find_node() {
            let mut full_args = vec![path.to_string_lossy().to_string()];
            full_args.extend(args_owned);
            return (node_path, full_args);
        }
    }

    #[cfg(windows)]
    {
        // On Windows, .cmd files are npm wrapper scripts that launch node.
        // Running them through cmd.exe causes pipe inheritance issues.
        // Instead, we find the underlying node.js script and run it directly.
        if let Some(ext) = path.extension() {
            let ext_lower = ext.to_string_lossy().to_lowercase();
            if ext_lower == "cmd" {
                // npm .cmd files have a corresponding node_modules structure
                // e.g., C:\Users\...\npm\copilot.cmd -> C:\Users\...\npm\node_modules\@github\copilot\npm-loader.js
                if let Some(parent) = path.parent() {
                    // Extract the package name from the .cmd filename
                    if let Some(stem) = path.file_stem() {
                        let stem_str = stem.to_string_lossy();

                        // Try to find the npm-loader.js in node_modules
                        // Common patterns: copilot -> @github/copilot, or package-name -> package-name
                        let possible_paths = vec![
                            parent
                                .join("node_modules/@github")
                                .join(&*stem_str)
                                .join("npm-loader.js"),
                            parent
                                .join("node_modules")
                                .join(&*stem_str)
                                .join("npm-loader.js"),
                            parent
                                .join("node_modules/@github")
                                .join(&*stem_str)
                                .join("index.js"),
                            parent
                                .join("node_modules")
                                .join(&*stem_str)
                                .join("index.js"),
                        ];

                        for loader_path in possible_paths {
                            if loader_path.exists() {
                                if let Some(node_path) = crate::process::find_node() {
                                    let mut full_args =
                                        vec![loader_path.to_string_lossy().to_string()];
                                    full_args.extend(args_owned);
                                    return (node_path, full_args);
                                }
                            }
                        }
                    }
                }

                // Fallback: use cmd /c if we can't find the loader
                let mut full_args = vec!["/c".to_string(), path.to_string_lossy().to_string()];
                full_args.extend(args_owned);
                return (PathBuf::from("cmd"), full_args);
            }

            // For .bat files, use cmd /c
            if ext_lower == "bat" {
                let mut full_args = vec!["/c".to_string(), path.to_string_lossy().to_string()];
                full_args.extend(args_owned);
                return (PathBuf::from("cmd"), full_args);
            }
        }

        // For non-absolute paths without extension, also use cmd /c for PATH resolution
        if !path.is_absolute() {
            let mut full_args = vec!["/c".to_string(), path.to_string_lossy().to_string()];
            full_args.extend(args_owned);
            return (PathBuf::from("cmd"), full_args);
        }
    }

    (path, args_owned)
}

fn resolve_cli_command(cli: &ResolvedCopilotCli, args: &[String]) -> (PathBuf, Vec<String>) {
    match cli {
        ResolvedCopilotCli::NativeExecutable(path) => resolve_native_cli_command(path, args),
        ResolvedCopilotCli::NodeScript {
            node_executable,
            script_path,
        } => {
            let mut full_args = vec![script_path.to_string_lossy().to_string()];
            full_args.extend(args.iter().cloned());
            (node_executable.clone(), full_args)
        }
    }
}

fn spawn_cli_stderr_logger(stderr: tokio::process::ChildStderr) {
    tokio::spawn(async move {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            tracing::debug!(target: "copilot_sdk::cli_stderr", "{line}");
        }
    });
}

/// Handler for client-level lifecycle events (session created, deleted, etc.).
pub type LifecycleHandler = Arc<dyn Fn(&SessionLifecycleEvent) + Send + Sync>;

/// Handle a tool.call request from the server.
async fn handle_tool_call(
    sessions: &RwLock<HashMap<String, Arc<Session>>>,
    params: &Value,
) -> Result<Value> {
    let session_id = params
        .get("sessionId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CopilotError::InvalidConfig("Missing sessionId".into()))?;

    let tool_name = params
        .get("toolName")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CopilotError::InvalidConfig("Missing toolName".into()))?;

    let arguments = normalize_tool_arguments(params);

    let session = sessions.read().await.get(session_id).cloned();

    let session = match session {
        Some(s) => s,
        None => {
            return Ok(json!({
                "result": {
                    "textResultForLlm": "Session not found",
                    "resultType": "failure",
                    "error": format!("Unknown session {}", session_id)
                }
            }));
        }
    };

    // Check if tool is registered
    if session.get_tool(tool_name).await.is_none() {
        return Ok(json!({
            "result": {
                "textResultForLlm": format!("Tool '{}' is not supported.", tool_name),
                "resultType": "failure",
                "error": format!("tool '{}' not supported", tool_name)
            }
        }));
    }

    // Invoke the tool handler
    match session.invoke_tool(tool_name, &arguments).await {
        Ok(result) => Ok(json!({ "result": result })),
        Err(e) => Ok(json!({
            "result": {
                "textResultForLlm": "Tool execution failed",
                "resultType": "failure",
                "error": e.to_string()
            }
        })),
    }
}

fn normalize_tool_arguments(params: &Value) -> Value {
    let raw = params
        .get("arguments")
        .or_else(|| params.get("argumentsJson"))
        .cloned()
        .unwrap_or(json!({}));

    match raw {
        Value::String(s) => serde_json::from_str(&s).unwrap_or(json!({})),
        Value::Null => json!({}),
        other => other,
    }
}

/// Handle a permission.request from the server.
async fn handle_permission_request(
    sessions: &RwLock<HashMap<String, Arc<Session>>>,
    params: &Value,
) -> Result<Value> {
    let session_id = params
        .get("sessionId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CopilotError::InvalidConfig("Missing sessionId".into()))?;

    // Permission request data may be nested in "permissionRequest" field
    let perm_data = params.get("permissionRequest").unwrap_or(params);

    let session = sessions.read().await.get(session_id).cloned();

    let session = match session {
        Some(s) => s,
        None => {
            // Default deny on unknown session
            return Ok(json!({
                "result": {
                    "kind": "denied-no-approval-rule-and-could-not-request-from-user"
                }
            }));
        }
    };

    // Build permission request
    use crate::types::PermissionRequest;
    let kind = perm_data
        .get("kind")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let tool_call_id = perm_data
        .get("toolCallId")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Collect extension data
    let mut extension_data = HashMap::new();
    if let Some(obj) = perm_data.as_object() {
        for (key, value) in obj {
            if key != "kind" && key != "toolCallId" {
                extension_data.insert(key.clone(), value.clone());
            }
        }
    }

    let request = PermissionRequest {
        kind,
        tool_call_id,
        extension_data,
    };

    let result = session.handle_permission_request(&request).await;

    // Build response
    let mut response = json!({
        "result": {
            "kind": result.kind
        }
    });

    if let Some(rules) = result.rules {
        response["result"]["rules"] = Value::Array(rules);
    }

    Ok(response)
}

/// Handle a userInput.request from the server.
async fn handle_user_input_request(
    sessions: &RwLock<HashMap<String, Arc<Session>>>,
    params: &Value,
) -> Result<Value> {
    let session_id = params
        .get("sessionId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CopilotError::InvalidConfig("Missing sessionId".into()))?;

    let session = sessions.read().await.get(session_id).cloned();

    let session = match session {
        Some(s) => s,
        None => {
            return Err(CopilotError::Protocol(format!(
                "Session not found for user input request: {session_id}"
            )));
        }
    };

    use crate::types::UserInputRequest;
    let request = UserInputRequest {
        question: params
            .get("question")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        choices: params.get("choices").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
        }),
        allow_freeform: params.get("allowFreeform").and_then(|v| v.as_bool()),
    };

    let response = session.handle_user_input_request(&request).await?;
    Ok(serde_json::to_value(response).unwrap_or(json!({})))
}

async fn handle_hooks_invoke(
    sessions: &RwLock<HashMap<String, Arc<Session>>>,
    params: &Value,
) -> Result<Value> {
    let session_id = params
        .get("sessionId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CopilotError::InvalidConfig("Missing sessionId".into()))?;

    let session = sessions.read().await.get(session_id).cloned();

    let session = match session {
        Some(s) => s,
        None => {
            return Err(CopilotError::Protocol(format!(
                "Session not found for hooks invoke: {session_id}"
            )));
        }
    };

    let hook_type = params
        .get("hookType")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let input = params.get("input").cloned().unwrap_or(Value::Null);

    session.handle_hooks_invoke(hook_type, &input).await
}

/// Handle an inbound `canvas.*` reverse-RPC request from the runtime.
async fn handle_canvas_request(
    sessions: &RwLock<HashMap<String, Arc<Session>>>,
    method: &str,
    params: &Value,
) -> Result<Value> {
    let session_id = params
        .get("sessionId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CopilotError::InvalidConfig("Missing sessionId".into()))?;

    let session = sessions
        .read()
        .await
        .get(session_id)
        .cloned()
        .ok_or_else(|| {
            CopilotError::Protocol(format!(
                "Session not found for canvas request: {session_id}"
            ))
        })?;

    let handler = session.canvas_handler().await.ok_or_else(|| {
        CopilotError::Protocol(
            "No CanvasHandler installed on this session; call \
             Session::register_canvas_handler before creating the session."
                .into(),
        )
    })?;

    let canvas_err = |e: crate::canvas::CanvasError| CopilotError::Protocol(e.to_string());

    match method {
        "canvas.open" => {
            let req = serde_json::from_value(params.clone())?;
            let result = handler.on_open(req).map_err(canvas_err)?;
            Ok(serde_json::to_value(result)?)
        }
        "canvas.close" => {
            let req = serde_json::from_value(params.clone())?;
            handler.on_close(req).map_err(canvas_err)?;
            Ok(Value::Null)
        }
        "canvas.action.invoke" => {
            let req = serde_json::from_value(params.clone())?;
            let result = handler.on_action(req).map_err(canvas_err)?;
            Ok(result)
        }
        _ => Err(CopilotError::Protocol(format!(
            "Unknown canvas method: {method}"
        ))),
    }
}

/// Handle an inbound `sessionFs.*` reverse-RPC request from the runtime.
async fn handle_session_fs_request(
    sessions: &RwLock<HashMap<String, Arc<Session>>>,
    method: &str,
    params: &Value,
) -> Result<Value> {
    use crate::session_fs::{SessionFsError, SessionFsSqliteParams, SessionFsSqliteQueryType};

    let session_id = params
        .get("sessionId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CopilotError::InvalidConfig("Missing sessionId".into()))?;

    let session = sessions
        .read()
        .await
        .get(session_id)
        .cloned()
        .ok_or_else(|| {
            CopilotError::Protocol(format!(
                "Session not found for sessionFs request: {session_id}"
            ))
        })?;

    let provider = session.session_fs_provider().await.ok_or_else(|| {
        CopilotError::Protocol(
            "No SessionFsProvider installed on this session; call \
             Session::register_session_fs_provider before creating the session."
                .into(),
        )
    })?;

    let str_arg = |name: &str| -> Result<String> {
        params
            .get(name)
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| CopilotError::InvalidConfig(format!("Missing {name} in {method}")))
    };
    let bool_arg = |name: &str| params.get(name).and_then(|v| v.as_bool()).unwrap_or(false);
    let mode_arg = || {
        params
            .get("mode")
            .and_then(|v| v.as_u64())
            .map(|m| m as u32)
    };

    // Void operations answer `null` on success and a bare `SessionFsError` on failure.
    fn void_result(outcome: std::result::Result<(), SessionFsError>) -> Result<Value> {
        match outcome {
            Ok(()) => Ok(Value::Null),
            Err(err) => Ok(serde_json::to_value(err)?),
        }
    }

    match method {
        "sessionFs.readFile" => {
            let path = str_arg("path")?;
            Ok(match provider.read_file(&path).await {
                Ok(content) => json!({ "content": content }),
                Err(err) => json!({ "content": "", "error": err }),
            })
        }
        "sessionFs.writeFile" => {
            let path = str_arg("path")?;
            let content = str_arg("content")?;
            void_result(provider.write_file(&path, &content, mode_arg()).await)
        }
        "sessionFs.appendFile" => {
            let path = str_arg("path")?;
            let content = str_arg("content")?;
            void_result(provider.append_file(&path, &content, mode_arg()).await)
        }
        "sessionFs.exists" => {
            let path = str_arg("path")?;
            let exists = provider.exists(&path).await.unwrap_or(false);
            Ok(json!({ "exists": exists }))
        }
        "sessionFs.stat" => {
            let path = str_arg("path")?;
            Ok(match provider.stat(&path).await {
                Ok(info) => serde_json::to_value(info)?,
                Err(err) => json!({
                    "isFile": false,
                    "isDirectory": false,
                    "size": 0,
                    "mtime": "",
                    "birthtime": "",
                    "error": err,
                }),
            })
        }
        "sessionFs.readdir" => {
            let path = str_arg("path")?;
            Ok(match provider.readdir(&path).await {
                Ok(entries) => json!({ "entries": entries }),
                Err(err) => json!({ "entries": [], "error": err }),
            })
        }
        "sessionFs.readdirWithTypes" => {
            let path = str_arg("path")?;
            Ok(match provider.readdir_with_types(&path).await {
                Ok(entries) => json!({ "entries": entries }),
                Err(err) => json!({ "entries": [], "error": err }),
            })
        }
        "sessionFs.mkdir" => {
            let path = str_arg("path")?;
            void_result(
                provider
                    .mkdir(&path, bool_arg("recursive"), mode_arg())
                    .await,
            )
        }
        "sessionFs.rm" => {
            let path = str_arg("path")?;
            void_result(
                provider
                    .rm(&path, bool_arg("recursive"), bool_arg("force"))
                    .await,
            )
        }
        "sessionFs.rename" => {
            let src = str_arg("src")?;
            let dest = str_arg("dest")?;
            void_result(provider.rename(&src, &dest).await)
        }
        "sessionFs.sqliteQuery" => {
            let sqlite = provider.sqlite().ok_or_else(|| {
                CopilotError::Protocol("SessionFsProvider does not implement SQLite support".into())
            })?;
            let query = str_arg("query")?;
            let query_type: SessionFsSqliteQueryType = serde_json::from_value(
                params
                    .get("queryType")
                    .cloned()
                    .ok_or_else(|| CopilotError::InvalidConfig("Missing queryType".into()))?,
            )?;
            let bind: Option<SessionFsSqliteParams> = params
                .get("params")
                .filter(|v| !v.is_null())
                .map(|v| serde_json::from_value(v.clone()))
                .transpose()?;

            Ok(
                match sqlite.query(query_type, &query, bind.as_ref()).await {
                    Ok(Some(result)) => serde_json::to_value(result)?,
                    Ok(None) => json!({ "rows": [], "columns": [], "rowsAffected": 0 }),
                    Err(err) => json!({
                        "rows": [],
                        "columns": [],
                        "rowsAffected": 0,
                        "error": err,
                    }),
                },
            )
        }
        "sessionFs.sqliteExists" => {
            let exists = match provider.sqlite() {
                Some(sqlite) => sqlite.exists().await.unwrap_or(false),
                None => false,
            };
            Ok(json!({ "exists": exists }))
        }
        _ => Err(CopilotError::Protocol(format!(
            "Unknown sessionFs method: {method}"
        ))),
    }
}

/// Handle an inbound `systemMessage.transform` request from the runtime.
async fn handle_system_message_transform(
    sessions: &RwLock<HashMap<String, Arc<Session>>>,
    params: &Value,
) -> Result<Value> {
    let session_id = params
        .get("sessionId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| CopilotError::InvalidConfig("Missing sessionId".into()))?;

    let sections = params
        .get("sections")
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            CopilotError::InvalidConfig("Invalid systemMessage.transform payload".into())
        })?;

    let session = sessions
        .read()
        .await
        .get(session_id)
        .cloned()
        .ok_or_else(|| CopilotError::Protocol(format!("Session not found: {session_id}")))?;

    let input: HashMap<String, String> = sections
        .iter()
        .map(|(id, value)| {
            let content = value
                .get("content")
                .and_then(|c| c.as_str())
                .unwrap_or_default()
                .to_string();
            (id.clone(), content)
        })
        .collect();

    let transformed = session.handle_system_message_transform(input).await;

    let out: serde_json::Map<String, Value> = transformed
        .into_iter()
        .map(|(id, content)| (id, json!({ "content": content })))
        .collect();

    Ok(json!({ "sections": out }))
}

/// Applies wire-level constants and mode defaults to a `session.create` /
/// `session.resume` payload, mirroring the Node.js client.
fn apply_wire_session_defaults(
    params: &mut Value,
    mode: CopilotClientMode,
    hooks: Option<&SessionHooks>,
) {
    let Some(obj) = params.as_object_mut() else {
        return;
    };

    // `hooks` is a capability flag on the wire, not the handler set itself.
    obj.insert(
        "hooks".into(),
        json!(hooks.map(SessionHooks::has_any).unwrap_or(false)),
    );

    // Constants the runtime expects on every session request.
    obj.entry("toolFilterPrecedence")
        .or_insert_with(|| json!("excluded"));
    obj.entry("envValueMode").or_insert_with(|| json!("direct"));
    obj.entry("includeSubAgentStreamingEvents")
        .or_insert_with(|| json!(true));

    if mode != CopilotClientMode::Empty {
        return;
    }

    // Empty mode starts from a locked-down baseline; caller values win.
    for (key, value) in [
        ("enableSessionTelemetry", json!(false)),
        ("mcpOAuthTokenStorage", json!("in-memory")),
        ("skipEmbeddingRetrieval", json!(true)),
        ("embeddingCacheStorage", json!("in-memory")),
        ("enableOnDemandInstructionDiscovery", json!(false)),
        ("enableFileHooks", json!(false)),
        ("enableHostGitOperations", json!(false)),
        ("enableSessionStore", json!(false)),
        ("enableSkills", json!(false)),
    ] {
        obj.entry(key).or_insert(value);
    }
}

/// Builds the post-create `session.options.update` patch, mirroring the Node.js
/// `updateSessionOptionsForMode` helper.
fn session_options_patch_for_mode(
    config: &SessionConfig,
    mode: CopilotClientMode,
) -> SessionUpdateOptions {
    session_options_patch(
        mode,
        config.skip_custom_instructions,
        config.custom_agents_local_only,
        config.coauthor_enabled,
        config.manage_schedule_enabled,
    )
}

/// Same as [`session_options_patch_for_mode`], but for `session.resume`.
fn resume_options_patch_for_mode(
    config: &ResumeSessionConfig,
    mode: CopilotClientMode,
) -> SessionUpdateOptions {
    session_options_patch(
        mode,
        config.skip_custom_instructions,
        config.custom_agents_local_only,
        config.coauthor_enabled,
        config.manage_schedule_enabled,
    )
}

fn session_options_patch(
    mode: CopilotClientMode,
    skip_custom_instructions: Option<bool>,
    custom_agents_local_only: Option<bool>,
    coauthor_enabled: Option<bool>,
    manage_schedule_enabled: Option<bool>,
) -> SessionUpdateOptions {
    if mode == CopilotClientMode::Empty {
        return SessionUpdateOptions {
            skip_custom_instructions: Some(skip_custom_instructions.unwrap_or(true)),
            custom_agents_local_only: Some(custom_agents_local_only.unwrap_or(true)),
            coauthor_enabled: Some(coauthor_enabled.unwrap_or(false)),
            manage_schedule_enabled: Some(manage_schedule_enabled.unwrap_or(false)),
            installed_plugins: Some(Vec::new()),
            ..Default::default()
        };
    }

    SessionUpdateOptions {
        skip_custom_instructions,
        custom_agents_local_only,
        coauthor_enabled,
        manage_schedule_enabled,
        ..Default::default()
    }
}

/// Applies the empty-mode system message default: remove the environment
/// context section unless the caller already decided what to do with it.
fn empty_mode_system_message(
    supplied: Option<crate::types::SystemMessageConfig>,
) -> crate::types::SystemMessageConfig {
    use crate::types::{
        SectionOverride, SystemMessageConfig, SystemMessageMode, SystemMessageSection,
    };

    let env_key = SystemMessageSection::EnvironmentContext.id().to_string();
    let Some(mut supplied) = supplied else {
        let mut sections = HashMap::new();
        sections.insert(env_key, SectionOverride::remove());
        return SystemMessageConfig {
            mode: Some(SystemMessageMode::Customize),
            sections: Some(sections),
            ..Default::default()
        };
    };

    if supplied.mode == Some(SystemMessageMode::Replace) {
        return supplied;
    }

    let mut sections = supplied.sections.take().unwrap_or_default();
    sections
        .entry(env_key)
        .or_insert_with(SectionOverride::remove);
    supplied.mode = Some(SystemMessageMode::Customize);
    supplied.sections = Some(sections);
    supplied
}

/// Splits section overrides into the wire payload and the client-side transform
/// callbacks, mirroring the Node.js `extractTransformCallbacks` helper.
fn extract_transform_callbacks(
    system_message: Option<&crate::types::SystemMessageConfig>,
) -> HashMap<String, crate::types::SectionTransformFn> {
    let Some(system_message) = system_message else {
        return HashMap::new();
    };
    if system_message.mode != Some(crate::types::SystemMessageMode::Customize) {
        return HashMap::new();
    }
    let Some(sections) = system_message.sections.as_ref() else {
        return HashMap::new();
    };

    sections
        .iter()
        .filter_map(|(id, override_)| {
            override_
                .transform_fn()
                .map(|callback| (id.clone(), Arc::clone(callback)))
        })
        .collect()
}

fn parse_cli_url(url: &str) -> Result<(String, u16)> {
    let mut s = url.trim();
    if let Some((_, rest)) = s.split_once("://") {
        s = rest;
    }
    if let Some((host_port, _)) = s.split_once('/') {
        s = host_port;
    }

    if s.chars().all(|c| c.is_ascii_digit()) {
        let port: u16 = s.parse().map_err(|_| {
            CopilotError::InvalidConfig(format!("Invalid port in cli_url: {}", url))
        })?;
        return Ok(("localhost".to_string(), port));
    }

    if let Some((host, port_str)) = s.rsplit_once(':') {
        let host = host.trim();
        let port: u16 = port_str.trim().parse().map_err(|_| {
            CopilotError::InvalidConfig(format!("Invalid port in cli_url: {}", url))
        })?;
        if host.is_empty() {
            return Ok(("localhost".to_string(), port));
        }
        return Ok((host.to_string(), port));
    }

    Err(CopilotError::InvalidConfig(format!(
        "Invalid cli_url format (expected host:port or port): {}",
        url
    )))
}

fn parse_listening_port(line: &str) -> Option<u16> {
    let lower = line.to_lowercase();
    let idx = lower.find("listening on port")?;
    let after = &line[idx..];

    let mut digits = String::new();
    let mut in_digits = false;
    for ch in after.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            in_digits = true;
        } else if in_digits {
            break;
        }
    }
    digits.parse::<u16>().ok()
}

async fn detect_tcp_port_from_stdout(stdout: tokio::process::ChildStdout) -> Result<u16> {
    let mut lines = BufReader::new(stdout).lines();
    let port = tokio::time::timeout(Duration::from_secs(15), async {
        while let Ok(Some(line)) = lines.next_line().await {
            if let Some(port) = parse_listening_port(&line) {
                return Ok(port);
            }
        }
        Err(CopilotError::PortDetectionFailed)
    })
    .await
    .map_err(|_| CopilotError::Timeout(Duration::from_secs(15)))??;

    Ok(port)
}

enum RpcClient {
    Stdio(StdioJsonRpcClient),
    Tcp(TcpJsonRpcClient),
    ParentStdio(JsonRpcClient<ParentStdioTransport>),
}

impl RpcClient {
    async fn stop(&self) {
        match self {
            RpcClient::Stdio(rpc) => rpc.stop().await,
            RpcClient::Tcp(rpc) => rpc.stop().await,
            RpcClient::ParentStdio(rpc) => rpc.stop().await,
        }
    }

    async fn set_notification_handler<F>(&self, handler: F)
    where
        F: Fn(&str, &Value) + Send + Sync + 'static,
    {
        let handler = Arc::new(handler);
        match self {
            RpcClient::Stdio(rpc) => {
                let handler = Arc::clone(&handler);
                rpc.set_notification_handler(move |method, params| {
                    (handler)(method, params);
                })
                .await;
            }
            RpcClient::Tcp(rpc) => {
                let handler = Arc::clone(&handler);
                rpc.set_notification_handler(move |method, params| {
                    (handler)(method, params);
                })
                .await;
            }
            RpcClient::ParentStdio(rpc) => {
                let handler = Arc::clone(&handler);
                rpc.set_notification_handler(move |method, params| {
                    (handler)(method, params);
                })
                .await;
            }
        }
    }

    async fn set_request_handler<F>(&self, handler: F)
    where
        F: Fn(&str, &Value) -> crate::jsonrpc::RequestHandlerFuture + Send + Sync + 'static,
    {
        let handler = Arc::new(handler);
        match self {
            RpcClient::Stdio(rpc) => {
                let handler = Arc::clone(&handler);
                rpc.set_request_handler(move |method, params| (handler)(method, params))
                    .await;
            }
            RpcClient::Tcp(rpc) => {
                let handler = Arc::clone(&handler);
                rpc.set_request_handler(move |method, params| (handler)(method, params))
                    .await;
            }
            RpcClient::ParentStdio(rpc) => {
                let handler = Arc::clone(&handler);
                rpc.set_request_handler(move |method, params| (handler)(method, params))
                    .await;
            }
        }
    }

    async fn invoke(&self, method: &str, params: Option<Value>) -> Result<Value> {
        match self {
            RpcClient::Stdio(rpc) => rpc.invoke(method, params).await,
            RpcClient::Tcp(rpc) => rpc.invoke(method, params).await,
            RpcClient::ParentStdio(rpc) => rpc.invoke(method, params).await,
        }
    }
}

// =============================================================================
// Client
// =============================================================================

/// Copilot client for managing connections and sessions.
///
/// The client manages the connection to the Copilot CLI server and provides
/// methods to create and manage conversation sessions.
///
/// # Example
///
/// ```no_run
/// use copilot_sdk::{Client, ClientOptions, SessionConfig};
///
/// #[tokio::main]
/// async fn main() -> copilot_sdk::Result<()> {
///     // Create client with options
///     let client = Client::new(ClientOptions::default())?;
///
///     // Start the client
///     client.start().await?;
///
///     // Create a session
///     let session = client.create_session(SessionConfig::default()).await?;
///
///     // Send a message and collect response
///     let response = session.send_and_collect("Hello!", None).await?;
///     println!("{}", response);
///
///     // Stop the client
///     client.stop().await;
///     Ok(())
/// }
/// ```
pub struct Client {
    options: ClientOptions,
    state: Arc<RwLock<ConnectionState>>,
    lifecycle: Mutex<()>,
    process: Mutex<Option<CopilotProcess>>,
    rpc: Arc<Mutex<Option<RpcClient>>>,
    sessions: Arc<RwLock<HashMap<String, Arc<Session>>>>,
    lifecycle_handlers: Arc<RwLock<HashMap<u64, LifecycleHandler>>>,
    next_lifecycle_handler_id: AtomicU64,
    models_cache: Arc<Mutex<Option<Vec<ModelInfo>>>>,
    negotiated_protocol_version: Arc<Mutex<Option<u32>>>,
}

impl Client {
    /// Create a new Copilot client with the given options.
    pub fn new(options: ClientOptions) -> Result<Self> {
        let mut options = options;

        if options.cli_url.is_some() {
            options.use_stdio = false;
        }

        // Validate mutually exclusive options
        if options.cli_url.is_some() {
            if options.cli_path.is_some() {
                return Err(CopilotError::InvalidConfig(
                    "cli_url is mutually exclusive with cli_path".into(),
                ));
            }
            if options.port != 0 {
                return Err(CopilotError::InvalidConfig(
                    "cli_url is mutually exclusive with port".into(),
                ));
            }
        }
        if options.use_stdio && options.port != 0 {
            return Err(CopilotError::InvalidConfig(
                "port is only valid when use_stdio=false".into(),
            ));
        }
        if options.cli_url.is_some() && options.github_token.is_some() {
            return Err(CopilotError::InvalidConfig(
                "github_token cannot be used with cli_url (external server doesn't accept token)"
                    .into(),
            ));
        }
        if options.cli_url.is_some() && options.use_logged_in_user.is_some() {
            return Err(CopilotError::InvalidConfig(
                "use_logged_in_user cannot be used with cli_url (external server doesn't accept this option)".into(),
            ));
        }

        // Empty mode: validate at construction time that the app supplied a
        // per-session persistence location. The runtime is mode-agnostic, so
        // without this check it would silently fall back to the user's home
        // directory, defeating the point of empty mode for multi-tenant hosts.
        if options.mode == CopilotClientMode::Empty {
            let has_persistence = options.cwd.is_some()
                || options.session_fs.is_some()
                // External / parent-owned runtimes manage their own persistence.
                || options.cli_url.is_some()
                || options.connection_kind == ConnectionKind::ParentProcess;
            if !has_persistence {
                return Err(CopilotError::InvalidConfig(
                    "Client was created with CopilotClientMode::Empty but neither `cwd` nor \
                     `session_fs` was set. Empty mode requires an explicit per-session \
                     persistence location; pick one."
                        .into(),
                ));
            }
        }

        if let Some(session_fs) = &options.session_fs {
            session_fs.validate().map_err(CopilotError::InvalidConfig)?;
        }

        Ok(Self {
            options,
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            lifecycle: Mutex::new(()),
            process: Mutex::new(None),
            rpc: Arc::new(Mutex::new(None)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            lifecycle_handlers: Arc::new(RwLock::new(HashMap::new())),
            next_lifecycle_handler_id: AtomicU64::new(1),
            models_cache: Arc::new(Mutex::new(None)),
            negotiated_protocol_version: Arc::new(Mutex::new(None)),
        })
    }

    /// Create a client builder for fluent configuration.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    // =========================================================================
    // Connection Management
    // =========================================================================

    /// Start the client and connect to the server.
    pub async fn start(&self) -> Result<()> {
        let _guard = self.lifecycle.lock().await;

        let mut state = self.state.write().await;
        if *state == ConnectionState::Connected {
            return Ok(());
        }
        if *state != ConnectionState::Disconnected {
            return Err(CopilotError::InvalidConfig(
                "Client is already started".into(),
            ));
        }
        *state = ConnectionState::Connecting;
        drop(state);

        // Start CLI server process
        let result = self.start_cli_server().await;
        if let Err(e) = result {
            *self.state.write().await = ConnectionState::Error;
            return Err(e);
        }

        // Verify protocol version
        if let Err(e) = self.verify_protocol_version().await {
            *self.state.write().await = ConnectionState::Error;
            return Err(e);
        }

        // Set up event handlers
        self.setup_handlers().await?;

        *self.state.write().await = ConnectionState::Connected;

        // Advertise the client-provided session filesystem, if configured.
        if let Err(e) = self.announce_session_fs().await {
            *self.state.write().await = ConnectionState::Error;
            return Err(e);
        }

        Ok(())
    }

    /// Sends `sessionFs.setProvider` when the client declares a session
    /// filesystem, telling the runtime to route all file operations back to us.
    async fn announce_session_fs(&self) -> Result<()> {
        let Some(config) = self.options.session_fs.as_ref() else {
            return Ok(());
        };
        let params = Some(serde_json::to_value(config)?);
        // Called from `start`, so bypass the auto-restart wrapper in `invoke`
        // to avoid a recursive start cycle.
        let rpc = self.rpc.lock().await;
        let rpc = rpc.as_ref().ok_or(CopilotError::NotConnected)?;
        rpc.invoke("sessionFs.setProvider", params).await?;
        Ok(())
    }

    /// Joins the current foreground session as a Copilot CLI extension.
    ///
    /// Intended for extensions launched as child processes of the Copilot CLI:
    /// the session id is read from the `SESSION_ID` environment variable and
    /// the client speaks JSON-RPC over this process's own stdin/stdout.
    ///
    /// The permission handler defaults to
    /// [`default_join_session_permission_handler`](crate::default_join_session_permission_handler)
    /// and `suppress_resume_event` defaults to `true`.
    ///
    /// # Errors
    ///
    /// Returns [`CopilotError::InvalidConfig`] when `SESSION_ID` is not set,
    /// which means the process was not started by the Copilot CLI.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use copilot_sdk::{Client, ResumeSessionConfig};
    ///
    /// # async fn run() -> copilot_sdk::Result<()> {
    /// let (client, session) = Client::join_session(ResumeSessionConfig::default()).await?;
    /// session.send("Hello from the extension").await?;
    /// # let _ = client;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn join_session(
        mut config: ResumeSessionConfig,
    ) -> Result<(Arc<Client>, Arc<Session>)> {
        let session_id = std::env::var("SESSION_ID").ok().filter(|s| !s.is_empty());
        let session_id = session_id.ok_or_else(|| {
            CopilotError::InvalidConfig(
                "join_session() is intended for extensions running as child processes of the \
                 Copilot CLI (SESSION_ID is not set)"
                    .into(),
            )
        })?;

        if config.suppress_resume_event.is_none() {
            config.suppress_resume_event = Some(true);
        }

        let client = Arc::new(Client::new(ClientOptions {
            connection_kind: ConnectionKind::ParentProcess,
            auto_start: false,
            auto_restart: false,
            ..Default::default()
        })?);
        client.start().await?;

        let session = client.resume_session(&session_id, config).await?;
        session
            .register_permission_handler(crate::types::default_join_session_permission_handler)
            .await;
        Ok((client, session))
    }

    /// Stop the client gracefully.
    pub async fn stop(&self) -> Vec<StopError> {
        let _guard = self.lifecycle.lock().await;
        let mut errors = Vec::new();

        let state = *self.state.read().await;
        if state == ConnectionState::Disconnected {
            self.sessions.write().await.clear();
            *self.rpc.lock().await = None;
            *self.process.lock().await = None;
            return errors;
        }

        // Best-effort destroy of all active sessions while still connected.
        let sessions: Vec<Arc<Session>> = self.sessions.read().await.values().cloned().collect();
        for session in sessions {
            if let Err(e) = session.destroy().await {
                errors.push(StopError {
                    message: format!("Failed to destroy session {}: {}", session.session_id(), e),
                    source: Some("session.destroy".into()),
                });
            }
        }
        self.sessions.write().await.clear();

        // Stop the RPC client
        if let Some(rpc) = self.rpc.lock().await.take() {
            rpc.stop().await;
        }

        // Stop the process
        if let Some(mut process) = self.process.lock().await.take() {
            let _ = process.terminate();
            let _ = process.wait().await;
        }

        *self.state.write().await = ConnectionState::Disconnected;
        *self.models_cache.lock().await = None;
        errors
    }

    /// Force stop the client immediately.
    pub async fn force_stop(&self) {
        let _guard = self.lifecycle.lock().await;

        self.sessions.write().await.clear();

        // Kill the process
        if let Some(mut process) = self.process.lock().await.take() {
            let _ = process.kill();
        }

        // Stop the RPC client
        if let Some(rpc) = self.rpc.lock().await.take() {
            rpc.stop().await;
        }

        *self.state.write().await = ConnectionState::Disconnected;
        *self.models_cache.lock().await = None;
    }

    /// Get the current connection state.
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }

    // =========================================================================
    // Session Management
    // =========================================================================

    /// Create a new Copilot session.
    pub async fn create_session(&self, mut config: SessionConfig) -> Result<Arc<Session>> {
        self.ensure_connected().await?;

        // Empty mode requires every session to opt into its tools explicitly.
        if self.options.mode == CopilotClientMode::Empty && config.available_tools.is_none() {
            return Err(CopilotError::InvalidConfig(
                "Client is in CopilotClientMode::Empty but the session config did not specify \
                 `available_tools`. Empty mode requires every session to explicitly opt into \
                 the tools it wants — e.g. `ToolSet::new().add_built_in(BuiltInTools::ISOLATED)`."
                    .into(),
            ));
        }

        // Empty mode drops the environment-context section unless the caller
        // already made a decision about it.
        if self.options.mode == CopilotClientMode::Empty {
            config.system_message = Some(empty_mode_system_message(config.system_message.take()));
        }

        // Apply BYOK from environment if enabled and not explicitly set
        if config.auto_byok_from_env && config.model.is_none() {
            config.model = ProviderConfig::model_from_env();
        }
        if config.auto_byok_from_env && config.provider.is_none() {
            config.provider = ProviderConfig::from_env();
        }

        // Build the request
        let mut params = serde_json::to_value(&config)?;
        apply_wire_session_defaults(&mut params, self.options.mode, config.hooks.as_ref());
        self.inject_trace_context(&mut params).await;

        // Send the request
        let result = self.invoke("session.create", Some(params)).await?;

        // Extract session ID
        let session_id = result
            .get("sessionId")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CopilotError::Protocol("Missing sessionId in response".into()))?
            .to_string();

        // Extract workspace_path (for infinite sessions)
        let workspace_path = result
            .get("workspacePath")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Create session object
        let session = self
            .create_session_object(session_id.clone(), workspace_path)
            .await;

        // Apply host-reported capabilities from the response.
        if let Some(caps) = result.get("capabilities") {
            if let Ok(capabilities) =
                serde_json::from_value::<crate::types::SessionCapabilities>(caps.clone())
            {
                session.set_capabilities(capabilities).await;
            }
        }

        // Register hooks from config if provided
        if let Some(hooks) = config.hooks.take() {
            if hooks.has_any() {
                session.register_hooks(hooks).await;
            }
        }

        // Register system message section transform callbacks.
        let transforms = extract_transform_callbacks(config.system_message.as_ref());
        if !transforms.is_empty() {
            session.register_transform_callbacks(transforms).await;
        }

        // Apply post-create session options. If the patch fails, disconnect the
        // orphaned runtime session rather than leaking it with permissive
        // defaults, then surface the original error.
        let patch = session_options_patch_for_mode(&config, self.options.mode);
        if !patch.is_empty() {
            if let Err(e) = session.update_options(patch).await {
                let _ = session.destroy().await;
                return Err(e);
            }
        }

        // Store session
        self.sessions
            .write()
            .await
            .insert(session_id, Arc::clone(&session));

        Ok(session)
    }

    /// Injects W3C trace-context headers into an outgoing request payload.
    async fn inject_trace_context(&self, params: &mut Value) {
        let Some(provider) = self.options.on_get_trace_context.as_ref() else {
            return;
        };
        let context = crate::trace::get_trace_context(Some(provider)).await;
        if context.is_empty() {
            return;
        }
        if let Some(obj) = params.as_object_mut() {
            if let Some(traceparent) = context.traceparent {
                obj.insert("traceparent".into(), json!(traceparent));
            }
            if let Some(tracestate) = context.tracestate {
                obj.insert("tracestate".into(), json!(tracestate));
            }
        }
    }

    /// Resume an existing session.
    pub async fn resume_session(
        &self,
        session_id: &str,
        mut config: ResumeSessionConfig,
    ) -> Result<Arc<Session>> {
        self.ensure_connected().await?;

        // Apply BYOK from environment if enabled and not explicitly set
        if config.auto_byok_from_env && config.provider.is_none() {
            config.provider = ProviderConfig::from_env();
        }

        // Build the request
        let mut params = serde_json::to_value(&config)?;
        params["sessionId"] = json!(session_id);
        apply_wire_session_defaults(&mut params, self.options.mode, config.hooks.as_ref());
        self.inject_trace_context(&mut params).await;

        // Send the request
        let result = self.invoke("session.resume", Some(params)).await?;

        // Extract session ID from response
        let resumed_id = result
            .get("sessionId")
            .and_then(|v| v.as_str())
            .unwrap_or(session_id)
            .to_string();

        // Extract workspace_path (for infinite sessions)
        let workspace_path = result
            .get("workspacePath")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Create session object
        let session = self
            .create_session_object(resumed_id.clone(), workspace_path)
            .await;

        // Apply host-reported capabilities from the response.
        if let Some(caps) = result.get("capabilities") {
            if let Ok(capabilities) =
                serde_json::from_value::<crate::types::SessionCapabilities>(caps.clone())
            {
                session.set_capabilities(capabilities).await;
            }
        }

        // Register hooks from config if provided
        if let Some(hooks) = config.hooks.take() {
            if hooks.has_any() {
                session.register_hooks(hooks).await;
            }
        }

        // Register system message section transform callbacks.
        let transforms = extract_transform_callbacks(config.system_message.as_ref());
        if !transforms.is_empty() {
            session.register_transform_callbacks(transforms).await;
        }

        // Record canvas instances the host restored alongside the session.
        if let Some(open) = result.get("openCanvases") {
            if let Ok(instances) =
                serde_json::from_value::<Vec<crate::canvas::OpenCanvasInstance>>(open.clone())
            {
                session.set_open_canvases(instances).await;
            }
        }

        // Apply post-resume session options; roll back on failure.
        let patch = resume_options_patch_for_mode(&config, self.options.mode);
        if !patch.is_empty() {
            if let Err(e) = session.update_options(patch).await {
                let _ = session.destroy().await;
                return Err(e);
            }
        }

        // Store session
        self.sessions
            .write()
            .await
            .insert(resumed_id, Arc::clone(&session));

        Ok(session)
    }

    /// List all available sessions.
    pub async fn list_sessions(&self) -> Result<Vec<SessionMetadata>> {
        self.ensure_connected().await?;

        let result = self.invoke("session.list", None).await?;

        let sessions: Vec<SessionMetadata> = result
            .get("sessions")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        Ok(sessions)
    }

    /// Delete a session.
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        self.ensure_connected().await?;

        let params = json!({ "sessionId": session_id });
        let result = self.invoke("session.delete", Some(params)).await?;

        if let Some(success) = result.get("success").and_then(|v| v.as_bool()) {
            if !success {
                let msg = result
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();
                return Err(CopilotError::Protocol(format!(
                    "Failed to delete session: {}",
                    msg
                )));
            }
        }

        // Remove from local cache
        self.sessions.write().await.remove(session_id);

        Ok(())
    }

    /// Get the ID of the most recently used session.
    pub async fn get_last_session_id(&self) -> Result<Option<String>> {
        self.ensure_connected().await?;

        let result = self.invoke("session.getLastId", None).await?;

        Ok(result
            .get("sessionId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()))
    }

    // =========================================================================
    // Server Communication
    // =========================================================================

    /// Send a ping to verify connection health.
    pub async fn ping(&self, message: Option<String>) -> Result<PingResponse> {
        self.ensure_connected().await?;

        let params = message.map(|m| json!({ "message": m }));
        let result = self.invoke("ping", params).await?;

        Ok(PingResponse {
            message: result
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            timestamp: result
                .get("timestamp")
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            protocol_version: result
                .get("protocolVersion")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
        })
    }

    /// Get CLI status including version and protocol information.
    pub async fn get_status(&self) -> Result<GetStatusResponse> {
        self.ensure_connected().await?;

        let result = self.invoke("status.get", None).await?;
        serde_json::from_value(result)
            .map_err(|e| CopilotError::Protocol(format!("Failed to parse status response: {}", e)))
    }

    /// Get current authentication status.
    pub async fn get_auth_status(&self) -> Result<GetAuthStatusResponse> {
        self.ensure_connected().await?;

        let result = self.invoke("auth.getStatus", None).await?;
        serde_json::from_value(result).map_err(|e| {
            CopilotError::Protocol(format!("Failed to parse auth status response: {}", e))
        })
    }

    /// List available models with their metadata.
    ///
    /// Results are cached after the first call. Use [`Client::clear_models_cache`] to force a refresh.
    ///
    /// If `on_list_models` was set via the builder, that callback is used instead of
    /// querying the CLI (useful for BYOK scenarios).
    ///
    /// # Errors
    /// Returns an error if not authenticated or if the request fails.
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>> {
        // Check cache first
        {
            let cache = self.models_cache.lock().await;
            if let Some(cached) = &*cache {
                return Ok(cached.clone());
            }
        }

        // Check for custom model list provider
        if let Some(ref handler) = self.options.on_list_models {
            let models = handler().await?;
            *self.models_cache.lock().await = Some(models.clone());
            return Ok(models);
        }

        self.ensure_connected().await?;

        let result = self.invoke("models.list", None).await?;
        let models = result
            .get("models")
            .cloned()
            .unwrap_or_else(|| serde_json::json!([]));
        let models: Vec<ModelInfo> = serde_json::from_value(models).map_err(|e| {
            CopilotError::Protocol(format!("Failed to parse models response: {}", e))
        })?;

        // Store in cache
        *self.models_cache.lock().await = Some(models.clone());

        Ok(models)
    }

    /// List available tools with optional model-specific overrides.
    pub async fn tools_list(&self, model_id: Option<&str>) -> Result<ToolsListResult> {
        self.ensure_connected().await?;

        let params = model_id.map(|id| json!({ "modelId": id }));
        let result = self.invoke("tools.list", params).await?;
        serde_json::from_value(result)
            .map_err(|e| CopilotError::Protocol(format!("Failed to parse tools list: {}", e)))
    }

    /// Get account quota information.
    pub async fn get_quota(&self) -> Result<QuotaResult> {
        self.ensure_connected().await?;

        let result = self.invoke("account.getQuota", None).await?;
        serde_json::from_value(result)
            .map_err(|e| CopilotError::Protocol(format!("Failed to parse quota result: {}", e)))
    }

    /// Clear the cached models list, forcing a fresh fetch on next `list_models()` call.
    pub async fn clear_models_cache(&self) {
        *self.models_cache.lock().await = None;
    }

    /// Get the foreground session ID and workspace path.
    pub async fn get_foreground_session_id(&self) -> Result<GetForegroundSessionResponse> {
        self.ensure_connected().await?;

        let result = self.invoke("session.getForeground", None).await?;
        serde_json::from_value(result).map_err(|e| {
            CopilotError::Protocol(format!("Failed to parse foreground response: {}", e))
        })
    }

    /// Set the foreground session ID.
    pub async fn set_foreground_session_id(
        &self,
        session_id: &str,
    ) -> Result<SetForegroundSessionResponse> {
        self.ensure_connected().await?;

        let params = json!({ "sessionId": session_id });
        let result = self.invoke("session.setForeground", Some(params)).await?;
        serde_json::from_value(result).map_err(|e| {
            CopilotError::Protocol(format!("Failed to parse set foreground response: {}", e))
        })
    }

    // =========================================================================
    // Lifecycle Event Handling
    // =========================================================================

    /// Register a handler for client-level lifecycle events.
    ///
    /// Lifecycle events include session created, deleted, updated, foreground, and background.
    /// Returns an unsubscribe closure that removes the handler when called.
    pub async fn on<F>(&self, handler: F) -> impl FnOnce()
    where
        F: Fn(&SessionLifecycleEvent) + Send + Sync + 'static,
    {
        let id = self
            .next_lifecycle_handler_id
            .fetch_add(1, Ordering::SeqCst);
        self.lifecycle_handlers
            .write()
            .await
            .insert(id, Arc::new(handler));

        let handlers = Arc::clone(&self.lifecycle_handlers);
        move || {
            tokio::spawn(async move {
                handlers.write().await.remove(&id);
            });
        }
    }

    // =========================================================================
    // Internal Methods
    // =========================================================================

    /// Invoke a JSON-RPC method.
    pub(crate) async fn invoke(&self, method: &str, params: Option<Value>) -> Result<Value> {
        let mut attempt = 0;

        loop {
            let result = {
                let rpc = self.rpc.lock().await;
                let rpc = rpc.as_ref().ok_or(CopilotError::NotConnected)?;
                rpc.invoke(method, params.clone()).await
            };

            match result {
                Ok(v) => return Ok(v),
                Err(e) => {
                    if attempt == 0
                        && *self.state.read().await == ConnectionState::Connected
                        && self.options.auto_restart
                        && self.should_restart_on_error(&e)
                    {
                        attempt += 1;
                        self.restart().await?;
                        continue;
                    }
                    return Err(e);
                }
            }
        }
    }

    /// Get a session by ID.
    pub async fn get_session(&self, session_id: &str) -> Option<Arc<Session>> {
        self.sessions.read().await.get(session_id).cloned()
    }

    /// Ensure client is connected.
    async fn ensure_connected(&self) -> Result<()> {
        match *self.state.read().await {
            ConnectionState::Connected => Ok(()),
            ConnectionState::Disconnected => {
                if self.options.auto_start {
                    self.start().await
                } else {
                    Err(CopilotError::NotConnected)
                }
            }
            ConnectionState::Error => {
                if self.options.auto_restart {
                    self.restart().await
                } else {
                    Err(CopilotError::NotConnected)
                }
            }
            ConnectionState::Connecting => Err(CopilotError::NotConnected),
        }
    }

    fn should_restart_on_error(&self, err: &CopilotError) -> bool {
        match err {
            CopilotError::ConnectionClosed | CopilotError::NotConnected => true,
            CopilotError::Transport(_) => true,
            CopilotError::ProcessExit(_) => true,
            CopilotError::JsonRpc { code, .. } => *code == -32801,
            _ => false,
        }
    }

    async fn restart(&self) -> Result<()> {
        self.force_stop().await;
        self.start().await
    }

    /// Start the CLI server process.
    async fn start_cli_server(&self) -> Result<()> {
        if self.options.connection_kind == ConnectionKind::ParentProcess {
            let rpc = JsonRpcClient::new(ParentStdioTransport::new());
            rpc.start().await?;
            *self.rpc.lock().await = Some(RpcClient::ParentStdio(rpc));
            return Ok(());
        }

        if let Some(cli_url) = &self.options.cli_url {
            let (host, port) = parse_cli_url(cli_url)?;
            let addr = format!("{}:{}", host, port);

            let rpc = TcpJsonRpcClient::connect(addr).await?;
            rpc.start().await?;

            *self.rpc.lock().await = Some(RpcClient::Tcp(rpc));
            return Ok(());
        }

        let cli = if let Some(cli_path) = self.options.cli_path.clone() {
            ResolvedCopilotCli::NativeExecutable(cli_path)
        } else {
            let discovery = crate::process::discover_copilot_cli();
            discovery
                .clone()
                .into_resolved_cli()
                .ok_or_else(|| CopilotError::InvalidConfig(discovery.not_found_message()))?
        };

        let log_level = self.options.log_level.to_string();

        let mut args: Vec<String> = Vec::new();
        if let Some(extra_args) = &self.options.cli_args {
            args.extend(extra_args.iter().cloned());
        }

        // Add deny-tool arguments
        if let Some(deny_tools) = &self.options.deny_tools {
            for tool_spec in deny_tools {
                args.push("--deny-tool".to_string());
                args.push(tool_spec.clone());
            }
        }

        // Add allow-tool arguments
        if let Some(allow_tools) = &self.options.allow_tools {
            for tool_spec in allow_tools {
                args.push("--allow-tool".to_string());
                args.push(tool_spec.clone());
            }
        }

        // Add allow-all-tools flag
        if self.options.allow_all_tools {
            args.push("--allow-all-tools".to_string());
        }

        args.extend(["--server".to_string(), "--log-level".to_string(), log_level]);

        if self.options.use_stdio {
            args.push("--stdio".to_string());
        } else if self.options.port != 0 {
            args.extend(["--port".to_string(), self.options.port.to_string()]);
        }

        // Wire github_token auth: CLI flag for auth token env var
        if self.options.github_token.is_some() {
            args.push("--auth-token-env".to_string());
            args.push("COPILOT_SDK_AUTH_TOKEN".to_string());
        }

        // Wire use_logged_in_user: when false, pass --no-auto-login
        if let Some(false) = self.options.use_logged_in_user {
            args.push("--no-auto-login".to_string());
        }

        if self.options.session_idle_timeout_seconds > 0 {
            args.push("--session-idle-timeout".to_string());
            args.push(self.options.session_idle_timeout_seconds.to_string());
        }

        if self.options.enable_remote_sessions {
            args.push("--remote".to_string());
        }

        // Resolve command and arguments based on platform
        // On Windows, use cmd /c for PATH resolution if path is not absolute (for .cmd files)
        let (executable, full_args) = resolve_cli_command(&cli, &args);

        // Build process options
        let mut proc_options = ProcessOptions::new()
            .stdin(self.options.use_stdio)
            .stdout(true)
            .stderr(true);

        if let Some(ref dir) = self.options.cwd {
            proc_options = proc_options.working_dir(dir.clone());
        }

        // Add environment variables
        if let Some(ref env) = self.options.environment {
            for (key, value) in env {
                proc_options = proc_options.env(key, value);
            }
        }

        // Remove NODE_DEBUG to avoid debug output interfering with JSON-RPC
        proc_options = proc_options.env("NODE_DEBUG", "");

        // In empty mode, disable the system keychain. It is a process-wide store
        // shared across sessions, which is unsafe for multi-tenant hosts; the
        // runtime falls back to file-based credential storage.
        if self.options.mode == CopilotClientMode::Empty {
            proc_options = proc_options.env("COPILOT_DISABLE_KEYTAR", "1");
        }

        // Wire github_token auth: pass via environment variable + CLI flag
        if let Some(ref token) = self.options.github_token {
            proc_options = proc_options.env("COPILOT_SDK_AUTH_TOKEN", token);
            args.push("--auth-token-env".to_string());
            args.push("COPILOT_SDK_AUTH_TOKEN".to_string());
        }

        // Wire use_logged_in_user: when false, pass --no-auto-login
        if let Some(false) = self.options.use_logged_in_user {
            args.push("--no-auto-login".to_string());
        }

        // Propagate telemetry configuration as environment variables
        if let Some(ref telemetry) = self.options.telemetry {
            if telemetry.otlp_endpoint.is_some() || telemetry.file_path.is_some() {
                proc_options = proc_options.env("COPILOT_OTEL_ENABLED", "true");
            }
            if let Some(ref endpoint) = telemetry.otlp_endpoint {
                proc_options = proc_options.env("OTEL_EXPORTER_OTLP_ENDPOINT", endpoint);
            }
            if let Some(ref path) = telemetry.file_path {
                proc_options = proc_options.env("COPILOT_OTEL_FILE_EXPORTER_PATH", path);
            }
            if let Some(ref exporter_type) = telemetry.exporter_type {
                proc_options = proc_options.env("COPILOT_OTEL_EXPORTER_TYPE", exporter_type);
            }
            if let Some(ref source_name) = telemetry.source_name {
                proc_options = proc_options.env("COPILOT_OTEL_SOURCE_NAME", source_name);
            }
            if let Some(capture) = telemetry.capture_content {
                if capture {
                    proc_options = proc_options
                        .env("OTEL_INSTRUMENTATION_GENAI_CAPTURE_MESSAGE_CONTENT", "true");
                }
            }
        }

        let args_refs: Vec<&str> = full_args.iter().map(|s| s.as_str()).collect();
        let mut process = CopilotProcess::spawn(&executable, &args_refs, proc_options)?;

        if let Some(stderr) = process.take_stderr() {
            spawn_cli_stderr_logger(stderr);
        }

        let rpc = if self.options.use_stdio {
            let transport = process.take_transport().ok_or_else(|| {
                CopilotError::InvalidConfig("Failed to get transport from process".into())
            })?;
            let rpc = StdioJsonRpcClient::new(transport);
            rpc.start().await?;
            RpcClient::Stdio(rpc)
        } else {
            let stdout = process.take_stdout().ok_or_else(|| {
                CopilotError::InvalidConfig("Failed to capture stdout for port detection".into())
            })?;

            let detected_port = detect_tcp_port_from_stdout(stdout).await?;
            let addr = format!("127.0.0.1:{}", detected_port);
            let rpc = TcpJsonRpcClient::connect(addr).await?;
            rpc.start().await?;
            RpcClient::Tcp(rpc)
        };

        *self.process.lock().await = Some(process);
        *self.rpc.lock().await = Some(rpc);

        Ok(())
    }

    /// Verify that the server's protocol version is within the supported range
    /// and store the negotiated version.
    async fn verify_protocol_version(&self) -> Result<()> {
        // NOTE: We call the underlying RPC directly instead of ping() because ping() calls
        // ensure_connected(), but we haven't set state to Connected yet.
        let rpc = self.rpc.lock().await;
        let rpc = rpc.as_ref().ok_or(CopilotError::NotConnected)?;
        let result = rpc
            .invoke("ping", Some(serde_json::json!({ "message": null })))
            .await?;

        let server_version = result
            .get("protocolVersion")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        match server_version {
            None => {
                return Err(CopilotError::ProtocolMismatch {
                    min: MIN_PROTOCOL_VERSION,
                    max: SDK_PROTOCOL_VERSION,
                    actual: 0,
                });
            }
            Some(version) if !(MIN_PROTOCOL_VERSION..=SDK_PROTOCOL_VERSION).contains(&version) => {
                return Err(CopilotError::ProtocolMismatch {
                    min: MIN_PROTOCOL_VERSION,
                    max: SDK_PROTOCOL_VERSION,
                    actual: version,
                });
            }
            Some(version) => {
                *self.negotiated_protocol_version.lock().await = Some(version);
            }
        }

        Ok(())
    }

    /// Get the negotiated protocol version (set after successful start).
    pub async fn negotiated_protocol_version(&self) -> Option<u32> {
        *self.negotiated_protocol_version.lock().await
    }

    /// Set up notification and request handlers.
    async fn setup_handlers(&self) -> Result<()> {
        let rpc = self.rpc.lock().await;
        let rpc = rpc.as_ref().ok_or(CopilotError::NotConnected)?;

        // Clone Arc references for the handlers
        let sessions = Arc::clone(&self.sessions);
        let lifecycle_handlers = Arc::clone(&self.lifecycle_handlers);

        // Set up notification handler for session events and lifecycle events
        rpc.set_notification_handler(move |method, params| {
            if method == "session.event" {
                let sessions = Arc::clone(&sessions);
                let params = params.clone();

                // Spawn a task to handle the event
                tokio::spawn(async move {
                    if let Some(session_id) = params.get("sessionId").and_then(|v| v.as_str()) {
                        if let Some(session) = sessions.read().await.get(session_id) {
                            if let Some(event_data) = params.get("event") {
                                if let Ok(event) = SessionEvent::from_json(event_data) {
                                    session.dispatch_event(event).await;
                                }
                            }
                        }
                    }
                });
            } else if method == "session.lifecycle" {
                let lifecycle_handlers = Arc::clone(&lifecycle_handlers);
                let params = params.clone();

                tokio::spawn(async move {
                    if let Ok(event) = serde_json::from_value::<SessionLifecycleEvent>(params) {
                        let handlers = lifecycle_handlers.read().await;
                        for handler in handlers.values() {
                            handler(&event);
                        }
                    }
                });
            }
        })
        .await;

        // Clone Arc references for request handler
        let sessions_for_requests = Arc::clone(&self.sessions);

        // Protocol v2 backward-compatibility adapters.
        // v2 servers send tool.call / permission.request as RPC requests.
        // v3 servers send them as broadcast session events (handled in Session::handle_broadcast_event).
        // We always register v2 handlers; a v3 server will simply never send these requests.
        rpc.set_request_handler(move |method, params| {
            use crate::jsonrpc::JsonRpcError;

            let sessions = Arc::clone(&sessions_for_requests);
            let method = method.to_string();
            let params = params.clone();

            Box::pin(async move {
                let result = match method.as_str() {
                    "tool.call" => handle_tool_call(&sessions, &params).await,
                    "permission.request" => handle_permission_request(&sessions, &params).await,
                    "userInput.request" => handle_user_input_request(&sessions, &params).await,
                    "hooks.invoke" => handle_hooks_invoke(&sessions, &params).await,
                    "systemMessage.transform" => {
                        handle_system_message_transform(&sessions, &params).await
                    }
                    m if m.starts_with("canvas.") => {
                        handle_canvas_request(&sessions, m, &params).await
                    }
                    m if m.starts_with("sessionFs.") => {
                        handle_session_fs_request(&sessions, m, &params).await
                    }
                    _ => {
                        return Err(JsonRpcError::new(
                            -32601,
                            format!("Unknown method: {}", method),
                        ));
                    }
                };

                result.map_err(|e| JsonRpcError::new(-32000, e.to_string()))
            })
        })
        .await;

        Ok(())
    }

    /// Create a session object with the invoke function.
    async fn create_session_object(
        &self,
        session_id: String,
        workspace_path: Option<String>,
    ) -> Arc<Session> {
        let rpc = Arc::clone(&self.rpc);

        // Create the invoke function that captures the RPC client
        let invoke_fn = move |method: &str, params: Option<Value>| {
            let rpc = Arc::clone(&rpc);
            let method = method.to_string();

            Box::pin(async move {
                let rpc = rpc.lock().await;
                let rpc = rpc.as_ref().ok_or(CopilotError::NotConnected)?;
                rpc.invoke(&method, params).await
            }) as crate::session::InvokeFuture
        };

        Arc::new(Session::new(session_id, workspace_path, invoke_fn))
    }
}

// =============================================================================
// Client Builder
// =============================================================================

/// Builder for creating a Copilot client.
#[derive(Debug, Default)]
pub struct ClientBuilder {
    options: ClientOptions,
}

impl ClientBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the CLI executable path.
    pub fn cli_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.options.cli_path = Some(path.into());
        self
    }

    /// Set additional CLI arguments passed to the Copilot CLI.
    pub fn cli_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.options.cli_args = Some(args.into_iter().map(Into::into).collect());
        self
    }

    /// Add a single CLI argument passed to the Copilot CLI.
    pub fn cli_arg(mut self, arg: impl Into<String>) -> Self {
        self.options
            .cli_args
            .get_or_insert_with(Vec::new)
            .push(arg.into());
        self
    }

    /// Use stdio mode (default).
    pub fn use_stdio(mut self, use_stdio: bool) -> Self {
        self.options.use_stdio = use_stdio;
        self
    }

    /// Set the CLI URL for TCP mode.
    ///
    /// Supports: `"host:port"`, `"http://host:port"`, or `"port"` (defaults to localhost).
    pub fn cli_url(mut self, url: impl Into<String>) -> Self {
        self.options.cli_url = Some(url.into());
        self.options.use_stdio = false;
        self
    }

    /// Set port for TCP mode (ignored for stdio mode).
    ///
    /// Use `0` to let the CLI choose a random available port.
    pub fn port(mut self, port: u16) -> Self {
        self.options.port = port;
        self
    }

    /// Auto-start the connection on first use.
    pub fn auto_start(mut self, auto_start: bool) -> Self {
        self.options.auto_start = auto_start;
        self
    }

    /// Auto-restart the connection after a fatal failure.
    pub fn auto_restart(mut self, auto_restart: bool) -> Self {
        self.options.auto_restart = auto_restart;
        self
    }

    /// Set the client mode.
    ///
    /// [`CopilotClientMode::Empty`] starts from a blank slate: sessions must
    /// opt into their tools explicitly and the client must be given an explicit
    /// persistence location via [`ClientBuilder::cwd`] or
    /// [`ClientBuilder::session_fs`].
    pub fn mode(mut self, mode: CopilotClientMode) -> Self {
        self.options.mode = mode;
        self
    }

    /// Supply a client-hosted filesystem for sessions to use.
    pub fn session_fs(mut self, config: crate::session_fs::SessionFsConfig) -> Self {
        self.options.session_fs = Some(config);
        self
    }

    /// Provide distributed-trace context injected into `session.create` and
    /// `session.resume` requests.
    pub fn on_get_trace_context(mut self, provider: crate::trace::TraceContextProvider) -> Self {
        self.options.on_get_trace_context = Some(provider);
        self
    }

    /// Set the idle timeout, in seconds, after which the runtime shuts a
    /// session down. `0` disables the timeout.
    pub fn session_idle_timeout_seconds(mut self, seconds: u64) -> Self {
        self.options.session_idle_timeout_seconds = seconds;
        self
    }

    /// Start the runtime with remote-session support (`--remote`).
    pub fn enable_remote_sessions(mut self, enabled: bool) -> Self {
        self.options.enable_remote_sessions = enabled;
        self
    }

    /// Set the log level.
    pub fn log_level(mut self, level: LogLevel) -> Self {
        self.options.log_level = level;
        self
    }

    /// Set the working directory.
    pub fn cwd(mut self, dir: impl Into<PathBuf>) -> Self {
        self.options.cwd = Some(dir.into());
        self
    }

    /// Add an environment variable.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.options
            .environment
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// Set a GitHub personal access token for authentication.
    pub fn github_token(mut self, token: impl Into<String>) -> Self {
        self.options.github_token = Some(token.into());
        self
    }

    /// Set whether to use the logged-in user for auth.
    pub fn use_logged_in_user(mut self, value: bool) -> Self {
        self.options.use_logged_in_user = Some(value);
        self
    }

    /// Add a single tool specification to deny.
    ///
    /// Passed as `--deny-tool` to the CLI. Takes precedence over allow options.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use copilot_sdk::Client;
    ///
    /// let client = Client::builder()
    ///     .deny_tool("shell(git push)")
    ///     .deny_tool("shell(git commit)")
    ///     .deny_tool("shell(rm)")
    ///     .build()?;
    /// # Ok::<(), copilot_sdk::CopilotError>(())
    /// ```
    pub fn deny_tool(mut self, tool_spec: impl Into<String>) -> Self {
        self.options
            .deny_tools
            .get_or_insert_with(Vec::new)
            .push(tool_spec.into());
        self
    }

    /// Set multiple tool specifications to deny.
    ///
    /// Passed as `--deny-tool` arguments to the CLI.
    pub fn deny_tools<I, S>(mut self, tool_specs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.options.deny_tools = Some(tool_specs.into_iter().map(Into::into).collect());
        self
    }

    /// Add a single tool specification to allow without manual approval.
    ///
    /// Passed as `--allow-tool` to the CLI.
    pub fn allow_tool(mut self, tool_spec: impl Into<String>) -> Self {
        self.options
            .allow_tools
            .get_or_insert_with(Vec::new)
            .push(tool_spec.into());
        self
    }

    /// Set multiple tool specifications to allow without manual approval.
    ///
    /// Passed as `--allow-tool` arguments to the CLI.
    pub fn allow_tools<I, S>(mut self, tool_specs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.options.allow_tools = Some(tool_specs.into_iter().map(Into::into).collect());
        self
    }

    /// Set the telemetry configuration.
    pub fn telemetry(mut self, config: TelemetryConfig) -> Self {
        self.options.telemetry = Some(config);
        self
    }

    /// Set a custom model list provider for BYOK.
    ///
    /// When set, `list_models()` will call this handler instead of querying the CLI.
    pub fn on_list_models<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = std::result::Result<Vec<ModelInfo>, CopilotError>>
            + Send
            + 'static,
    {
        self.options.on_list_models = Some(Arc::new(move || Box::pin(handler())));
        self
    }

    /// Allow all tools without manual approval.
    ///
    /// Passes `--allow-all-tools` to the CLI. Use with `deny_tool()` to create
    /// an allowlist with specific exceptions.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use copilot_sdk::Client;
    ///
    /// // Allow everything except dangerous git operations and rm
    /// let client = Client::builder()
    ///     .allow_all_tools(true)
    ///     .deny_tool("shell(git push)")
    ///     .deny_tool("shell(git commit)")
    ///     .deny_tool("shell(rm)")
    ///     .build()?;
    /// # Ok::<(), copilot_sdk::CopilotError>(())
    /// ```
    pub fn allow_all_tools(mut self, allow: bool) -> Self {
        self.options.allow_all_tools = allow;
        self
    }

    /// Build the client.
    pub fn build(self) -> Result<Client> {
        Client::new(self.options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SectionOverride, SectionOverrideAction};

    #[test]
    fn test_client_builder() {
        let client = Client::builder()
            .cli_path("/usr/bin/copilot")
            .cli_arg("--foo")
            .use_stdio(true)
            .log_level(LogLevel::Debug)
            .cwd("/tmp")
            .env("FOO", "bar")
            .build();

        assert!(client.is_ok());
    }

    #[test]
    fn test_client_builder_deny_allow_tools() {
        let client = Client::builder()
            .allow_all_tools(true)
            .deny_tool("shell(git push)")
            .deny_tool("shell(git commit)")
            .deny_tool("shell(rm)")
            .allow_tool("shell(ls)")
            .build()
            .unwrap();

        assert!(client.options.allow_all_tools);
        assert_eq!(
            client.options.deny_tools,
            Some(vec![
                "shell(git push)".to_string(),
                "shell(git commit)".to_string(),
                "shell(rm)".to_string(),
            ])
        );
        assert_eq!(
            client.options.allow_tools,
            Some(vec!["shell(ls)".to_string()])
        );
    }

    #[test]
    fn test_client_builder_deny_tools_batch() {
        let client = Client::builder()
            .deny_tools(vec!["shell(git push)", "shell(git add)"])
            .build()
            .unwrap();

        assert_eq!(
            client.options.deny_tools,
            Some(vec![
                "shell(git push)".to_string(),
                "shell(git add)".to_string(),
            ])
        );
    }

    #[test]
    fn test_client_mutually_exclusive_options() {
        let options = ClientOptions {
            cli_path: Some("/usr/bin/copilot".into()),
            cli_url: Some("http://localhost:8080".into()),
            ..Default::default()
        };
        assert!(matches!(
            Client::new(options),
            Err(CopilotError::InvalidConfig(_))
        ));

        let options = ClientOptions {
            cli_url: Some("localhost:8080".into()),
            port: 1234,
            ..Default::default()
        };
        assert!(matches!(
            Client::new(options),
            Err(CopilotError::InvalidConfig(_))
        ));

        let options = ClientOptions {
            use_stdio: true,
            port: 1234,
            ..Default::default()
        };
        assert!(matches!(
            Client::new(options),
            Err(CopilotError::InvalidConfig(_))
        ));

        // github_token + cli_url is invalid
        let options = ClientOptions {
            cli_url: Some("localhost:8080".into()),
            github_token: Some("ghp_abc123".into()),
            ..Default::default()
        };
        assert!(matches!(
            Client::new(options),
            Err(CopilotError::InvalidConfig(_))
        ));

        // use_logged_in_user + cli_url is invalid
        let options = ClientOptions {
            cli_url: Some("localhost:8080".into()),
            use_logged_in_user: Some(true),
            ..Default::default()
        };
        assert!(matches!(
            Client::new(options),
            Err(CopilotError::InvalidConfig(_))
        ));
    }

    #[tokio::test]
    async fn test_client_state_initial() {
        let client = Client::new(ClientOptions::default()).unwrap();
        assert_eq!(client.state().await, ConnectionState::Disconnected);
    }

    #[test]
    fn test_normalize_tool_arguments_object() {
        let params = json!({
            "arguments": { "n": 42 }
        });
        assert_eq!(normalize_tool_arguments(&params), json!({ "n": 42 }));
    }

    #[test]
    fn test_normalize_tool_arguments_string() {
        let params = json!({
            "arguments": "{\"n\":42}"
        });
        assert_eq!(normalize_tool_arguments(&params), json!({ "n": 42 }));
    }

    #[test]
    fn test_normalize_tool_arguments_fallback_arguments_json() {
        let params = json!({
            "argumentsJson": "{\"text\":\"hello\",\"shift\":-5}"
        });
        assert_eq!(
            normalize_tool_arguments(&params),
            json!({ "text": "hello", "shift": -5 })
        );
    }

    #[test]
    fn test_normalize_tool_arguments_invalid_json_string() {
        let params = json!({
            "arguments": "{not valid json"
        });
        assert_eq!(normalize_tool_arguments(&params), json!({}));
    }

    // =========================================================================
    // Client mode
    // =========================================================================

    #[test]
    fn test_empty_mode_requires_persistence() {
        let result = Client::builder().mode(CopilotClientMode::Empty).build();
        assert!(matches!(result, Err(CopilotError::InvalidConfig(_))));
    }

    #[test]
    fn test_empty_mode_accepts_cwd() {
        assert!(Client::builder()
            .mode(CopilotClientMode::Empty)
            .cwd("/tmp")
            .build()
            .is_ok());
    }

    #[test]
    fn test_empty_mode_accepts_session_fs() {
        let config = crate::session_fs::SessionFsConfig::new("/w", "/state");
        assert!(Client::builder()
            .mode(CopilotClientMode::Empty)
            .session_fs(config)
            .build()
            .is_ok());
    }

    #[test]
    fn test_copilot_cli_mode_is_the_default() {
        let client = Client::builder().build().unwrap();
        assert_eq!(client.options.mode, CopilotClientMode::CopilotCli);
        assert_eq!(client.options.connection_kind, ConnectionKind::Child);
    }

    #[test]
    fn test_empty_mode_system_message_defaults_to_removing_env_context() {
        let config = empty_mode_system_message(None);
        assert_eq!(
            config.mode,
            Some(crate::types::SystemMessageMode::Customize)
        );
        let sections = config.sections.unwrap();
        assert_eq!(
            sections
                .get(crate::types::SystemMessageSection::EnvironmentContext.id())
                .unwrap()
                .action,
            SectionOverrideAction::Remove
        );
    }

    #[test]
    fn test_empty_mode_system_message_respects_replace() {
        let supplied = crate::types::SystemMessageConfig {
            mode: Some(crate::types::SystemMessageMode::Replace),
            ..Default::default()
        };
        let config = empty_mode_system_message(Some(supplied));
        assert_eq!(config.mode, Some(crate::types::SystemMessageMode::Replace));
        assert!(config.sections.is_none());
    }

    #[test]
    fn test_empty_mode_system_message_preserves_explicit_env_context() {
        let mut sections = HashMap::new();
        sections.insert(
            crate::types::SystemMessageSection::EnvironmentContext
                .id()
                .to_string(),
            SectionOverride::append("extra"),
        );
        let supplied = crate::types::SystemMessageConfig {
            mode: Some(crate::types::SystemMessageMode::Customize),
            sections: Some(sections),
            ..Default::default()
        };
        let config = empty_mode_system_message(Some(supplied));
        let sections = config.sections.unwrap();
        assert_eq!(
            sections
                .get(crate::types::SystemMessageSection::EnvironmentContext.id())
                .unwrap()
                .action,
            SectionOverrideAction::Append
        );
    }

    // =========================================================================
    // System message transform
    // =========================================================================

    #[test]
    fn test_extract_transform_callbacks_requires_customize_mode() {
        let mut sections = HashMap::new();
        sections.insert(
            "identity".to_string(),
            SectionOverride::transform(|content| Box::pin(async move { content })),
        );
        let config = crate::types::SystemMessageConfig {
            mode: Some(crate::types::SystemMessageMode::Replace),
            sections: Some(sections),
            ..Default::default()
        };
        assert!(extract_transform_callbacks(Some(&config)).is_empty());
    }

    #[test]
    fn test_extract_transform_callbacks_collects_only_transforms() {
        let mut sections = HashMap::new();
        sections.insert(
            "identity".to_string(),
            SectionOverride::transform(|content| Box::pin(async move { content })),
        );
        sections.insert("tone".to_string(), SectionOverride::remove());
        let config = crate::types::SystemMessageConfig {
            mode: Some(crate::types::SystemMessageMode::Customize),
            sections: Some(sections),
            ..Default::default()
        };
        let callbacks = extract_transform_callbacks(Some(&config));
        assert_eq!(callbacks.len(), 1);
        assert!(callbacks.contains_key("identity"));
    }

    #[test]
    fn test_extract_transform_callbacks_without_config() {
        assert!(extract_transform_callbacks(None).is_empty());
    }

    #[test]
    fn test_transform_override_serializes_as_transform_action() {
        let over = SectionOverride::transform(|content| Box::pin(async move { content }));
        let value = serde_json::to_value(&over).unwrap();
        assert_eq!(value["action"], "transform");
        assert!(value.get("transform").is_none());
    }

    fn test_sessions(session: Arc<Session>) -> RwLock<HashMap<String, Arc<Session>>> {
        let mut map = HashMap::new();
        map.insert(session.session_id().to_string(), session);
        RwLock::new(map)
    }

    fn noop_invoke(_method: &str, _params: Option<Value>) -> crate::session::InvokeFuture {
        Box::pin(async { Ok(json!({})) })
    }

    #[tokio::test]
    async fn test_handle_system_message_transform_passes_through_unregistered() {
        let session = Arc::new(Session::new("s1".to_string(), None, noop_invoke));
        let sessions = test_sessions(session);

        let out = handle_system_message_transform(
            &sessions,
            &json!({
                "sessionId": "s1",
                "sections": { "identity": { "content": "original" } }
            }),
        )
        .await
        .unwrap();

        assert_eq!(out["sections"]["identity"]["content"], "original");
    }

    #[tokio::test]
    async fn test_handle_system_message_transform_applies_callback() {
        let session = Arc::new(Session::new("s1".to_string(), None, noop_invoke));
        let mut callbacks: HashMap<String, crate::types::SectionTransformFn> = HashMap::new();
        callbacks.insert(
            "identity".to_string(),
            Arc::new(|content: String| Box::pin(async move { format!("{content}!") }) as _),
        );
        session.register_transform_callbacks(callbacks).await;
        let sessions = test_sessions(session);

        let out = handle_system_message_transform(
            &sessions,
            &json!({
                "sessionId": "s1",
                "sections": {
                    "identity": { "content": "hello" },
                    "tone": { "content": "keep" }
                }
            }),
        )
        .await
        .unwrap();

        assert_eq!(out["sections"]["identity"]["content"], "hello!");
        assert_eq!(out["sections"]["tone"]["content"], "keep");
    }

    #[tokio::test]
    async fn test_handle_system_message_transform_unknown_session() {
        let session = Arc::new(Session::new("s1".to_string(), None, noop_invoke));
        let sessions = test_sessions(session);
        let err = handle_system_message_transform(
            &sessions,
            &json!({ "sessionId": "nope", "sections": {} }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, CopilotError::Protocol(_)));
    }

    // =========================================================================
    // sessionFs dispatch
    // =========================================================================

    #[derive(Default)]
    struct StubFs {
        files: std::sync::Mutex<HashMap<String, String>>,
    }

    impl crate::session_fs::SessionFsProvider for StubFs {
        fn read_file<'a>(
            &'a self,
            path: &'a str,
        ) -> crate::session_fs::SessionFsFuture<'a, String> {
            Box::pin(async move {
                self.files
                    .lock()
                    .unwrap()
                    .get(path)
                    .cloned()
                    .ok_or_else(|| crate::session_fs::SessionFsError::not_found(path))
            })
        }

        fn write_file<'a>(
            &'a self,
            path: &'a str,
            content: &'a str,
            _mode: Option<u32>,
        ) -> crate::session_fs::SessionFsFuture<'a, ()> {
            Box::pin(async move {
                self.files
                    .lock()
                    .unwrap()
                    .insert(path.to_string(), content.to_string());
                Ok(())
            })
        }

        fn exists<'a>(&'a self, path: &'a str) -> crate::session_fs::SessionFsFuture<'a, bool> {
            Box::pin(async move { Ok(self.files.lock().unwrap().contains_key(path)) })
        }

        fn stat<'a>(
            &'a self,
            path: &'a str,
        ) -> crate::session_fs::SessionFsFuture<'a, crate::session_fs::SessionFsFileInfo> {
            Box::pin(async move {
                let files = self.files.lock().unwrap();
                let content = files
                    .get(path)
                    .ok_or_else(|| crate::session_fs::SessionFsError::not_found(path))?;
                Ok(crate::session_fs::SessionFsFileInfo {
                    is_file: true,
                    is_directory: false,
                    size: content.len() as u64,
                    mtime: "2024-01-01T00:00:00.000Z".to_string(),
                    birthtime: "2024-01-01T00:00:00.000Z".to_string(),
                })
            })
        }

        fn readdir<'a>(
            &'a self,
            _path: &'a str,
        ) -> crate::session_fs::SessionFsFuture<'a, Vec<String>> {
            Box::pin(async move { Ok(self.files.lock().unwrap().keys().cloned().collect()) })
        }

        fn readdir_with_types<'a>(
            &'a self,
            _path: &'a str,
        ) -> crate::session_fs::SessionFsFuture<'a, Vec<crate::session_fs::SessionFsDirEntry>>
        {
            Box::pin(async move {
                Ok(self
                    .files
                    .lock()
                    .unwrap()
                    .keys()
                    .map(crate::session_fs::SessionFsDirEntry::file)
                    .collect())
            })
        }

        fn mkdir<'a>(
            &'a self,
            _path: &'a str,
            _recursive: bool,
            _mode: Option<u32>,
        ) -> crate::session_fs::SessionFsFuture<'a, ()> {
            Box::pin(async { Ok(()) })
        }

        fn rm<'a>(
            &'a self,
            path: &'a str,
            _recursive: bool,
            _force: bool,
        ) -> crate::session_fs::SessionFsFuture<'a, ()> {
            Box::pin(async move {
                self.files.lock().unwrap().remove(path);
                Ok(())
            })
        }

        fn rename<'a>(
            &'a self,
            src: &'a str,
            dest: &'a str,
        ) -> crate::session_fs::SessionFsFuture<'a, ()> {
            Box::pin(async move {
                let mut files = self.files.lock().unwrap();
                let content = files
                    .remove(src)
                    .ok_or_else(|| crate::session_fs::SessionFsError::not_found(src))?;
                files.insert(dest.to_string(), content);
                Ok(())
            })
        }
    }

    async fn stub_fs_sessions() -> RwLock<HashMap<String, Arc<Session>>> {
        let session = Arc::new(Session::new("s1".to_string(), None, noop_invoke));
        session
            .register_session_fs_provider(Arc::new(StubFs::default()))
            .await;
        test_sessions(session)
    }

    #[tokio::test]
    async fn test_session_fs_write_then_read() {
        let sessions = stub_fs_sessions().await;

        let written = handle_session_fs_request(
            &sessions,
            "sessionFs.writeFile",
            &json!({ "sessionId": "s1", "path": "/a.txt", "content": "hi" }),
        )
        .await
        .unwrap();
        assert_eq!(written, Value::Null);

        let read = handle_session_fs_request(
            &sessions,
            "sessionFs.readFile",
            &json!({ "sessionId": "s1", "path": "/a.txt" }),
        )
        .await
        .unwrap();
        assert_eq!(read["content"], "hi");
        assert!(read.get("error").is_none());
    }

    #[tokio::test]
    async fn test_session_fs_read_missing_returns_enoent() {
        let sessions = stub_fs_sessions().await;
        let read = handle_session_fs_request(
            &sessions,
            "sessionFs.readFile",
            &json!({ "sessionId": "s1", "path": "/missing" }),
        )
        .await
        .unwrap();
        assert_eq!(read["error"]["code"], "ENOENT");
    }

    #[tokio::test]
    async fn test_session_fs_exists_and_stat() {
        let sessions = stub_fs_sessions().await;
        handle_session_fs_request(
            &sessions,
            "sessionFs.writeFile",
            &json!({ "sessionId": "s1", "path": "/a.txt", "content": "abc" }),
        )
        .await
        .unwrap();

        let exists = handle_session_fs_request(
            &sessions,
            "sessionFs.exists",
            &json!({ "sessionId": "s1", "path": "/a.txt" }),
        )
        .await
        .unwrap();
        assert_eq!(exists["exists"], true);

        let stat = handle_session_fs_request(
            &sessions,
            "sessionFs.stat",
            &json!({ "sessionId": "s1", "path": "/a.txt" }),
        )
        .await
        .unwrap();
        assert_eq!(stat["isFile"], true);
        assert_eq!(stat["size"], 3);
    }

    #[tokio::test]
    async fn test_session_fs_append_uses_default_impl() {
        let sessions = stub_fs_sessions().await;
        for chunk in ["a", "b"] {
            handle_session_fs_request(
                &sessions,
                "sessionFs.appendFile",
                &json!({ "sessionId": "s1", "path": "/log", "content": chunk }),
            )
            .await
            .unwrap();
        }
        let read = handle_session_fs_request(
            &sessions,
            "sessionFs.readFile",
            &json!({ "sessionId": "s1", "path": "/log" }),
        )
        .await
        .unwrap();
        assert_eq!(read["content"], "ab");
    }

    #[tokio::test]
    async fn test_session_fs_rename_and_rm() {
        let sessions = stub_fs_sessions().await;
        handle_session_fs_request(
            &sessions,
            "sessionFs.writeFile",
            &json!({ "sessionId": "s1", "path": "/a", "content": "x" }),
        )
        .await
        .unwrap();

        let renamed = handle_session_fs_request(
            &sessions,
            "sessionFs.rename",
            &json!({ "sessionId": "s1", "src": "/a", "dest": "/b" }),
        )
        .await
        .unwrap();
        assert_eq!(renamed, Value::Null);

        let failed = handle_session_fs_request(
            &sessions,
            "sessionFs.rename",
            &json!({ "sessionId": "s1", "src": "/nope", "dest": "/c" }),
        )
        .await
        .unwrap();
        assert_eq!(failed["code"], "ENOENT");

        let removed = handle_session_fs_request(
            &sessions,
            "sessionFs.rm",
            &json!({ "sessionId": "s1", "path": "/b", "recursive": false, "force": true }),
        )
        .await
        .unwrap();
        assert_eq!(removed, Value::Null);
    }

    #[tokio::test]
    async fn test_session_fs_readdir_with_types_shape() {
        let sessions = stub_fs_sessions().await;
        handle_session_fs_request(
            &sessions,
            "sessionFs.writeFile",
            &json!({ "sessionId": "s1", "path": "/a", "content": "x" }),
        )
        .await
        .unwrap();

        let listing = handle_session_fs_request(
            &sessions,
            "sessionFs.readdirWithTypes",
            &json!({ "sessionId": "s1", "path": "/" }),
        )
        .await
        .unwrap();
        assert_eq!(listing["entries"][0]["name"], "/a");
        assert_eq!(listing["entries"][0]["type"], "file");
    }

    #[tokio::test]
    async fn test_session_fs_sqlite_absent_by_default() {
        let sessions = stub_fs_sessions().await;
        let exists = handle_session_fs_request(
            &sessions,
            "sessionFs.sqliteExists",
            &json!({ "sessionId": "s1" }),
        )
        .await
        .unwrap();
        assert_eq!(exists["exists"], false);

        let err = handle_session_fs_request(
            &sessions,
            "sessionFs.sqliteQuery",
            &json!({ "sessionId": "s1", "query": "SELECT 1", "queryType": "query" }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, CopilotError::Protocol(_)));
    }

    #[tokio::test]
    async fn test_session_fs_without_provider_is_an_error() {
        let session = Arc::new(Session::new("s1".to_string(), None, noop_invoke));
        let sessions = test_sessions(session);
        let err = handle_session_fs_request(
            &sessions,
            "sessionFs.readFile",
            &json!({ "sessionId": "s1", "path": "/a" }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, CopilotError::Protocol(_)));
    }

    #[tokio::test]
    async fn test_inject_trace_context_adds_headers() {
        let client = Client::builder()
            .on_get_trace_context(Arc::new(|| {
                Box::pin(async {
                    crate::trace::TraceContext {
                        traceparent: Some("00-abc-def-01".to_string()),
                        tracestate: Some("vendor=1".to_string()),
                    }
                })
            }))
            .build()
            .unwrap();

        let mut params = json!({ "sessionId": "s1" });
        client.inject_trace_context(&mut params).await;
        assert_eq!(params["traceparent"], "00-abc-def-01");
        assert_eq!(params["tracestate"], "vendor=1");
    }

    #[tokio::test]
    async fn test_inject_trace_context_noop_without_provider() {
        let client = Client::builder().build().unwrap();
        let mut params = json!({ "sessionId": "s1" });
        client.inject_trace_context(&mut params).await;
        assert_eq!(params, json!({ "sessionId": "s1" }));
    }

    #[tokio::test]
    async fn test_session_fs_unknown_method() {
        let sessions = stub_fs_sessions().await;
        let err = handle_session_fs_request(
            &sessions,
            "sessionFs.teleport",
            &json!({ "sessionId": "s1", "path": "/a" }),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, CopilotError::Protocol(_)));
    }
    // ---- Wave 3: wire defaults + session options patch ----

    #[test]
    fn test_apply_wire_session_defaults_injects_constants() {
        let mut params = json!({});
        apply_wire_session_defaults(&mut params, CopilotClientMode::CopilotCli, None);
        assert_eq!(params["toolFilterPrecedence"], "excluded");
        assert_eq!(params["envValueMode"], "direct");
        assert_eq!(params["includeSubAgentStreamingEvents"], true);
        assert_eq!(params["hooks"], false);
        // Non-empty mode must not lock down runtime features.
        assert!(params.get("enableSkills").is_none());
        assert!(params.get("enableSessionStore").is_none());
    }

    #[test]
    fn test_apply_wire_session_defaults_preserves_caller_values() {
        let mut params = json!({
            "toolFilterPrecedence": "available",
            "includeSubAgentStreamingEvents": false,
            "enableSkills": true,
        });
        apply_wire_session_defaults(&mut params, CopilotClientMode::Empty, None);
        assert_eq!(params["toolFilterPrecedence"], "available");
        assert_eq!(params["includeSubAgentStreamingEvents"], false);
        assert_eq!(params["enableSkills"], true);
    }

    #[test]
    fn test_apply_wire_session_defaults_empty_mode_locks_down_features() {
        let mut params = json!({});
        apply_wire_session_defaults(&mut params, CopilotClientMode::Empty, None);
        assert_eq!(params["enableSessionTelemetry"], false);
        assert_eq!(params["mcpOAuthTokenStorage"], "in-memory");
        assert_eq!(params["skipEmbeddingRetrieval"], true);
        assert_eq!(params["embeddingCacheStorage"], "in-memory");
        assert_eq!(params["enableOnDemandInstructionDiscovery"], false);
        assert_eq!(params["enableFileHooks"], false);
        assert_eq!(params["enableHostGitOperations"], false);
        assert_eq!(params["enableSessionStore"], false);
        assert_eq!(params["enableSkills"], false);
    }

    #[test]
    fn test_apply_wire_session_defaults_hooks_flag() {
        let hooks = SessionHooks {
            on_session_start: Some(Arc::new(|_| Default::default())),
            ..Default::default()
        };
        let mut params = json!({});
        apply_wire_session_defaults(&mut params, CopilotClientMode::CopilotCli, Some(&hooks));
        assert_eq!(params["hooks"], true);
    }

    #[test]
    fn test_session_options_patch_empty_mode_forces_defaults() {
        let patch =
            session_options_patch_for_mode(&SessionConfig::default(), CopilotClientMode::Empty);
        assert_eq!(patch.skip_custom_instructions, Some(true));
        assert_eq!(patch.custom_agents_local_only, Some(true));
        assert_eq!(patch.coauthor_enabled, Some(false));
        assert_eq!(patch.manage_schedule_enabled, Some(false));
        assert_eq!(patch.installed_plugins.as_deref(), Some(&[][..]));
    }

    #[test]
    fn test_session_options_patch_empty_mode_respects_explicit_values() {
        let config = SessionConfig {
            skip_custom_instructions: Some(false),
            coauthor_enabled: Some(true),
            ..Default::default()
        };
        let patch = session_options_patch_for_mode(&config, CopilotClientMode::Empty);
        assert_eq!(patch.skip_custom_instructions, Some(false));
        assert_eq!(patch.coauthor_enabled, Some(true));
        // Unset fields still get the empty-mode default.
        assert_eq!(patch.custom_agents_local_only, Some(true));
    }

    #[test]
    fn test_session_options_patch_cli_mode_passes_through() {
        let patch = session_options_patch_for_mode(
            &SessionConfig::default(),
            CopilotClientMode::CopilotCli,
        );
        assert!(patch.is_empty());

        let config = SessionConfig {
            manage_schedule_enabled: Some(true),
            ..Default::default()
        };
        let patch = session_options_patch_for_mode(&config, CopilotClientMode::CopilotCli);
        assert!(!patch.is_empty());
        assert_eq!(patch.manage_schedule_enabled, Some(true));
        assert!(patch.installed_plugins.is_none());
    }

    #[test]
    fn test_resume_options_patch_for_mode() {
        let patch = resume_options_patch_for_mode(
            &ResumeSessionConfig::default(),
            CopilotClientMode::CopilotCli,
        );
        assert!(patch.is_empty());

        let patch = resume_options_patch_for_mode(
            &ResumeSessionConfig::default(),
            CopilotClientMode::Empty,
        );
        assert_eq!(patch.custom_agents_local_only, Some(true));
        assert_eq!(patch.installed_plugins.as_deref(), Some(&[][..]));
    }

    #[test]
    fn test_resume_session_config_wave3_wire_names() {
        let config = ResumeSessionConfig {
            available_tools: Some(vec!["shell".into()]),
            excluded_tools: Some(vec!["write".into()]),
            reasoning_summary: Some(crate::types::ReasoningSummary::Detailed),
            enable_mcp_apps: Some(true),
            config_directory: Some("/cfg".into()),
            mcp_oauth_token_storage: Some(crate::types::StorageMode::InMemory),
            continue_pending_work: Some(true),
            github_token: Some("tok".into()),
            remote_session: Some(crate::types::RemoteSessionMode::On),
            open_canvases: Some(vec![crate::canvas::OpenCanvasInstance {
                instance_id: "i1".into(),
                extension_id: "e1".into(),
                extension_name: None,
                canvas_id: "c1".into(),
                title: None,
                status: None,
                url: None,
                input: None,
                reopen: true,
                availability: crate::canvas::CanvasInstanceAvailability::Ready,
            }]),
            ..Default::default()
        };
        let v = serde_json::to_value(&config).unwrap();
        assert_eq!(v["availableTools"][0], "shell");
        assert_eq!(v["excludedTools"][0], "write");
        assert_eq!(v["reasoningSummary"], "detailed");
        assert_eq!(v["requestMcpApps"], true);
        assert_eq!(v["configDir"], "/cfg");
        assert_eq!(v["mcpOAuthTokenStorage"], "in-memory");
        assert_eq!(v["continuePendingWork"], true);
        assert_eq!(v["gitHubToken"], "tok");
        assert_eq!(v["remoteSession"], "on");
        assert_eq!(v["openCanvases"][0]["instanceId"], "i1");
        assert_eq!(v["openCanvases"][0]["availability"], "ready");
        // Post-resume options are never sent on the resume request itself.
        assert!(v.get("skipCustomInstructions").is_none());
        assert!(v.get("coauthorEnabled").is_none());
    }
}
