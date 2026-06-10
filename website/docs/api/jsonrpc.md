---
id: jsonrpc
title: "API: jsonrpc"
sidebar_label: jsonrpc
---

# Module `jsonrpc`

A JSON-RPC 2.0 client with bidirectional communication. Re-exported at the crate root as `copilot_sdk::{JsonRpcClient, JsonRpcRequest, JsonRpcResponse, JsonRpcError, JsonRpcId, NotificationHandler, RequestHandler}`. See [Transport and protocol](/docs/core-concepts/transport-and-protocol).

## `JsonRpcId`

```rust
pub enum JsonRpcId {
    Num(i64),
    Str(String),
}
// Conversions: From<i64>, From<String>, From<&str>
```

## `JsonRpcRequest`

```rust
pub struct JsonRpcRequest {
    pub jsonrpc: String,         // "2.0"
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub id: Option<JsonRpcId>,   // None for notifications
}

impl JsonRpcRequest {
    pub fn new(method: impl Into<String>, params: Option<Value>, id: Option<JsonRpcId>) -> Self;
    pub fn notification(method: impl Into<String>, params: Option<Value>) -> Self;
    pub fn is_notification(&self) -> bool;
}
```

## `JsonRpcResponse`

```rust
pub struct JsonRpcResponse {
    pub jsonrpc: String,         // "2.0"
    pub id: Option<JsonRpcId>,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn success(id: JsonRpcId, result: Value) -> Self;
    pub fn error(id: JsonRpcId, error: JsonRpcError) -> Self;
    pub fn is_error(&self) -> bool;
}
```

## `JsonRpcError`

```rust
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl JsonRpcError {
    pub fn new(code: i32, message: impl Into<String>) -> Self;
    pub fn with_data(code: i32, message: impl Into<String>, data: Value) -> Self;

    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}
```

## Handler aliases

```rust
pub type NotificationHandler = Arc<dyn Fn(&str, &Value) + Send + Sync>;
pub type RequestHandler = Arc<dyn Fn(&str, &Value) -> RequestHandlerFuture + Send + Sync>;
pub type RequestHandlerFuture =
    Pin<Box<dyn Future<Output = std::result::Result<Value, JsonRpcError>> + Send>>;
```

## `JsonRpcClient`

The generic client over any [`Transport`](/docs/api/transport#transport).

```rust
pub struct JsonRpcClient<T: Transport> { /* ... */ }

impl<T: Transport> JsonRpcClient<T> {
    pub fn new(transport: T) -> Self;
    pub async fn start(&self) -> Result<()>;
    pub async fn stop(&self);
    pub fn is_running(&self) -> bool;

    pub async fn set_notification_handler<F>(&self, handler: F)
    where F: Fn(&str, &Value) + Send + Sync + 'static;
    pub async fn set_request_handler<F>(&self, handler: F)
    where F: Fn(&str, &Value) -> RequestHandlerFuture + Send + Sync + 'static;

    pub async fn invoke(&self, method: &str, params: Option<Value>) -> Result<Value>;
    pub async fn invoke_with_timeout(&self, method: &str, params: Option<Value>, timeout: Duration) -> Result<Value>;
    pub async fn notify(&self, method: &str, params: Option<Value>) -> Result<()>;
    pub async fn send_response(&self, id: JsonRpcId, result: Value) -> Result<()>;
    pub async fn send_error_response(&self, id: JsonRpcId, error: JsonRpcError) -> Result<()>;
}
```

### Dispatch flow

A background read loop decodes each framed message and routes it:

- **response** - completes the matching pending request,
- **notification** - invokes the notification handler,
- **request** - invokes the request handler, then sends a response.

## Transport-specific clients

The module also provides clients that manage separate read/write paths:

```rust
pub struct StdioJsonRpcClient { /* over StdioTransport */ }
impl StdioJsonRpcClient {
    pub fn new(transport: StdioTransport) -> Self;
    pub async fn start(&self) -> Result<()>;
    pub async fn stop(&self);
    pub fn is_running(&self) -> bool;
    pub async fn invoke(&self, method: &str, params: Option<Value>) -> Result<Value>;
    pub async fn invoke_with_timeout(&self, method: &str, params: Option<Value>, timeout: Duration) -> Result<Value>;
    pub async fn notify(&self, method: &str, params: Option<Value>) -> Result<()>;
    // set_notification_handler / set_request_handler as above
}

pub struct TcpJsonRpcClient { /* over a TcpStream */ }
impl TcpJsonRpcClient {
    pub async fn connect(addr: impl AsRef<str>) -> Result<Self>;
    pub fn new(stream: tokio::net::TcpStream) -> Self;
    pub async fn start(&self) -> Result<()>;
    pub async fn stop(&self);
    pub fn is_running(&self) -> bool;
    pub async fn invoke(&self, method: &str, params: Option<Value>) -> Result<Value>;
    pub async fn invoke_with_timeout(&self, method: &str, params: Option<Value>, timeout: Duration) -> Result<Value>;
    pub async fn notify(&self, method: &str, params: Option<Value>) -> Result<()>;
    // set_notification_handler / set_request_handler as above
}
```

The TCP client connects with `TcpStream::connect`, splits into read/write halves, and uses the same `Content-Length` framing as stdio.

## Related

- [Transport](/docs/api/transport)
- [Error](/docs/api/error)
- [Transport and protocol concept](/docs/core-concepts/transport-and-protocol)
