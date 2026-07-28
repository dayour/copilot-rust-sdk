// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Wire-protocol parity gate.
//!
//! These tests are the guard rail against two classes of defect:
//!
//! 1. **Invented method names.** Any RPC method name passed to an invocation
//!    site must exist in the CLI's published schema. A typo, a snake_case slip,
//!    or a wrong namespace fails the build instead of silently 404-ing at
//!    runtime against a live CLI.
//! 2. **Silent parity regressions.** The set of schema methods the crate does
//!    *not* yet bind is pinned in `tests/rpc_parity_allowlist.txt`. Implementing
//!    a method and deleting its line is always allowed; dropping an existing
//!    binding is not.

use copilot_sdk::generated::methods::{self, Surface};
use regex::Regex;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// Every RPC method name literal that the crate actually sends on the wire.
///
/// Anchored to the two invocation helpers so that event-type strings, lifecycle
/// constants and doc examples are never mistaken for outbound RPC calls.
fn invoked_methods() -> BTreeSet<String> {
    scan(
        &Regex::new(r#"(?:\.invoke|invoke_fn\))\(\s*"([A-Za-z][A-Za-z0-9_.]*)""#)
            .expect("valid regex"),
    )
}

/// Every RPC method name the crate *serves* as a reverse-RPC handler.
///
/// Reverse RPC (the `clientSession` surface) is dispatched by matching on the
/// inbound method name, so these appear as match-arm patterns rather than
/// invocation arguments.
fn served_methods() -> BTreeSet<String> {
    scan(&Regex::new(r#""([A-Za-z][A-Za-z0-9_.]*)"\s*=>"#).expect("valid regex"))
}

/// A method is implemented if the crate either calls it or serves it.
fn implemented_methods() -> BTreeSet<String> {
    let mut all = invoked_methods();
    all.extend(served_methods());
    all
}

fn scan(re: &Regex) -> BTreeSet<String> {
    let mut files = Vec::new();
    rust_sources(&repo_root().join("src"), &mut files);
    assert!(!files.is_empty(), "found no Rust sources under src/");

    let mut found = BTreeSet::new();
    for file in files {
        // Generated code is a registry of names, not a set of call sites.
        if file.components().any(|c| c.as_os_str() == "generated") {
            continue;
        }
        let text = std::fs::read_to_string(&file).expect("read source file");
        for caps in re.captures_iter(&text) {
            found.insert(caps[1].to_string());
        }
    }
    found
}

fn allowlist_path() -> PathBuf {
    repo_root().join("tests/rpc_parity_allowlist.txt")
}

/// Methods the SDK legitimately sends that `api.schema.json` does not declare.
///
/// The vendored schema documents the server- and session-scoped RPC surfaces,
/// but omits the session *lifecycle* methods that create and manage sessions in
/// the first place, plus two client-level status calls. Every entry here is
/// verified against the reference implementation under `copilot-sdk/nodejs/src`
/// and must cite where it is used there.
///
/// This list is deliberately closed: anything not in the schema and not listed
/// here fails [`every_invoked_method_exists_in_schema`].
const OFF_SCHEMA_METHODS: &[(&str, &str)] = &[
    ("session.create", "client.ts"),
    ("session.resume", "client.ts"),
    ("session.list", "client.ts"),
    ("session.delete", "client.ts"),
    ("session.getLastId", "client.ts"),
    ("session.getMetadata", "client.ts"),
    ("session.getForeground", "client.ts"),
    ("session.setForeground", "client.ts"),
    ("session.getMessages", "session.ts"),
    ("session.destroy", "session.ts"),
    ("status.get", "client.ts"),
    ("auth.getStatus", "client.ts"),
];

fn is_off_schema(name: &str) -> bool {
    OFF_SCHEMA_METHODS.iter().any(|(m, _)| *m == name)
}

fn unbound_allowlist() -> BTreeSet<String> {
    let text = std::fs::read_to_string(allowlist_path()).expect("read parity allowlist");
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect()
}

/// The schema must be non-empty and internally consistent.
#[test]
fn schema_registry_is_well_formed() {
    assert_eq!(
        methods::ALL_METHODS.len(),
        methods::METHOD_COUNT,
        "ALL_METHODS length disagrees with METHOD_COUNT"
    );

    let unique: BTreeSet<&str> = methods::ALL_METHODS.iter().map(|m| m.wire_name).collect();
    assert_eq!(
        unique.len(),
        methods::ALL_METHODS.len(),
        "duplicate wire names in generated registry"
    );

    for m in methods::ALL_METHODS {
        assert!(
            !m.wire_name.is_empty(),
            "generated registry contains an empty wire name"
        );
        assert_eq!(
            methods::find(m.wire_name).map(|f| f.wire_name),
            Some(m.wire_name),
            "find() failed to resolve {}",
            m.wire_name
        );
    }
}

/// Session-scoped methods must carry the `session.` prefix, and only those.
#[test]
fn session_surface_names_are_prefixed() {
    for m in methods::ALL_METHODS {
        match m.surface {
            Surface::Session => assert!(
                m.wire_name.starts_with("session."),
                "session-surface method `{}` lacks the `session.` prefix",
                m.wire_name
            ),
            Surface::Server | Surface::ClientSession => assert!(
                !m.wire_name.starts_with("session."),
                "non-session method `{}` unexpectedly starts with `session.`",
                m.wire_name
            ),
        }
    }
}

/// **The regression gate.** Every method name we send must exist in the schema.
///
/// This is the test that catches wrong casing (`switch_to` vs `switchTo`) and
/// wrong namespaces (`session.workspace.*` vs `session.workspaces.*`).
#[test]
fn every_invoked_method_exists_in_schema() {
    let invoked = invoked_methods();
    assert!(
        !invoked.is_empty(),
        "parity scanner matched no invocation sites - the regex is broken"
    );

    let unknown: Vec<&String> = invoked
        .iter()
        .filter(|name| methods::find(name).is_none() && !is_off_schema(name))
        .collect();

    assert!(
        unknown.is_empty(),
        "these RPC method names are sent on the wire but exist neither in \
         schemas/api.schema.json nor in the verified OFF_SCHEMA_METHODS list:\n  {}\n\n\
         Either the name is wrong (check casing and namespace against the \
         schema), the vendored schema is stale, or the method needs an \
         OFF_SCHEMA_METHODS entry citing its nodejs call site.",
        unknown
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// Every off-schema exception must still be genuinely absent from the schema.
///
/// Prevents the list rotting into a place where real methods hide from the gate
/// once a schema bump starts declaring them.
#[test]
fn off_schema_exceptions_are_still_needed() {
    let stale: Vec<&str> = OFF_SCHEMA_METHODS
        .iter()
        .filter(|(m, _)| methods::find(m).is_some())
        .map(|(m, _)| *m)
        .collect();

    assert!(
        stale.is_empty(),
        "these methods are now declared by the schema and must be removed from \
         OFF_SCHEMA_METHODS so they are covered by the parity gate:\n  {}",
        stale.join("\n  ")
    );

    for (method, source) in OFF_SCHEMA_METHODS {
        assert!(
            !method.is_empty() && !source.is_empty(),
            "OFF_SCHEMA_METHODS entries must cite a nodejs source file"
        );
    }
}

/// Bindings may only be added, never silently removed.
#[test]
fn unbound_methods_match_allowlist() {
    let implemented = implemented_methods();
    let allow = unbound_allowlist();

    let actually_unbound: BTreeSet<String> = methods::ALL_METHODS
        .iter()
        .map(|m| m.wire_name.to_string())
        .filter(|name| !implemented.contains(name))
        .collect();

    // Regression: something that used to be bound no longer is.
    let regressed: Vec<&String> = actually_unbound.difference(&allow).collect();
    assert!(
        regressed.is_empty(),
        "these schema methods lost their Rust binding:\n  {}\n\nRe-add the \
         binding, or append them to tests/rpc_parity_allowlist.txt if the \
         removal is deliberate.",
        regressed
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );

    // Hygiene: the allowlist must not list methods that are in fact bound.
    let stale: Vec<&String> = allow.difference(&actually_unbound).collect();
    assert!(
        stale.is_empty(),
        "tests/rpc_parity_allowlist.txt lists methods that ARE now bound - \
         delete these lines to lock in the progress:\n  {}",
        stale
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// Emits the human-readable parity matrix; never fails.
#[test]
fn report_parity_coverage() {
    let implemented = implemented_methods();
    let mut rows: Vec<(&str, usize, usize)> = Vec::new();

    for (label, surface) in [
        ("server", Surface::Server),
        ("session", Surface::Session),
        ("clientSession", Surface::ClientSession),
    ] {
        let total = methods::ALL_METHODS
            .iter()
            .filter(|m| m.surface == surface)
            .count();
        let bound = methods::ALL_METHODS
            .iter()
            .filter(|m| m.surface == surface && implemented.contains(m.wire_name))
            .count();
        rows.push((label, bound, total));
    }

    let bound_total: usize = rows.iter().map(|r| r.1).sum();
    let all_total: usize = rows.iter().map(|r| r.2).sum();

    println!("\n=== RPC parity ===");
    for (label, bound, total) in &rows {
        println!(
            "  {label:<14} {bound:>3}/{total:<3}  {:>5.1}%",
            (*bound as f64 / *total as f64) * 100.0
        );
    }
    println!(
        "  {:<14} {bound_total:>3}/{all_total:<3}  {:>5.1}%",
        "TOTAL",
        (bound_total as f64 / all_total as f64) * 100.0
    );
}
