---
id: error-handling
title: Error Handling
sidebar_label: Error Handling
---

# Error Handling

All fallible SDK operations return [`copilot_sdk::Result<T>`](/docs/api/error#result), which is an alias for `std::result::Result<T, CopilotError>`. This page explains the error variants and how to handle them robustly.

## The `Result` alias

```rust
pub type Result<T> = std::result::Result<T, CopilotError>;
```

Use it as your function return type to get clean `?` propagation:

```rust
async fn run() -> copilot_sdk::Result<()> {
    let client = copilot_sdk::Client::builder().build()?;
    client.start().await?;
    // ...
    Ok(())
}
```

## The `CopilotError` enum

[`CopilotError`](/docs/api/error#copiloterror) enumerates everything that can go wrong, from transport failures to protocol mismatches.

| Variant | Meaning | Typical cause |
|---------|---------|---------------|
| `Transport(io::Error)` | Raw I/O failure | Broken pipe, socket reset |
| `ConnectionClosed` | Peer disconnected unexpectedly | CLI exited mid-request |
| `NotConnected` | Client is not connected | Called an RPC before `start` |
| `JsonRpc { code, message, data }` | Server returned a JSON-RPC error | Invalid params, server-side failure |
| `ProtocolMismatch { min, max, actual }` | No common protocol version | CLI too old/new |
| `Protocol(String)` | Framing/parse/protocol violation | Malformed frame |
| `Json(serde_json::Error)` | (De)serialization error | Unexpected payload shape |
| `Timeout(Duration)` | Request timed out | Slow or stuck operation |
| `SessionNotFound(String)` | Session id is unknown | Resuming a deleted session |
| `SessionDestroyed` | Session already destroyed | Using a session after `destroy` |
| `InvalidConfig(String)` | Bad configuration | Conflicting options |
| `ProcessStart(io::Error)` | CLI failed to start | CLI not found, not executable |
| `ProcessExit(Option<i32>)` | CLI exited unexpectedly | Crash, kill signal |
| `PortDetectionFailed` | TCP port discovery failed | TCP mode setup issue |
| `Shutdown` | Client is shutting down | Call during teardown |
| `ToolNotFound(String)` | Tool not registered | Invoking an unknown tool |
| `ToolError(String)` | Tool execution failed | Handler returned an error |
| `PermissionDenied(String)` | Permission was denied | Denied tool call |
| `ChannelError` | Internal channel send failure | Receiver dropped |

`CopilotError` derives `From<std::io::Error>` and `From<serde_json::Error>`, so `?` converts those automatically.

## Matching on errors

```rust
use copilot_sdk::CopilotError;

match session.send_and_collect("Hi", None).await {
    Ok(text) => println!("{text}"),
    Err(CopilotError::Timeout(dur)) => {
        eprintln!("timed out after {dur:?}");
    }
    Err(CopilotError::JsonRpc { code, message, .. }) => {
        eprintln!("server error {code}: {message}");
    }
    Err(CopilotError::NotConnected) => {
        eprintln!("did you call client.start().await?");
    }
    Err(e) => eprintln!("unexpected: {e}"),
}
```

## Startup errors

The most common first-run problems map to specific variants:

- **`ProcessStart`** - the CLI could not be launched. Check that `copilot` is on `PATH` or set `COPILOT_CLI_PATH`. See [Requirements](/docs/getting-started/requirements#cli-discovery).
- **`ProtocolMismatch`** - update the CLI (or the SDK) so their supported protocol ranges overlap.
- **`NotConnected`** - you called an RPC before `start` completed.

## Timeouts

Several operations accept a timeout, and the JSON-RPC layer supports `invoke_with_timeout`. On expiry you get `CopilotError::Timeout(Duration)`. Wrap long operations and decide whether to retry, abort, or surface the error.

```rust
let result = session
    .send_and_wait("Long task", Some(std::time::Duration::from_secs(300)))
    .await;
```

## Shutdown errors

[`client.stop()`](/docs/core-concepts/client-lifecycle#stopping) does **not** return a `Result`. Instead it returns `Vec<StopError>` so a noisy shutdown never masks your program's real result:

```rust
for err in client.stop().await {
    eprintln!("shutdown: {}{}", err.message,
        err.source.map(|s| format!(" ({s})")).unwrap_or_default());
}
```

## Logging, not panicking

The crate forbids `unsafe` and avoids panics in normal operation - errors are values. When integrating, prefer logging and graceful degradation over `unwrap`. Avoid logging prompts, tokens, or secrets (see the [security guidance](/docs/contributing#security)).

## Next

- [Error API reference](/docs/api/error)
- [Client lifecycle](/docs/core-concepts/client-lifecycle)
