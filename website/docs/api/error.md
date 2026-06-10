---
id: error
title: "API: error"
sidebar_label: error
---

# Module `error`

The crate's error type and `Result` alias. Re-exported at the crate root as `copilot_sdk::{CopilotError, Result}`. See [Error handling](/docs/core-concepts/error-handling).

## `Result`

```rust
pub type Result<T> = std::result::Result<T, CopilotError>;
```

## `CopilotError`

```rust
pub enum CopilotError {
    Transport(std::io::Error),                          // #[from]
    ConnectionClosed,
    NotConnected,
    JsonRpc { code: i32, message: String, data: Option<serde_json::Value> },
    ProtocolMismatch { min: u32, max: u32, actual: u32 },
    Protocol(String),
    Json(serde_json::Error),                            // #[from]
    Timeout(Duration),
    SessionNotFound(String),
    SessionDestroyed,
    InvalidConfig(String),
    ProcessStart(std::io::Error),
    ProcessExit(Option<i32>),
    PortDetectionFailed,
    Shutdown,
    ToolNotFound(String),
    ToolError(String),
    PermissionDenied(String),
    ChannelError,
}
```

### Variant reference

| Variant | Meaning |
|---------|---------|
| `Transport(io::Error)` | Raw transport/IO failure. |
| `ConnectionClosed` | Peer disconnected unexpectedly. |
| `NotConnected` | Client is not connected. |
| `JsonRpc { code, message, data }` | Server returned a JSON-RPC error. |
| `ProtocolMismatch { min, max, actual }` | No common protocol version. |
| `Protocol(String)` | Framing/parse/protocol violation. |
| `Json(serde_json::Error)` | JSON (de)serialization error. |
| `Timeout(Duration)` | Request timed out. |
| `SessionNotFound(String)` | Unknown session id. |
| `SessionDestroyed` | Session already destroyed. |
| `InvalidConfig(String)` | Invalid configuration. |
| `ProcessStart(io::Error)` | CLI process failed to start. |
| `ProcessExit(Option<i32>)` | CLI exited unexpectedly. |
| `PortDetectionFailed` | TCP port discovery failed. |
| `Shutdown` | Client is shutting down. |
| `ToolNotFound(String)` | Tool not registered. |
| `ToolError(String)` | Tool execution failed. |
| `PermissionDenied(String)` | Permission denied. |
| `ChannelError` | Internal channel send failure. |

`CopilotError` derives `From<std::io::Error>` and `From<serde_json::Error>`, so `?` converts those automatically.

:::note `StopError`
`StopError` (returned by [`Client::stop`](/docs/api/client#constructors-and-lifecycle)) is defined in [`types`](/docs/api/types#responses-and-status), not here. It collects non-fatal shutdown problems without failing the stop call.
:::

## Related

- [Error handling concept](/docs/core-concepts/error-handling)
- [Types](/docs/api/types)
