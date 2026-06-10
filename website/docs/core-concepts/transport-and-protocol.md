---
id: transport-and-protocol
title: Transport and Protocol
sidebar_label: Transport and Protocol
---

# Transport and Protocol

This page documents the wire protocol: how bytes move between the SDK and the CLI, how messages are framed, and how protocol versions are negotiated. Most applications never touch this layer directly, but understanding it helps with debugging and advanced setups.

## Transports

The SDK supports two transports:

- **stdio** (default) - the SDK spawns the CLI as a child process and talks over its `stdin`/`stdout`. Configured with `use_stdio(true)`.
- **TCP** - the SDK connects to a CLI server over a socket, which may be spawned by the SDK or run externally. Configured with `cli_url(...)` and/or `port(...)`.

The [`Transport`](/docs/api/transport#transport) trait abstracts raw byte I/O:

```rust
pub trait Transport: Send + Sync {
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> /* Future<Result<usize>> */;
    fn write<'a>(&'a mut self, data: &'a [u8]) -> /* Future<Result<()>> */;
    fn close(&mut self) -> /* Future<Result<()>> */;
    fn is_open(&self) -> bool;
}
```

[`StdioTransport`](/docs/api/transport#stdiotransport) implements this over a child process's pipes. The TCP path uses Tokio's `TcpStream` split into read/write halves inside the [`jsonrpc`](/docs/api/jsonrpc) layer.

:::note Windows pipes
On Windows, flushing a closed child pipe can fail in benign ways; the stdio transport tolerates these flush errors during shutdown so a clean exit is still reported.
:::

## Message framing

Messages use **`Content-Length` framing**, the same scheme as the Language Server Protocol. Each message is a JSON body preceded by a header block:

```text
Content-Length: 123\r\n
\r\n
{"jsonrpc":"2.0","method":"...","params":{...},"id":1}
```

[`MessageFramer`](/docs/api/transport#messageframer) handles both directions:

- **Writing** prepends `Content-Length: <byte-length>\r\n\r\n` to the serialized JSON.
- **Reading** parses headers until a blank line, requires a `Content-Length`, then reads exactly that many body bytes.

```rust
// Conceptually:
let mut framer = MessageFramer::new(transport);
framer.write_message(&json_string).await?;
let incoming: String = framer.read_message().await?;
```

## JSON-RPC 2.0

On top of framing sits a full [JSON-RPC 2.0](https://www.jsonrpc.org/specification) client, [`JsonRpcClient`](/docs/api/jsonrpc#jsonrpcclient). It supports the three JSON-RPC message shapes:

| Shape | Has `id`? | Direction in this SDK |
|-------|-----------|-----------------------|
| Request | yes | Both ways (you call the CLI; the CLI calls you) |
| Response | yes | Reply to a request |
| Notification | no | Both ways (events, fire-and-forget) |

Key operations:

```rust
// Request/response with a typed result value.
let result = rpc.invoke("status.get", None).await?;

// Request with an explicit timeout.
let result = rpc.invoke_with_timeout("models.list", None, Duration::from_secs(30)).await?;

// Fire-and-forget notification.
rpc.notify("session.cancel", Some(params)).await?;
```

### Request ids

[`JsonRpcId`](/docs/api/jsonrpc#jsonrpcid) can be a number or a string; the SDK auto-generates ids for outbound requests and matches inbound responses against the pending-request table.

### Standard error codes

[`JsonRpcError`](/docs/api/jsonrpc#jsonrpcerror) exposes the standard constants:

| Constant | Code |
|----------|------|
| `PARSE_ERROR` | -32700 |
| `INVALID_REQUEST` | -32600 |
| `METHOD_NOT_FOUND` | -32601 |
| `INVALID_PARAMS` | -32602 |
| `INTERNAL_ERROR` | -32603 |

Server errors surface to your code as [`CopilotError::JsonRpc { code, message, data }`](/docs/api/error).

### Bidirectional dispatch

The client's background read loop dispatches each inbound frame:

- a **response** completes the matching pending request,
- a **notification** invokes the notification handler (this is how session events flow),
- a **request** invokes the request handler and the SDK sends back a response (this is how tool calls, permission prompts, hooks, and user-input requests flow).

## Protocol versioning

- **SDK protocol version:** 3 (constant [`SDK_PROTOCOL_VERSION`](/docs/api/types)).
- **Minimum supported:** 2.

During [`start`](/docs/core-concepts/client-lifecycle#starting), the SDK and CLI negotiate a common version. If the CLI's range does not overlap the SDK's, you get [`CopilotError::ProtocolMismatch { min, max, actual }`](/docs/api/error). After a successful start, read the negotiated value:

```rust
let v = client.negotiated_protocol_version().await; // Option<u32>
```

## Debugging the wire

To inspect raw traffic, raise the client log level and consider the `debug_pipe` example in the repository, which demonstrates observing framed messages.

```rust
use copilot_sdk::LogLevel;
let client = Client::builder().log_level(LogLevel::Debug).build()?;
```

## Next

- [JSON-RPC API reference](/docs/api/jsonrpc)
- [Transport API reference](/docs/api/transport)
- [Error handling](/docs/core-concepts/error-handling)
