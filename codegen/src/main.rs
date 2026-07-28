// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Code generator for the `copilot-sdk` crate.
//!
//! Reads the vendored `schemas/api.schema.json` (shipped inside the
//! `@github/copilot` npm package) and emits `src/generated/methods.rs`, a
//! registry of every JSON-RPC method the Copilot CLI exposes.
//!
//! Run with `cd codegen && cargo run`.

use serde_json::Value;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::PathBuf;

/// One RPC method discovered in the schema.
#[derive(Debug, Clone)]
struct Method {
    wire: String,
    surface: &'static str,
    stability: String,
    visibility: String,
    description: String,
    has_params: bool,
    has_result: bool,
}

fn main() {
    let root = repo_root();
    let schema_path = root.join("schemas/api.schema.json");
    let raw = std::fs::read_to_string(&schema_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", schema_path.display()));
    let schema: Value = serde_json::from_str(&raw).expect("schema is not valid JSON");

    let mut methods = Vec::new();
    for surface in ["server", "session", "clientSession"] {
        let node = schema
            .get(surface)
            .unwrap_or_else(|| panic!("schema missing `{surface}` root"));
        collect(node, surface, &mut methods);
    }
    methods.sort_by(|a, b| a.wire.cmp(&b.wire));

    let unique: BTreeSet<&str> = methods.iter().map(|m| m.wire.as_str()).collect();
    assert_eq!(
        unique.len(),
        methods.len(),
        "duplicate rpcMethod values in schema"
    );

    let out = render(&schema, &methods);
    let out_dir = root.join("src/generated");
    std::fs::create_dir_all(&out_dir).expect("create src/generated");
    let out_path = out_dir.join("methods.rs");
    std::fs::write(&out_path, out).expect("write methods.rs");

    // Normalize through rustfmt so the emitted file is byte-identical to what
    // `cargo fmt` would produce. Without this the CI drift check would fail
    // forever: rustfmt rewraps long constant lines that the renderer emits on
    // a single line.
    let fmt = std::process::Command::new("rustfmt")
        .arg("--edition")
        .arg("2021")
        .arg(&out_path)
        .status()
        .expect("run rustfmt (is the rustfmt component installed?)");
    assert!(fmt.success(), "rustfmt failed on {}", out_path.display());

    println!(
        "generated {} ({} methods: {} server, {} session, {} clientSession)",
        out_path.display(),
        methods.len(),
        methods.iter().filter(|m| m.surface == "server").count(),
        methods.iter().filter(|m| m.surface == "session").count(),
        methods.iter().filter(|m| m.surface == "clientSession").count(),
    );
}

/// Locate the crate root whether run from `codegen/` or the repo root.
fn repo_root() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if manifest.join("../schemas/api.schema.json").exists() {
        return manifest.parent().expect("codegen has a parent").to_path_buf();
    }
    manifest
}

/// Recursively collect every node carrying an `rpcMethod` key.
fn collect(node: &Value, surface: &'static str, out: &mut Vec<Method>) {
    let Some(map) = node.as_object() else { return };
    for value in map.values() {
        let Some(obj) = value.as_object() else { continue };
        if let Some(wire) = obj.get("rpcMethod").and_then(|v| v.as_str()) {
            let has_result = obj
                .get("result")
                .map(|r| r.get("type").and_then(|t| t.as_str()) != Some("null"))
                .unwrap_or(false);
            out.push(Method {
                wire: wire.to_string(),
                surface,
                stability: obj
                    .get("stability")
                    .and_then(|v| v.as_str())
                    .unwrap_or("stable")
                    .to_string(),
                visibility: obj
                    .get("visibility")
                    .and_then(|v| v.as_str())
                    .unwrap_or("public")
                    .to_string(),
                description: obj
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                has_params: obj.contains_key("params"),
                has_result,
            });
        } else {
            collect(value, surface, out);
        }
    }
}

/// `model.getCurrent` -> `MODEL_GET_CURRENT`
fn screaming_snake(input: &str) -> String {
    let mut out = String::new();
    let mut prev_lower_or_digit = false;
    for ch in input.chars() {
        if ch == '.' || ch == '-' {
            out.push('_');
            prev_lower_or_digit = false;
            continue;
        }
        if ch.is_ascii_uppercase() {
            if prev_lower_or_digit {
                out.push('_');
            }
            out.push(ch);
            prev_lower_or_digit = false;
        } else {
            out.push(ch.to_ascii_uppercase());
            prev_lower_or_digit = ch.is_ascii_lowercase() || ch.is_ascii_digit();
        }
    }
    out
}

/// Constant identifier for a method, relative to its surface module.
fn const_name(m: &Method) -> String {
    let rel = if m.surface == "session" {
        m.wire.strip_prefix("session.").unwrap_or(&m.wire)
    } else {
        &m.wire
    };
    screaming_snake(rel)
}

fn module_for(surface: &str) -> &'static str {
    match surface {
        "server" => "server",
        "session" => "session",
        "clientSession" => "client_session",
        other => panic!("unknown surface {other}"),
    }
}

/// Collapse a description into a single safe rustdoc line.
fn doc_line(s: &str) -> String {
    let flat = s.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.is_empty() {
        "No description provided by the schema.".to_string()
    } else {
        flat
    }
}

fn render(schema: &Value, methods: &[Method]) -> String {
    let title = schema.get("title").and_then(|v| v.as_str()).unwrap_or("CopilotApi");
    let mut s = String::new();

    writeln!(s, "// Copyright (c) 2026 Elias Bachaalany").unwrap();
    writeln!(s, "// SPDX-License-Identifier: MIT").unwrap();
    writeln!(s).unwrap();
    writeln!(s, "// @generated by `codegen` from `schemas/api.schema.json` - DO NOT EDIT.").unwrap();
    writeln!(s, "// Regenerate with `cd codegen && cargo run`.").unwrap();
    writeln!(s).unwrap();
    writeln!(s, "//! Registry of every JSON-RPC method exposed by the Copilot CLI.").unwrap();
    writeln!(s, "//!").unwrap();
    writeln!(s, "//! Generated from the `{title}` schema that ships inside the").unwrap();
    writeln!(s, "//! `@github/copilot` npm package. Using these constants instead of string").unwrap();
    writeln!(s, "//! literals guarantees outbound method names match the wire protocol.").unwrap();
    writeln!(s).unwrap();

    // ---- Stability ----
    writeln!(s, "/// Stability contract the CLI advertises for a method.").unwrap();
    writeln!(s, "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]").unwrap();
    writeln!(s, "pub enum Stability {{").unwrap();
    writeln!(s, "    /// Covered by the SDK's compatibility guarantees.").unwrap();
    writeln!(s, "    Stable,").unwrap();
    writeln!(s, "    /// May change or be removed without a major version bump.").unwrap();
    writeln!(s, "    Experimental,").unwrap();
    writeln!(s, "}}").unwrap();
    writeln!(s).unwrap();

    // ---- Surface ----
    writeln!(s, "/// Which JSON-RPC surface a method belongs to.").unwrap();
    writeln!(s, "#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]").unwrap();
    writeln!(s, "pub enum Surface {{").unwrap();
    writeln!(s, "    /// Client-to-server methods that are not scoped to a session.").unwrap();
    writeln!(s, "    Server,").unwrap();
    writeln!(s, "    /// Client-to-server methods scoped to a single session.").unwrap();
    writeln!(s, "    Session,").unwrap();
    writeln!(s, "    /// Reverse RPC: server-to-client callbacks the SDK must serve.").unwrap();
    writeln!(s, "    ClientSession,").unwrap();
    writeln!(s, "}}").unwrap();
    writeln!(s).unwrap();

    // ---- MethodInfo ----
    writeln!(s, "/// Schema-derived metadata describing a single RPC method.").unwrap();
    writeln!(s, "#[derive(Debug, Clone, Copy)]").unwrap();
    writeln!(s, "pub struct MethodInfo {{").unwrap();
    writeln!(s, "    /// Exact wire name, e.g. `session.model.getCurrent`.").unwrap();
    writeln!(s, "    pub wire_name: &'static str,").unwrap();
    writeln!(s, "    /// Surface the method is served on.").unwrap();
    writeln!(s, "    pub surface: Surface,").unwrap();
    writeln!(s, "    /// Stability contract advertised by the schema.").unwrap();
    writeln!(s, "    pub stability: Stability,").unwrap();
    writeln!(s, "    /// Whether the schema marks the method as internal-only.").unwrap();
    writeln!(s, "    pub internal: bool,").unwrap();
    writeln!(s, "    /// Whether the method accepts a params object.").unwrap();
    writeln!(s, "    pub has_params: bool,").unwrap();
    writeln!(s, "    /// Whether the method returns a non-null result.").unwrap();
    writeln!(s, "    pub has_result: bool,").unwrap();
    writeln!(s, "}}").unwrap();
    writeln!(s).unwrap();

    // ---- per-surface constant modules ----
    for surface in ["server", "session", "clientSession"] {
        let list: Vec<&Method> = methods.iter().filter(|m| m.surface == surface).collect();
        let module = module_for(surface);
        let human = match surface {
            "server" => "server-level (non session-scoped)",
            "session" => "session-scoped",
            _ => "reverse-RPC (server-to-client)",
        };
        writeln!(s, "/// Wire names for {human} methods.").unwrap();
        if surface == "session" {
            writeln!(s, "///").unwrap();
            writeln!(s, "/// Constant names drop the redundant `session.` prefix.").unwrap();
        }
        writeln!(s, "pub mod {module} {{").unwrap();
        for m in &list {
            writeln!(s, "    /// {}", doc_line(&m.description)).unwrap();
            writeln!(s, "    ///").unwrap();
            writeln!(s, "    /// Wire name: `{}`", m.wire).unwrap();
            writeln!(
                s,
                "    pub const {}: &str = \"{}\";",
                const_name(m),
                m.wire
            )
            .unwrap();
        }
        writeln!(s, "}}").unwrap();
        writeln!(s).unwrap();
    }

    // ---- ALL_METHODS ----
    writeln!(s, "/// Every method declared by the schema, sorted by wire name.").unwrap();
    writeln!(s, "///").unwrap();
    writeln!(
        s,
        "/// Used by the parity gate in `tests/rpc_parity.rs` to prove the crate binds"
    )
    .unwrap();
    writeln!(s, "/// the complete protocol surface.").unwrap();
    writeln!(s, "pub static ALL_METHODS: &[MethodInfo] = &[").unwrap();
    for m in methods {
        let surface = match m.surface {
            "server" => "Server",
            "session" => "Session",
            _ => "ClientSession",
        };
        let stability = if m.stability == "experimental" {
            "Experimental"
        } else {
            "Stable"
        };
        writeln!(s, "    MethodInfo {{").unwrap();
        writeln!(s, "        wire_name: \"{}\",", m.wire).unwrap();
        writeln!(s, "        surface: Surface::{surface},").unwrap();
        writeln!(s, "        stability: Stability::{stability},").unwrap();
        writeln!(s, "        internal: {},", m.visibility == "internal").unwrap();
        writeln!(s, "        has_params: {},", m.has_params).unwrap();
        writeln!(s, "        has_result: {},", m.has_result).unwrap();
        writeln!(s, "    }},").unwrap();
    }
    writeln!(s, "];").unwrap();
    writeln!(s).unwrap();

    // ---- counts ----
    writeln!(s, "/// Total number of methods declared by the schema.").unwrap();
    writeln!(s, "pub const METHOD_COUNT: usize = {};", methods.len()).unwrap();
    writeln!(s).unwrap();
    writeln!(s, "/// Look up schema metadata for a wire name.").unwrap();
    writeln!(s, "pub fn find(wire_name: &str) -> Option<&'static MethodInfo> {{").unwrap();
    writeln!(s, "    ALL_METHODS.iter().find(|m| m.wire_name == wire_name)").unwrap();
    writeln!(s, "}}").unwrap();

    s
}

#[cfg(test)]
mod tests {
    use super::screaming_snake;

    #[test]
    fn converts_dotted_camel_case() {
        assert_eq!(screaming_snake("model.getCurrent"), "MODEL_GET_CURRENT");
        assert_eq!(screaming_snake("ping"), "PING");
        assert_eq!(screaming_snake("mcp.apps.callTool"), "MCP_APPS_CALL_TOOL");
        assert_eq!(screaming_snake("sessionFs.readFile"), "SESSION_FS_READ_FILE");
    }
}
