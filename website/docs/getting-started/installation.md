---
id: installation
title: Installation
sidebar_label: Installation
---

# Installation

Add the crate and an async runtime to your `Cargo.toml`.

## From crates.io

```toml
[dependencies]
copilot-sdk = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

:::note Crate name vs. import path
The crate is named **`copilot-sdk`** on crates.io, but Rust replaces hyphens with underscores, so you import it as **`copilot_sdk`**:

```rust
use copilot_sdk::{Client, SessionConfig};
```
:::

## From a local checkout

If you are developing against a local clone of the repository:

```toml
[dependencies]
copilot-sdk = { path = "../copilot-sdk-rust" }
```

## From git

```toml
[dependencies]
copilot-sdk = { git = "https://github.com/dayour/copilot-rust-sdk", branch = "main" }
```

## Cargo features

The crate exposes a small set of [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html):

| Feature | Default | Purpose |
|---------|---------|---------|
| `schemars` | off | Derive JSON schemas for tool parameter types via [`schemars`](https://docs.rs/schemars). Enables [`Tool::typed_schema`](/docs/api/tools). |
| `e2e` | off | Enable end-to-end tests that require a real, authenticated Copilot CLI. |
| `snapshots` | off | Enable snapshot-based conformance tests against upstream YAML snapshots. |

Enable a feature like this:

```toml
[dependencies]
copilot-sdk = { version = "0.1", features = ["schemars"] }
```

## Required Tokio features

The SDK is async and built on Tokio. At a minimum you need a multi-threaded runtime and the `macros` feature for `#[tokio::main]`:

```toml
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

The crate itself depends on the following Tokio features internally: `rt-multi-thread`, `net`, `process`, `sync`, `io-util`, `macros`, and `time`. You only need to enable the features your own code uses.

## Verifying the install

Create a tiny program and run it:

```rust title="src/main.rs"
#[tokio::main]
async fn main() -> copilot_sdk::Result<()> {
    let client = copilot_sdk::Client::builder().build()?;
    client.start().await?;
    let status = client.get_status().await?;
    println!("Copilot CLI version: {}", status.version);
    println!("Protocol version: {}", status.protocol_version);
    client.stop().await;
    Ok(())
}
```

```bash
cargo run
```

If you see a version and protocol number, the SDK located the CLI, started it, and negotiated the protocol successfully. If not, see [Requirements](/docs/getting-started/requirements#cli-discovery) and [Error handling](/docs/core-concepts/error-handling).

## Next steps

- [Quick Start](/docs/getting-started/quick-start)
- [Your First Session](/docs/getting-started/your-first-session)
