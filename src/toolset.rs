// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Source-qualified tool filtering for [`SessionConfig::available_tools`].
//!
//! Mirrors the Node.js `ToolSet` builder and `BuiltInTools` constants. Tools
//! are classified by the runtime at registration time (not from name parsing),
//! so `add_built_in("foo")` matches only tools the runtime registered as
//! built-in, even if an MCP server or custom-agent extension happens to
//! register a tool with the same wire name.
//!
//! # Example
//! ```
//! use copilot_sdk::{ToolSet, BuiltInTools};
//!
//! let tools = ToolSet::new()
//!     .add_built_in_many(BuiltInTools::ISOLATED)
//!     .add_mcp("*")
//!     .add_custom("*")
//!     .into_vec();
//! assert!(tools.contains(&"mcp:*".to_string()));
//! ```
//!
//! [`SessionConfig::available_tools`]: crate::types::SessionConfig::available_tools

/// Error returned when a tool name passed to [`ToolSet`] is not the wildcard
/// `*` and does not match the runtime's `^[a-zA-Z0-9_-]+$` name rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidToolName {
    /// The classification the name was added under (`builtin`, `mcp`, `custom`).
    pub kind: &'static str,
    /// The offending name.
    pub name: String,
}

impl std::fmt::Display for InvalidToolName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Invalid {} tool name '{}': tool names must match /^[a-zA-Z0-9_-]+$/ or be the wildcard '*'.",
            self.kind, self.name
        )
    }
}

impl std::error::Error for InvalidToolName {}

fn is_valid_tool_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// Builder that produces a list of source-qualified tool filter strings for
/// [`SessionConfig::available_tools`](crate::types::SessionConfig::available_tools).
///
/// The infallible `add_*` methods silently skip invalid names (matching the
/// ergonomics of a fluent builder); use the `try_add_*` variants to surface a
/// [`InvalidToolName`] error at the SDK boundary instead.
#[derive(Debug, Clone, Default)]
pub struct ToolSet {
    items: Vec<String>,
}

impl ToolSet {
    /// Create an empty tool set.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add a single built-in tool pattern (a specific name or `"*"`).
    /// Invalid names are skipped; use [`ToolSet::try_add_built_in`] to detect them.
    pub fn add_built_in(mut self, name: &str) -> Self {
        let _ = self.push("builtin", name);
        self
    }

    /// Add multiple built-in tool patterns (e.g. [`BuiltInTools::ISOLATED`]).
    pub fn add_built_in_many<I, S>(mut self, names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        for name in names {
            let _ = self.push("builtin", name.as_ref());
        }
        self
    }

    /// Add a custom tool pattern (a specific name or `"*"`).
    pub fn add_custom(mut self, name: &str) -> Self {
        let _ = self.push("custom", name);
        self
    }

    /// Add an MCP tool pattern (the canonical wire name or `"*"`).
    pub fn add_mcp(mut self, tool_name: &str) -> Self {
        let _ = self.push("mcp", tool_name);
        self
    }

    /// Fallible variant of [`ToolSet::add_built_in`].
    pub fn try_add_built_in(mut self, name: &str) -> Result<Self, InvalidToolName> {
        self.push("builtin", name)?;
        Ok(self)
    }

    /// Fallible variant of [`ToolSet::add_custom`].
    pub fn try_add_custom(mut self, name: &str) -> Result<Self, InvalidToolName> {
        self.push("custom", name)?;
        Ok(self)
    }

    /// Fallible variant of [`ToolSet::add_mcp`].
    pub fn try_add_mcp(mut self, tool_name: &str) -> Result<Self, InvalidToolName> {
        self.push("mcp", tool_name)?;
        Ok(self)
    }

    fn push(&mut self, kind: &'static str, name: &str) -> Result<(), InvalidToolName> {
        if name != "*" && !is_valid_tool_name(name) {
            return Err(InvalidToolName {
                kind,
                name: name.to_string(),
            });
        }
        self.items.push(format!("{kind}:{name}"));
        Ok(())
    }

    /// Returns a defensive copy of the accumulated filter strings, suitable for
    /// passing as [`SessionConfig::available_tools`](crate::types::SessionConfig::available_tools).
    pub fn to_vec(&self) -> Vec<String> {
        self.items.clone()
    }

    /// Consumes the builder and returns the accumulated filter strings.
    pub fn into_vec(self) -> Vec<String> {
        self.items
    }
}

impl From<ToolSet> for Vec<String> {
    fn from(set: ToolSet) -> Self {
        set.items
    }
}

/// Curated sets of built-in tool names for common scenarios. Each constant is
/// meant to be passed to [`ToolSet::add_built_in_many`].
pub struct BuiltInTools;

impl BuiltInTools {
    /// Built-in tools that operate only within the bounds of a single session:
    /// no host filesystem access outside the session, no cross-session state,
    /// no host environment access, no network. Safe to enable in
    /// `Mode = "empty"` scenarios (e.g. multi-tenant servers) without leaking
    /// host capabilities.
    pub const ISOLATED: &'static [&'static str] = &[
        "ask_user",
        "task_complete",
        "exit_plan_mode",
        "task",
        "read_agent",
        "write_agent",
        "list_agents",
        "send_inbox",
        "context_board",
        "skill",
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_source_qualified_filters() {
        let tools = ToolSet::new()
            .add_built_in("bash")
            .add_mcp("github-list_issues")
            .add_custom("my_tool")
            .into_vec();
        assert_eq!(
            tools,
            vec!["builtin:bash", "mcp:github-list_issues", "custom:my_tool"]
        );
    }

    #[test]
    fn wildcard_is_allowed() {
        let tools = ToolSet::new().add_built_in("*").add_mcp("*").into_vec();
        assert_eq!(tools, vec!["builtin:*", "mcp:*"]);
    }

    #[test]
    fn add_built_in_many_expands_isolated() {
        let tools = ToolSet::new()
            .add_built_in_many(BuiltInTools::ISOLATED)
            .into_vec();
        assert_eq!(tools.len(), BuiltInTools::ISOLATED.len());
        assert!(tools.contains(&"builtin:exit_plan_mode".to_string()));
        assert!(tools.contains(&"builtin:skill".to_string()));
    }

    #[test]
    fn invalid_name_is_skipped_in_infallible_api() {
        let tools = ToolSet::new().add_built_in("bad name!").into_vec();
        assert!(tools.is_empty());
    }

    #[test]
    fn invalid_name_errors_in_fallible_api() {
        let err = ToolSet::new().try_add_mcp("bad name!").unwrap_err();
        assert_eq!(err.kind, "mcp");
        assert_eq!(err.name, "bad name!");
    }

    #[test]
    fn into_vec_via_from() {
        let v: Vec<String> = ToolSet::new().add_custom("x").into();
        assert_eq!(v, vec!["custom:x"]);
    }
}
