---
id: requirements
title: Requirements
sidebar_label: Requirements
---

# Requirements

Before using the SDK you need a working, authenticated Copilot CLI and a recent Rust toolchain.

## Rust toolchain

- **Rust 1.85 or newer.** The repository pins the toolchain in [`rust-toolchain.toml`](https://github.com/dayour/copilot-rust-sdk/blob/main/rust-toolchain.toml).
- The crate is published with `edition = "2021"` and `rust-version = "1.85"`.

Check your version:

```bash
rustc --version
```

Install or update Rust with [rustup](https://rustup.rs):

```bash
rustup update stable
```

## GitHub Copilot CLI

The SDK does **not** bundle the Copilot CLI. You must install and authenticate it separately.

1. Install the GitHub Copilot CLI (see the official Copilot CLI documentation).
2. Authenticate it so that it has a valid GitHub session, or supply a token (see [Authentication](#authentication)).
3. Make the `copilot` executable discoverable.

### CLI discovery

When you build a [`Client`](/docs/api/client) without an explicit path, the SDK locates the CLI using [`find_copilot_cli`](/docs/api/process) in this order:

1. The `COPILOT_CLI_PATH` environment variable, if set and pointing at an existing file.
2. `copilot` on your `PATH`.
3. On Windows only: `copilot.cmd`, then `copilot.exe`.

You can always override discovery explicitly:

```rust
use copilot_sdk::Client;

let client = Client::builder()
    .cli_path(r"C:\tools\copilot\copilot.cmd")
    .build()?;
```

Or via environment variable:

```bash
# PowerShell
$env:COPILOT_CLI_PATH = "C:\tools\copilot\copilot.cmd"
```

```bash
# bash
export COPILOT_CLI_PATH=/usr/local/bin/copilot
```

:::tip Node-based CLIs
If the CLI is distributed as a Node.js script, the SDK can detect `.js`/`.mjs` entry points and locate `node` on your `PATH`. See [`is_node_script`](/docs/api/process) and [`find_node`](/docs/api/process).
:::

## Authentication

The SDK relies on the CLI's own authentication. You have a few options:

- **Use the logged-in user** - the default. The CLI uses whatever GitHub session it was authenticated with.
- **Supply a token** - pass a GitHub token to the builder:

  ```rust
  let client = Client::builder()
      .github_token(std::env::var("GITHUB_TOKEN")?)
      .build()?;
  ```

- **BYOK** - use your own provider keys for OpenAI-compatible endpoints. See [BYOK](/docs/guides/byok).

You can inspect the current authentication state at runtime:

```rust
let auth = client.get_auth_status().await?;
println!("authenticated: {}", auth.is_authenticated);
```

## Platform support

The SDK is pure safe Rust (`#![forbid(unsafe_code)]`) and runs anywhere Tokio and the Copilot CLI run, including Linux, macOS, and Windows. Windows pipe quirks are handled in the [stdio transport](/docs/core-concepts/transport-and-protocol).

## Next steps

- [Installation](/docs/getting-started/installation)
- [Quick Start](/docs/getting-started/quick-start)
