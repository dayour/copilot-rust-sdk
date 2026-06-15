# Copilot Cloud Agent Onboarding (copilot-rust-sdk)

This repository is a Rust SDK for the GitHub Copilot CLI runtime.

## Start Here (fast orientation)

1. Read `AGENTS.md` for repository-specific workflow rules.
2. Read `README.md` for feature overview and examples.
3. Check `.github/workflows/ci.yml` to mirror CI checks locally.

## Codebase layout

- `src/`: library implementation (`client`, `session`, `transport`, `jsonrpc`, `types`)
- `examples/`: runnable examples (e.g. `cargo run --example basic_chat`)
- `tests/`: integration tests
  - `e2e_tests.rs` and `e2e_parity_tests.rs` are feature-gated
  - `snapshot_conformance.rs` is feature-gated and optional
- `website/`: Docusaurus docs site

## Rust/tooling expectations

- Rust toolchain is pinned to `1.85.0` (`rust-toolchain.toml`).
- Crate forbids unsafe code (`#![forbid(unsafe_code)]`).
- Keep attribution headers in Rust files (enforced by `tests/attribution_headers.rs`).

## Commands to run

Mirror CI and run these from repository root:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo doc --no-deps
cargo package
```

For feature-gated suites when relevant:

```bash
cargo test --features e2e -- --test-threads=1
cargo test --features snapshots --test snapshot_conformance
```

If snapshot tests cannot auto-detect fixtures, set `COPILOT_SDK_RUST_SNAPSHOT_DIR` or `UPSTREAM_SNAPSHOTS`.

## Change strategy

- Prefer minimal, surgical edits scoped to the issue.
- Do not alter unrelated tests or behavior.
- When touching API behavior, update docs/examples in the same PR.
- Avoid logging secrets/prompts/tokens.

## Errors encountered during onboarding and workarounds

1. **GitHub Actions run showed `action_required` with no jobs (`total_jobs: 0`)** for run `27523989432`.
   - **Workaround:** inspect the run first (`list_workflow_runs` + `get_workflow_run`); if jobs are absent, treat it as workflow-gating/approval state rather than a code test failure.

2. **First Rust command triggered toolchain bootstrap and component downloads** (`rustfmt`, `clippy`, etc.).
   - **Workaround:** allow extra time on first run; subsequent commands are much faster.

3. **Running multiple Cargo commands concurrently caused lock waits** (`Blocking waiting for file lock on package cache/build directory`).
   - **Workaround:** run validation commands sequentially for predictable execution.

4. **`cargo doc --no-deps` emitted broken intra-doc link warnings** (unresolved links `clear_models_cache` and `off`) but still exited successfully.
   - **Workaround:** treat as non-blocking for onboarding unless your change touches those docs; if you touch them, fix links in the same PR.
