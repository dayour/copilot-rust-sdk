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
    assert!(
        text.contains("[package.metadata.docs.rs]\nall-features = true"),
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
