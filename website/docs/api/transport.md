---
id: transport
title: "API: transport"
sidebar_label: transport
---

# Module `transport`

Raw byte I/O and `Content-Length` message framing. Re-exported at the crate root as `copilot_sdk::{Transport, StdioTransport, MessageFramer}`. See [Transport and protocol](/docs/core-concepts/transport-and-protocol).

## `Transport`

Async transport for raw byte I/O.

```rust
pub trait Transport: Send + Sync {
    fn read<'a>(&'a mut self, buf: &'a mut [u8])
        -> Pin<Box<dyn Future<Output = Result<usize>> + Send + 'a>>;
    fn write<'a>(&'a mut self, data: &'a [u8])
        -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>>;
    fn close(&mut self)
        -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
    fn is_open(&self) -> bool;
}
```

## `StdioTransport`

Transport over the stdin/stdout of a child process.

```rust
pub struct StdioTransport { /* ... */ }

impl StdioTransport {
    pub fn new(stdin: tokio::process::ChildStdin, stdout: tokio::process::ChildStdout) -> Self;
    pub fn split(self) -> (tokio::process::ChildStdin, tokio::process::ChildStdout);
}
```

Behavior notes:

- Reads from the child's `stdout`, writes to the child's `stdin`.
- `close()` marks the transport closed (sets `is_open()` to `false`).
- On Windows, flush failures on a closing pipe are tolerated so shutdown can complete cleanly.

## `MessageFramer`

Handles `Content-Length` header framing for JSON-RPC messages over any [`Transport`](#transport).

```rust
pub struct MessageFramer<T: Transport> { /* ... */ }

impl<T: Transport> MessageFramer<T> {
    pub fn new(transport: T) -> Self;
    pub async fn read_message(&mut self) -> Result<String>;
    pub async fn write_message(&mut self, message: &str) -> Result<()>;
    pub fn transport(&self) -> &T;
    pub fn transport_mut(&mut self) -> &mut T;
    pub fn into_transport(self) -> T;
}
```

Framing format:

```text
Content-Length: <byte-length>\r\n
\r\n
<json-body>
```

- **Writing** prepends the `Content-Length` header to the serialized JSON body.
- **Reading** parses headers until a blank line, requires `Content-Length`, then reads exactly that many body bytes.

## TCP

There is no standalone TCP `Transport` implementation in this module. TCP support is provided in the [`jsonrpc`](/docs/api/jsonrpc) module via Tokio's `TcpStream` split into read/write halves, using the same `Content-Length` framing.

## Related

- [JSON-RPC](/docs/api/jsonrpc)
- [Process](/docs/api/process)
- [Transport and protocol concept](/docs/core-concepts/transport-and-protocol)
