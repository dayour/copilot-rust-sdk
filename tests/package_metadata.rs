// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

use std::fs;
use std::path::PathBuf;

#[test]
fn package_metadata_points_to_current_repository() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let text = fs::read_to_string(manifest).expect("read Cargo.toml");

    assert!(
        text.contains(r#"repository = "https://github.com/dayour/copilot-rust-sdk""#),
        "Cargo.toml repository should point at dayour/copilot-rust-sdk"
    );
    assert!(
        text.contains(r#"homepage = "https://github.com/dayour/copilot-rust-sdk""#),
        "Cargo.toml homepage should point at dayour/copilot-rust-sdk"
    );
    assert!(text.contains("[package.metadata.docs.rs]"));
    assert!(
        text.contains("all-features = true"),
        "docs.rs metadata should build all features"
    );
}

#[test]
fn ci_docs_build_matches_docs_rs_features() {
    let workflow = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/ci.yml");
    let text = fs::read_to_string(workflow).expect("read ci.yml");

    assert!(
        text.contains("cargo doc --no-deps --all-features"),
        "CI docs step should build docs with all features"
    );
}

#[test]
fn msrv_and_ci_match_pinned_toolchain() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let toolchain = fs::read_to_string(root.join("rust-toolchain.toml")).expect("read toolchain");
    let manifest = fs::read_to_string(root.join("Cargo.toml")).expect("read manifest");
    let policy = fs::read_to_string(root.join("docs/msrv-policy.md")).expect("read MSRV policy");

    assert!(toolchain.contains(r#"channel = "1.99.0""#));
    assert!(manifest.contains(r#"rust-version = "1.99.0""#));
    assert!(manifest.contains(r#"edition = "2021""#));
    assert!(policy.contains("The minimum supported Rust version (MSRV) is Rust 1.99.0."));

    let workflow = fs::read_to_string(root.join(".github/workflows/ci.yml")).expect("read CI");
    let workflow: serde_yaml::Value = serde_yaml::from_str(&workflow).expect("parse CI");
    for job in ["test", "parity"] {
        let steps = workflow["jobs"][job]["steps"]
            .as_sequence()
            .expect("job steps");
        let install = steps
            .iter()
            .find(|step| step["name"].as_str() == Some("Install Rust toolchain"))
            .expect("toolchain installation step");
        assert_eq!(
            install["with"]["toolchain"].as_str(),
            Some("1.99.0"),
            "{job} must use the pinned toolchain"
        );
    }
}
