---
id: contributing
title: Contributing
sidebar_label: Contributing
---

# Contributing

Thanks for your interest in improving the SDK. This page summarizes the development workflow, conventions, and quality gates.

## Project layout

```text
src/          library implementation (client, session, transport, jsonrpc, types, ...)
examples/     runnable samples (cargo run --example <name>)
tests/        integration tests (some feature-gated)
website/      this documentation site (Docusaurus)
.github/      CI workflows
```

## Setup

Enable the pre-commit hooks so formatting and linting issues are caught before push:

```bash
git config core.hooksPath .githooks
```

The toolchain is pinned in `rust-toolchain.toml` (Rust 1.85). Install it with [rustup](https://rustup.rs).

## Build, test, and lint

```bash
cargo build                                              # compile the crate
cargo test                                               # unit + integration tests
cargo fmt --all                                          # format (CI checks with --check)
cargo clippy --all-targets --all-features -- -D warnings # lint (warnings are errors)
cargo doc --no-deps                                      # build API docs locally
```

CI runs format, clippy, test, doc, and package steps. Run them locally before opening a PR.

### Feature-gated tests

Some suites require extra setup and are off by default:

```bash
# End-to-end tests against a real, authenticated Copilot CLI.
cargo test --features e2e -- --test-threads=1

# Snapshot conformance against upstream YAML snapshots.
cargo test --features snapshots --test snapshot_conformance
```

For snapshot tests, set `COPILOT_SDK_RUST_SNAPSHOT_DIR` or `UPSTREAM_SNAPSHOTS` to point at the upstream `copilot-sdk/test/snapshots` directory if auto-detection fails.

## Coding style

- Format with `rustfmt`; lint with `clippy` (treat warnings as errors).
- No `unsafe`: the crate sets `#![forbid(unsafe_code)]`.
- Keep the **attribution header** at the top of every Rust file. This is enforced by `tests/attribution_headers.rs`:

  ```rust
  // Copyright (c) 2026 Elias Bachaalany
  // SPDX-License-Identifier: MIT
  ```

- Comment only what needs clarification; prefer clear names over narration.

## Commits and pull requests

- Use [Conventional Commits](https://www.conventionalcommits.org): `feat:`, `fix:`, `docs:`, `chore:`, and keep commits focused.
- PRs should state **what** and **why**, **how to test** (commands), and include docs/example updates for any API or behavior change.

## Documentation

This site lives in `website/` and is built with [Docusaurus](https://docusaurus.io).

```bash
cd website
npm install
npm start          # local dev server with hot reload
npm run build      # production build into website/build
```

When you change the public API, update the relevant pages under `docs/api/` and any affected guide. Keep code snippets compiling against the current API.

## Security

The SDK drives the Copilot CLI runtime, so handle sensitive data carefully:

- **Never log secrets, prompts, or tokens.** Avoid `println!`-ing message content in committed code paths.
- **Do not commit credentials.** Read provider keys and GitHub tokens from the environment or a secrets manager (see [BYOK](/docs/guides/byok)).
- **Telemetry content capture** is opt-in. Leave `capture_content` disabled where prompts may be sensitive (see [Telemetry](/docs/guides/telemetry)).
- **Be deliberate with tool permissions.** Avoid `allow_all_tools` in untrusted contexts; prefer explicit allow/deny lists and a [permission handler](/docs/guides/permissions), especially when the agent can reach a [shell](/docs/guides/shell).

## Getting help

Open an issue or discussion on [GitHub](https://github.com/dayour/copilot-rust-sdk). For upstream context, see the [official SDKs](https://github.com/github/copilot-sdk).
