// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Extension-owned canvas declarations.
//!
//! Mirrors the Node.js `canvas` module's declarative wire surface. A canvas is
//! declared on `session.create` / `session.resume`; the runtime then drives it
//! with `canvas.open`, `canvas.close`, and `canvas.action.invoke` JSON-RPC
//! requests routed by `canvasId`.
//!
//! This module provides the **declarative model** (the serialized wire shape
//! sent to the runtime) plus [`CanvasError`]. Node.js co-locates handler
//! closures via a per-canvas `createCanvas` factory; the Rust SDK follows the
//! single-`CanvasHandler`-per-session pattern used by the Python, Go, and .NET
//! SDKs, where handler dispatch switches on `canvasId`. Both target the same
//! JSON-RPC wire protocol.
//!
//! # Example
//! ```
//! use copilot_sdk::canvas::{CanvasBuilder, CanvasActionDeclaration};
//! use serde_json::json;
//!
//! let canvas = CanvasBuilder::new("charts", "Charts", "Renders charts")
//!     .input_schema(json!({ "type": "object" }))
//!     .action(CanvasActionDeclaration::new("refresh").description("Refresh data"))
//!     .build();
//! assert_eq!(canvas.id, "charts");
//! assert_eq!(canvas.actions.as_ref().unwrap().len(), 1);
//! ```

use serde::{Deserialize, Serialize};

/// JSON Schema value describing a canvas `input` or action-`input` payload.
pub type CanvasJsonSchema = serde_json::Value;

/// Wire metadata for a single agent-callable canvas action. Names MUST NOT
/// start with `canvas.` — that prefix is reserved for lifecycle verbs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasActionDeclaration {
    /// Action identifier, unique within the canvas.
    pub name: String,
    /// Description shown to the model when picking an action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional JSON Schema for the action's `input` payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<CanvasJsonSchema>,
}

impl CanvasActionDeclaration {
    /// Create an action declaration with just a name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            input_schema: None,
        }
    }

    /// Set the action description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the action input schema.
    pub fn input_schema(mut self, schema: CanvasJsonSchema) -> Self {
        self.input_schema = Some(schema);
        self
    }
}

/// Declarative metadata for a single canvas, serialized over the wire on
/// `session.create` / `session.resume`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasDeclaration {
    /// Canvas id, unique within the declaring connection.
    pub id: String,
    /// Human-readable label shown in discovery and host UI chrome.
    pub display_name: String,
    /// Short, single-sentence description shown to the agent in canvas catalogs.
    pub description: String,
    /// Optional JSON Schema for the `input` payload accepted by `canvas.open`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<CanvasJsonSchema>,
    /// Agent-invocable actions exposed via `invoke_canvas_action`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actions: Option<Vec<CanvasActionDeclaration>>,
}

/// Whether a restored canvas instance is still usable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CanvasInstanceAvailability {
    /// The declaring provider is connected and the canvas can be re-rendered.
    Ready,
    /// The instance was restored from history but its provider is unavailable.
    Stale,
}

/// A canvas instance that is currently open in the host, as reported by
/// `session.resume` and `session.create`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCanvasInstance {
    /// Stable caller-supplied canvas instance identifier.
    pub instance_id: String,
    /// Owning provider identifier.
    pub extension_id: String,
    /// Owning extension display name, when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extension_name: Option<String>,
    /// Provider-local canvas identifier.
    pub canvas_id: String,
    /// Rendered title.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Provider-supplied status text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// URL for web-rendered canvases.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Input supplied when the instance was opened.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    /// Whether this snapshot came from an idempotent reopen.
    pub reopen: bool,
    /// Whether the instance is still backed by a live provider.
    pub availability: CanvasInstanceAvailability,
}

/// Fluent builder that produces a [`CanvasDeclaration`].
#[derive(Debug, Clone)]
pub struct CanvasBuilder {
    declaration: CanvasDeclaration,
}

impl CanvasBuilder {
    /// Start building a canvas declaration.
    pub fn new(
        id: impl Into<String>,
        display_name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            declaration: CanvasDeclaration {
                id: id.into(),
                display_name: display_name.into(),
                description: description.into(),
                input_schema: None,
                actions: None,
            },
        }
    }

    /// Set the canvas open-input schema.
    pub fn input_schema(mut self, schema: CanvasJsonSchema) -> Self {
        self.declaration.input_schema = Some(schema);
        self
    }

    /// Add an agent-invocable action.
    pub fn action(mut self, action: CanvasActionDeclaration) -> Self {
        self.declaration
            .actions
            .get_or_insert_with(Vec::new)
            .push(action);
        self
    }

    /// Consume the builder and return the [`CanvasDeclaration`].
    pub fn build(self) -> CanvasDeclaration {
        self.declaration
    }
}

/// Structured error returned from a canvas handler.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasError {
    /// Machine-readable error code.
    pub code: String,
    /// Human-readable error message.
    pub message: String,
}

impl CanvasError {
    /// Create a canvas error with an explicit code and message.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Default error when an action is declared but no handler is wired.
    pub fn no_handler() -> Self {
        Self::new(
            "canvas_action_no_handler",
            "No handler implemented for this canvas action",
        )
    }
}

impl std::fmt::Display for CanvasError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for CanvasError {}

// =============================================================================
// Canvas provider wire types (inbound `canvas.*` requests)
// =============================================================================

/// Parameters for an inbound `canvas.open` request from the runtime.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasOpenRequest {
    /// Target session identifier.
    pub session_id: String,
    /// Owning provider/extension identifier.
    #[serde(default)]
    pub extension_id: String,
    /// Provider-local canvas identifier.
    pub canvas_id: String,
    /// Stable caller-supplied canvas instance identifier.
    pub instance_id: String,
    /// Canvas open input payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    /// Host context supplied by the runtime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<serde_json::Value>,
    /// Session context supplied by the runtime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<serde_json::Value>,
}

/// Parameters for an inbound `canvas.close` request from the runtime.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasCloseRequest {
    /// Target session identifier.
    pub session_id: String,
    /// Owning provider/extension identifier.
    #[serde(default)]
    pub extension_id: String,
    /// Provider-local canvas identifier.
    pub canvas_id: String,
    /// Canvas instance identifier.
    pub instance_id: String,
    /// Host context supplied by the runtime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<serde_json::Value>,
    /// Session context supplied by the runtime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<serde_json::Value>,
}

/// Parameters for an inbound `canvas.action.invoke` request from the runtime.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasInvokeActionRequest {
    /// Target session identifier.
    pub session_id: String,
    /// Owning provider/extension identifier.
    #[serde(default)]
    pub extension_id: String,
    /// Provider-local canvas identifier.
    pub canvas_id: String,
    /// Canvas instance identifier.
    pub instance_id: String,
    /// Action name to invoke.
    pub action_name: String,
    /// Action input payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    /// Host context supplied by the runtime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<serde_json::Value>,
    /// Session context supplied by the runtime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session: Option<serde_json::Value>,
}

/// Result returned from a `canvas.open` handler.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CanvasOpenResult {
    /// Provider-supplied status text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Provider-supplied title shown in host chrome.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Handler for inbound `canvas.*` reverse-RPC requests.
///
/// A single handler per session dispatches all canvas lifecycle verbs, switching
/// on [`CanvasOpenRequest::canvas_id`] as needed. Register via
/// [`Session::register_canvas_handler`](crate::Session::register_canvas_handler).
/// Mirrors the Node.js per-canvas `createCanvas` factory against the same wire
/// protocol (`canvas.open`, `canvas.close`, `canvas.action.invoke`).
pub trait CanvasHandler: Send + Sync {
    /// Handle a `canvas.open` request. Returns provider metadata for the host.
    fn on_open(
        &self,
        request: CanvasOpenRequest,
    ) -> std::result::Result<CanvasOpenResult, CanvasError>;

    /// Handle a `canvas.close` request.
    fn on_close(&self, request: CanvasCloseRequest) -> std::result::Result<(), CanvasError> {
        let _ = request;
        Ok(())
    }

    /// Handle a `canvas.action.invoke` request. Returns the action result payload.
    fn on_action(
        &self,
        request: CanvasInvokeActionRequest,
    ) -> std::result::Result<serde_json::Value, CanvasError> {
        let _ = request;
        Err(CanvasError::no_handler())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn builds_declaration_with_actions() {
        let canvas = CanvasBuilder::new("charts", "Charts", "Renders charts")
            .input_schema(json!({ "type": "object" }))
            .action(
                CanvasActionDeclaration::new("refresh")
                    .description("Refresh data")
                    .input_schema(json!({ "type": "object" })),
            )
            .build();

        assert_eq!(canvas.id, "charts");
        assert_eq!(canvas.display_name, "Charts");
        let actions = canvas.actions.unwrap();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].name, "refresh");
        assert_eq!(actions[0].description.as_deref(), Some("Refresh data"));
    }

    #[test]
    fn declaration_omits_empty_optionals() {
        let canvas = CanvasBuilder::new("a", "A", "desc").build();
        let value = serde_json::to_value(&canvas).unwrap();
        assert!(value.get("inputSchema").is_none());
        assert!(value.get("actions").is_none());
        assert_eq!(value["displayName"], "A");
    }

    #[test]
    fn canvas_error_no_handler_default() {
        let err = CanvasError::no_handler();
        assert_eq!(err.code, "canvas_action_no_handler");
        assert!(err.to_string().contains("No handler"));
    }
}
