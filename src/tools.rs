// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Tool definition utilities for the Copilot SDK.
//!
//! Provides convenience functions for defining tools with automatic
//! result normalization and error handling.

use crate::types::{Tool, ToolBinaryResult, ToolResultObject};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single content block within an MCP `CallToolResult`.
///
/// Tagged on the `type` discriminator, mirroring the MCP wire shape consumed by
/// [`convert_mcp_call_tool_result`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum McpCallToolResultContent {
    /// A text content block.
    Text {
        /// The text payload.
        text: String,
    },
    /// An inline image content block.
    Image {
        /// Base64-encoded image data.
        data: String,
        /// The image MIME type.
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    /// An embedded resource content block.
    Resource {
        /// The embedded resource.
        resource: McpCallToolResultResource,
    },
}

/// The embedded resource inside an MCP `resource` content block.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpCallToolResultResource {
    /// The resource URI.
    #[serde(default)]
    pub uri: String,
    /// Optional MIME type.
    #[serde(rename = "mimeType", default, skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    /// Optional inline text.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Optional base64-encoded binary blob.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blob: Option<String>,
}

/// An MCP-compatible `CallToolResult`. Pass to [`convert_mcp_call_tool_result`]
/// to produce a [`ToolResultObject`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McpCallToolResult {
    /// The content blocks.
    #[serde(default)]
    pub content: Vec<McpCallToolResultContent>,
    /// Whether the call resulted in an error.
    #[serde(rename = "isError", default, skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

/// Converts an MCP `CallToolResult` into the SDK's [`ToolResultObject`] format.
///
/// Text blocks (and resource text) are concatenated with `\n`; image blocks and
/// resource blobs become binary results. `is_error: true` maps the result type
/// to `"failure"`.
pub fn convert_mcp_call_tool_result(call_result: &McpCallToolResult) -> ToolResultObject {
    let mut text_parts: Vec<String> = Vec::new();
    let mut binary_results: Vec<ToolBinaryResult> = Vec::new();

    for block in &call_result.content {
        match block {
            McpCallToolResultContent::Text { text } => {
                text_parts.push(text.clone());
            }
            McpCallToolResultContent::Image { data, mime_type } => {
                if !data.is_empty() && !mime_type.is_empty() {
                    binary_results.push(ToolBinaryResult {
                        data: data.clone(),
                        mime_type: mime_type.clone(),
                        result_type: "image".to_string(),
                        description: None,
                    });
                }
            }
            McpCallToolResultContent::Resource { resource } => {
                if let Some(text) = resource.text.as_ref().filter(|t| !t.is_empty()) {
                    text_parts.push(text.clone());
                }
                if let Some(blob) = resource.blob.as_ref().filter(|b| !b.is_empty()) {
                    let mime_type = resource
                        .mime_type
                        .as_ref()
                        .filter(|m| !m.is_empty())
                        .cloned()
                        .unwrap_or_else(|| "application/octet-stream".to_string());
                    binary_results.push(ToolBinaryResult {
                        data: blob.clone(),
                        mime_type,
                        result_type: "resource".to_string(),
                        description: Some(resource.uri.clone()),
                    });
                }
            }
        }
    }

    ToolResultObject {
        text_result_for_llm: text_parts.join("\n"),
        binary_results_for_llm: if binary_results.is_empty() {
            None
        } else {
            Some(binary_results)
        },
        result_type: if call_result.is_error.unwrap_or(false) {
            "failure".to_string()
        } else {
            "success".to_string()
        },
        error: None,
        session_log: None,
        tool_telemetry: None,
    }
}

/// Normalize any result into a ToolResultObject.
///
/// - `None` / null → empty success
/// - `String` → success with text
/// - `ToolResultObject` (dict with resultType + textResultForLlm) → pass-through
/// - Everything else → JSON serialize
pub fn normalize_result(result: Value) -> ToolResultObject {
    match result {
        Value::Null => ToolResultObject {
            text_result_for_llm: String::new(),
            binary_results_for_llm: None,
            result_type: "success".to_string(),
            error: None,
            session_log: None,
            tool_telemetry: None,
        },
        Value::String(s) => ToolResultObject {
            text_result_for_llm: s,
            binary_results_for_llm: None,
            result_type: "success".to_string(),
            error: None,
            session_log: None,
            tool_telemetry: None,
        },
        Value::Object(ref map)
            if map.contains_key("resultType") && map.contains_key("textResultForLlm") =>
        {
            serde_json::from_value(result).unwrap_or_else(|_| ToolResultObject {
                text_result_for_llm: "Failed to parse tool result".to_string(),
                binary_results_for_llm: None,
                result_type: "failure".to_string(),
                error: None,
                session_log: None,
                tool_telemetry: None,
            })
        }
        other => ToolResultObject {
            text_result_for_llm: serde_json::to_string(&other).unwrap_or_default(),
            binary_results_for_llm: None,
            result_type: "success".to_string(),
            error: None,
            session_log: None,
            tool_telemetry: None,
        },
    }
}

/// Define a tool with metadata for registration on a session.
///
/// Returns a `Tool` struct with name, description, and parameters schema.
/// The handler must be registered separately on the session via
/// `session.register_tool_with_handler()`.
///
/// # Example
/// ```rust,no_run
/// use copilot_sdk::tools::define_tool;
/// use serde_json::json;
///
/// let tool = define_tool(
///     "my_tool",
///     "A description of my tool",
///     Some(json!({"type": "object", "properties": {"query": {"type": "string"}}})),
/// );
/// // Register on session: session.register_tool_with_handler(tool, Some(handler)).await;
/// ```
pub fn define_tool(name: &str, description: &str, parameters_schema: Option<Value>) -> Tool {
    Tool {
        name: name.to_string(),
        description: description.to_string(),
        parameters_schema: parameters_schema.unwrap_or(serde_json::json!({})),
        overrides_built_in_tool: false,
        skip_permission: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_normalize_null() {
        let result = normalize_result(Value::Null);
        assert_eq!(result.result_type, "success");
        assert_eq!(result.text_result_for_llm, "");
    }

    #[test]
    fn test_normalize_string() {
        let result = normalize_result(Value::String("hello".to_string()));
        assert_eq!(result.result_type, "success");
        assert_eq!(result.text_result_for_llm, "hello");
    }

    #[test]
    fn test_normalize_tool_result_passthrough() {
        let val = json!({
            "resultType": "success",
            "textResultForLlm": "tool output"
        });
        let result = normalize_result(val);
        assert_eq!(result.result_type, "success");
        assert_eq!(result.text_result_for_llm, "tool output");
    }

    #[test]
    fn test_normalize_other_value() {
        let val = json!({"key": "value"});
        let result = normalize_result(val);
        assert_eq!(result.result_type, "success");
        assert!(result.text_result_for_llm.contains("key"));
    }

    #[test]
    fn test_define_tool_basic() {
        let tool = define_tool("test_tool", "A test tool", None);
        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "A test tool");
    }

    #[test]
    fn test_define_tool_with_schema() {
        let schema = json!({"type": "object", "properties": {"q": {"type": "string"}}});
        let tool = define_tool("search", "Search tool", Some(schema.clone()));
        assert_eq!(tool.name, "search");
        assert_eq!(tool.parameters_schema, schema);
    }

    #[test]
    fn test_convert_mcp_text_and_error() {
        let result = McpCallToolResult {
            content: vec![
                McpCallToolResultContent::Text {
                    text: "line one".to_string(),
                },
                McpCallToolResultContent::Text {
                    text: "line two".to_string(),
                },
            ],
            is_error: Some(true),
        };
        let converted = convert_mcp_call_tool_result(&result);
        assert_eq!(converted.text_result_for_llm, "line one\nline two");
        assert_eq!(converted.result_type, "failure");
        assert!(converted.binary_results_for_llm.is_none());
    }

    #[test]
    fn test_convert_mcp_image_and_resource() {
        let result = McpCallToolResult {
            content: vec![
                McpCallToolResultContent::Image {
                    data: "aGk=".to_string(),
                    mime_type: "image/png".to_string(),
                },
                McpCallToolResultContent::Resource {
                    resource: McpCallToolResultResource {
                        uri: "file:///a.bin".to_string(),
                        mime_type: None,
                        text: Some("resource text".to_string()),
                        blob: Some("YmxvYg==".to_string()),
                    },
                },
            ],
            is_error: None,
        };
        let converted = convert_mcp_call_tool_result(&result);
        assert_eq!(converted.result_type, "success");
        assert_eq!(converted.text_result_for_llm, "resource text");
        let bins = converted.binary_results_for_llm.unwrap();
        assert_eq!(bins.len(), 2);
        assert_eq!(bins[0].result_type, "image");
        assert_eq!(bins[1].result_type, "resource");
        assert_eq!(bins[1].mime_type, "application/octet-stream");
        assert_eq!(bins[1].description.as_deref(), Some("file:///a.bin"));
    }

    #[test]
    fn test_convert_mcp_parses_from_json() {
        let value = json!({
            "content": [{ "type": "text", "text": "hello" }],
            "isError": false
        });
        let result: McpCallToolResult = serde_json::from_value(value).unwrap();
        let converted = convert_mcp_call_tool_result(&result);
        assert_eq!(converted.text_result_for_llm, "hello");
        assert_eq!(converted.result_type, "success");
    }
}
