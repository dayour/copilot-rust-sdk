// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! JSON-RPC 2.0 client for the Copilot SDK.
//!
//! Provides bidirectional JSON-RPC communication over any transport.

use crate::error::{CopilotError, Result};
use crate::transport::{MessageFramer, MessageReader, MessageWriter, StdioTransport, Transport};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot, watch, Mutex, RwLock};

// =============================================================================
// JSON-RPC 2.0 Message Types
// =============================================================================

/// JSON-RPC request ID (can be string or integer).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcId {
    Num(i64),
    Str(String),
}

impl From<i64> for JsonRpcId {
    fn from(n: i64) -> Self {
        Self::Num(n)
    }
}

impl From<String> for JsonRpcId {
    fn from(s: String) -> Self {
        Self::Str(s)
    }
}

impl From<&str> for JsonRpcId {
    fn from(s: &str) -> Self {
        Self::Str(s.to_string())
    }
}

/// JSON-RPC 2.0 Request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonRpcId>,
}

impl JsonRpcRequest {
    /// Create a new request.
    pub fn new(method: impl Into<String>, params: Option<Value>, id: Option<JsonRpcId>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id,
        }
    }

    /// Create a notification (no id).
    pub fn notification(method: impl Into<String>, params: Option<Value>) -> Self {
        Self::new(method, params, None)
    }

    /// Check if this is a notification.
    pub fn is_notification(&self) -> bool {
        self.id.is_none()
    }
}

/// JSON-RPC 2.0 Error object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    /// Create a new error.
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    /// Create an error with data.
    pub fn with_data(code: i32, message: impl Into<String>, data: Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }

    /// Standard error codes.
    pub const PARSE_ERROR: i32 = -32700;
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
}

/// JSON-RPC 2.0 Response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<JsonRpcId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Create a success response.
    pub fn success(id: JsonRpcId, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            result: Some(result),
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(id: JsonRpcId, error: JsonRpcError) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            result: None,
            error: Some(error),
        }
    }

    /// Check if this is an error response.
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

// =============================================================================
// Handler Types
// =============================================================================

/// Handler for incoming notifications.
pub type NotificationHandler = Arc<dyn Fn(&str, &Value) + Send + Sync>;

/// Future returned by async request handlers.
pub type RequestHandlerFuture =
    Pin<Box<dyn std::future::Future<Output = std::result::Result<Value, JsonRpcError>> + Send>>;

/// Handler for incoming requests (returns result or error).
pub type RequestHandler = Arc<dyn Fn(&str, &Value) -> RequestHandlerFuture + Send + Sync>;

// =============================================================================
// Pending Request Tracking
// =============================================================================

struct PendingRequest {
    sender: oneshot::Sender<Result<Value>>,
}

#[derive(Clone, Copy)]
enum CloseReason {
    Disconnected,
    Shutdown,
}

impl CloseReason {
    fn error(self) -> CopilotError {
        match self {
            Self::Disconnected => CopilotError::ConnectionClosed,
            Self::Shutdown => CopilotError::Shutdown,
        }
    }
}

#[derive(Default)]
struct PendingRequests {
    requests: HashMap<i64, PendingRequest>,
    closed: Option<CloseReason>,
}

struct ConnectionState {
    running: AtomicBool,
    pending: Mutex<PendingRequests>,
    closed: watch::Sender<Option<CloseReason>>,
    malformed: AtomicU64,
}

impl ConnectionState {
    fn new() -> Self {
        Self {
            running: AtomicBool::new(false),
            pending: Mutex::new(PendingRequests::default()),
            closed: watch::channel(None).0,
            malformed: AtomicU64::new(0),
        }
    }

    async fn start(&self) -> Result<bool> {
        let pending = self.pending.lock().await;
        if let Some(reason) = pending.closed {
            return Err(reason.error());
        }
        Ok(!self.running.swap(true, Ordering::SeqCst))
    }

    async fn close(&self, reason: CloseReason) {
        // Registration and closure share a lock so no request can miss the drain.
        let mut pending = self.pending.lock().await;
        if pending.closed.is_some() {
            return;
        }
        pending.closed = Some(reason);
        self.running.store(false, Ordering::SeqCst);
        self.closed.send_replace(Some(reason));
        for (_, request) in pending.requests.drain() {
            let _ = request.sender.send(Err(reason.error()));
        }
    }

    async fn closed(&self) -> CopilotError {
        let mut receiver = self.closed.subscribe();
        loop {
            if let Some(reason) = *receiver.borrow_and_update() {
                return reason.error();
            }
            // The sender lives at least as long as this borrowed state.
            let _ = receiver.changed().await;
        }
    }

    async fn send(&self, write: impl Future<Output = Result<()>>) -> Result<()> {
        tokio::select! {
            biased;
            error = self.closed() => Err(error),
            result = write => {
                if result.is_err() {
                    self.close(CloseReason::Disconnected).await;
                    return Err(self.closed().await);
                }
                Ok(())
            }
        }
    }

    async fn invoke(
        &self,
        id: i64,
        timeout: Duration,
        send: impl Future<Output = Result<()>>,
    ) -> Result<Value> {
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            if let Some(reason) = pending.closed {
                return Err(reason.error());
            }
            pending.requests.insert(id, PendingRequest { sender: tx });
        }

        if let Err(error) = send.await {
            self.pending.lock().await.requests.remove(&id);
            return Err(error);
        }

        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(result)) => result,
            outcome => {
                let mut pending = self.pending.lock().await;
                pending.requests.remove(&id);
                Err(match pending.closed {
                    Some(reason) => reason.error(),
                    None if outcome.is_err() => CopilotError::Timeout(timeout),
                    None => CopilotError::ConnectionClosed,
                })
            }
        }
    }

    async fn respond(&self, response: JsonRpcResponse) {
        let Some(JsonRpcId::Num(id)) = response.id else {
            return;
        };
        let mut pending = self.pending.lock().await;
        if let Some(request) = pending.requests.remove(&id) {
            let result = match response.error {
                Some(error) => Err(CopilotError::JsonRpc {
                    code: error.code,
                    message: error.message,
                    data: error.data,
                }),
                None => Ok(response.result.unwrap_or(Value::Null)),
            };
            let _ = request.sender.send(result);
        }
    }

    fn malformed(&self, category: &'static str) {
        self.malformed.fetch_add(1, Ordering::Relaxed);
        tracing::warn!(category, "Discarding malformed JSON-RPC message");
    }

    fn parse(&self, message: &str) -> Option<Value> {
        let message: Value = match serde_json::from_str(message) {
            Ok(message) => message,
            Err(_) => {
                self.malformed("json");
                return None;
            }
        };
        let valid = message.get("jsonrpc").and_then(Value::as_str) == Some("2.0")
            && if message.get("method").is_some() {
                message.get("method").is_some_and(Value::is_string)
                    && message.get("result").is_none()
                    && message.get("error").is_none()
                    && message
                        .get("params")
                        .is_none_or(|p| p.is_null() || p.is_object() || p.is_array())
            } else {
                message.get("id").is_some_and(|id| !id.is_null())
                    && (message.get("result").is_some() ^ message.get("error").is_some())
                    && message.get("error").is_none_or(Value::is_object)
            };
        if !valid {
            self.malformed("envelope");
            return None;
        }
        Some(message)
    }

    async fn read_failed(&self, error: &CopilotError) {
        if matches!(error, CopilotError::Protocol(_) | CopilotError::Json(_)) {
            self.malformed("framing");
        }
        // A failed frame may have consumed an unknown number of bytes; do not
        // attempt to resynchronize, or repeatedly retry a failed IO operation.
        self.close(CloseReason::Disconnected).await;
    }
}

// =============================================================================
// Shared State (for background task)
// =============================================================================

struct SharedState<T: Transport> {
    framer: Mutex<MessageFramer<T>>,
    connection: ConnectionState,
    notification_handler: RwLock<Option<NotificationHandler>>,
    request_handler: RwLock<Option<RequestHandler>>,
}

// =============================================================================
// JSON-RPC Client
// =============================================================================

/// JSON-RPC 2.0 client with bidirectional communication.
///
/// Features:
/// - Send requests and await responses (with timeout)
/// - Send notifications (fire-and-forget)
/// - Handle incoming notifications via callback
/// - Handle incoming requests (server-to-client calls) via callback
/// - Background read loop with automatic dispatch
pub struct JsonRpcClient<T: Transport> {
    state: Arc<SharedState<T>>,
    next_id: AtomicI64,
    shutdown_tx: Mutex<Option<mpsc::Sender<()>>>,
}

impl<T: Transport + 'static> JsonRpcClient<T> {
    /// Create a new JSON-RPC client wrapping a transport.
    pub fn new(transport: T) -> Self {
        Self {
            state: Arc::new(SharedState {
                framer: Mutex::new(MessageFramer::new(transport)),
                connection: ConnectionState::new(),
                notification_handler: RwLock::new(None),
                request_handler: RwLock::new(None),
            }),
            next_id: AtomicI64::new(1),
            shutdown_tx: Mutex::new(None),
        }
    }

    /// Start the background read loop.
    pub async fn start(&self) -> Result<()> {
        if !self.state.connection.start().await? {
            return Ok(()); // Already running
        }
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        *self.shutdown_tx.lock().await = Some(shutdown_tx);

        // Clone Arc for the background task
        let state = Arc::clone(&self.state);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = shutdown_rx.recv() => {
                        state.connection.close(CloseReason::Shutdown).await;
                        break;
                    }
                    _ = state.connection.closed() => {
                        break;
                    }
                    result = async {
                        let mut framer = state.framer.lock().await;
                        framer.read_message().await
                    } => {
                        match result {
                            Ok(message_str) => {
                                if let Some(message) = state.connection.parse(&message_str) {
                                    tokio::select! {
                                        biased;
                                        _ = state.connection.closed() => break,
                                        _ = Self::dispatch_message(&state, message) => {}
                                    }
                                }
                            }
                            Err(error) => {
                                state.connection.read_failed(&error).await;
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the client.
    pub async fn stop(&self) {
        self.state.connection.close(CloseReason::Shutdown).await;
    }

    pub(crate) async fn disconnect(&self) {
        self.state.connection.close(CloseReason::Disconnected).await;
    }

    /// Check if client is running.
    pub fn is_running(&self) -> bool {
        self.state.connection.running.load(Ordering::SeqCst)
    }

    /// Number of malformed inbound messages or frames observed by this client.
    pub fn malformed_message_count(&self) -> u64 {
        self.state.connection.malformed.load(Ordering::Relaxed)
    }

    /// Set handler for incoming notifications.
    pub async fn set_notification_handler<F>(&self, handler: F)
    where
        F: Fn(&str, &Value) + Send + Sync + 'static,
    {
        *self.state.notification_handler.write().await = Some(Arc::new(handler));
    }

    /// Set handler for incoming requests.
    pub async fn set_request_handler<F>(&self, handler: F)
    where
        F: Fn(&str, &Value) -> RequestHandlerFuture + Send + Sync + 'static,
    {
        *self.state.request_handler.write().await = Some(Arc::new(handler));
    }

    /// Send a request and await response.
    pub async fn invoke(&self, method: &str, params: Option<Value>) -> Result<Value> {
        self.invoke_with_timeout(method, params, Duration::from_secs(30))
            .await
    }

    /// Send a request with custom timeout.
    pub async fn invoke_with_timeout(
        &self,
        method: &str,
        params: Option<Value>,
        timeout: Duration,
    ) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let request = JsonRpcRequest::new(method, params, Some(JsonRpcId::Num(id)));
        let request_json = serde_json::to_string(&request)?;
        self.state
            .connection
            .invoke(id, timeout, self.send_raw(&request_json))
            .await
    }

    /// Send a notification (no response expected).
    pub async fn notify(&self, method: &str, params: Option<Value>) -> Result<()> {
        let request = JsonRpcRequest::notification(method, params);
        let request_json = serde_json::to_string(&request)?;
        self.send_raw(&request_json).await
    }

    /// Send a response to an incoming request.
    pub async fn send_response(&self, id: JsonRpcId, result: Value) -> Result<()> {
        let response = JsonRpcResponse::success(id, result);
        let response_json = serde_json::to_string(&response)?;
        self.send_raw(&response_json).await
    }

    /// Send an error response to an incoming request.
    pub async fn send_error_response(&self, id: JsonRpcId, error: JsonRpcError) -> Result<()> {
        let response = JsonRpcResponse::error(id, error);
        let response_json = serde_json::to_string(&response)?;
        self.send_raw(&response_json).await
    }

    /// Send a raw JSON-RPC message.
    async fn send_raw(&self, message: &str) -> Result<()> {
        self.state
            .connection
            .send(async {
                let mut framer = self.state.framer.lock().await;
                framer.write_message(message).await
            })
            .await
    }

    /// Dispatch an incoming message.
    async fn dispatch_message(state: &SharedState<T>, message: Value) {
        // Check if it's a response (has id and result/error, no method)
        if message.get("id").is_some()
            && !message.get("id").map(|v| v.is_null()).unwrap_or(true)
            && (message.get("result").is_some() || message.get("error").is_some())
            && message.get("method").is_none()
        {
            Self::handle_response(state, message).await;
            return;
        }

        // Check if it's a request or notification (has method)
        if message.get("method").is_some() {
            if let Ok(request) = serde_json::from_value::<JsonRpcRequest>(message) {
                if request.is_notification() {
                    Self::handle_notification(state, &request).await;
                } else {
                    Self::handle_request(state, &request).await;
                }
            } else {
                state.connection.malformed("request");
            }
        }
    }

    /// Handle an incoming response.
    async fn handle_response(state: &SharedState<T>, message: Value) {
        // Parse response
        let response: JsonRpcResponse = match serde_json::from_value(message) {
            Ok(r) => r,
            Err(_) => {
                state.connection.malformed("response");
                return;
            }
        };
        state.connection.respond(response).await;
    }

    /// Handle an incoming notification.
    async fn handle_notification(state: &SharedState<T>, request: &JsonRpcRequest) {
        let handler = state.notification_handler.read().await;
        if let Some(handler) = handler.as_ref() {
            let params = request.params.as_ref().unwrap_or(&Value::Null);
            handler(&request.method, params);
        }
    }

    /// Handle an incoming request.
    async fn handle_request(state: &SharedState<T>, request: &JsonRpcRequest) {
        let id = match &request.id {
            Some(id) => id.clone(),
            None => return, // Not a request
        };

        let handler = state.request_handler.read().await;
        let params = request.params.as_ref().unwrap_or(&Value::Null);

        let response = if let Some(handler) = handler.as_ref() {
            // Call the async handler and await result
            match handler(&request.method, params).await {
                Ok(result) => JsonRpcResponse::success(id, result),
                Err(error) => JsonRpcResponse::error(id, error),
            }
        } else {
            // No handler - respond with method not found
            JsonRpcResponse::error(
                id,
                JsonRpcError::new(
                    JsonRpcError::METHOD_NOT_FOUND,
                    format!("Method not found: {}", request.method),
                ),
            )
        };

        // Send response
        if let Ok(response_json) = serde_json::to_string(&response) {
            let _ = state
                .connection
                .send(async {
                    let mut framer = state.framer.lock().await;
                    framer.write_message(&response_json).await
                })
                .await;
        }
    }
}

// =============================================================================
// Stdio JSON-RPC Client (split read/write paths)
// =============================================================================

/// Shared state for the Stdio JSON-RPC client.
struct StdioSharedState {
    writer: Mutex<MessageWriter<tokio::process::ChildStdin>>,
    connection: ConnectionState,
    notification_handler: RwLock<Option<NotificationHandler>>,
    request_handler: RwLock<Option<RequestHandler>>,
}

/// JSON-RPC client for stdio transports with separate read/write paths.
///
/// This client avoids the lock contention issue by using separate mutexes
/// for reading and writing.
pub struct StdioJsonRpcClient {
    state: Arc<StdioSharedState>,
    reader: Mutex<Option<MessageReader<tokio::process::ChildStdout>>>,
    next_id: AtomicI64,
    shutdown_tx: Mutex<Option<mpsc::Sender<()>>>,
}

impl StdioJsonRpcClient {
    /// Create a new stdio JSON-RPC client from a transport.
    pub fn new(transport: StdioTransport) -> Self {
        let (writer, reader) = transport.split();
        Self {
            state: Arc::new(StdioSharedState {
                writer: Mutex::new(MessageWriter::new(writer)),
                connection: ConnectionState::new(),
                notification_handler: RwLock::new(None),
                request_handler: RwLock::new(None),
            }),
            reader: Mutex::new(Some(MessageReader::new(reader))),
            next_id: AtomicI64::new(1),
            shutdown_tx: Mutex::new(None),
        }
    }

    /// Start the background read loop.
    pub async fn start(&self) -> Result<()> {
        let reader = self.reader.lock().await.take().ok_or_else(|| {
            CopilotError::InvalidConfig("Reader already taken or client already started".into())
        })?;
        self.start_with_reader(reader).await
    }

    /// Start the background read loop with a specific reader.
    async fn start_with_reader(
        &self,
        mut reader: MessageReader<tokio::process::ChildStdout>,
    ) -> Result<()> {
        if !self.state.connection.start().await? {
            return Ok(()); // Already running
        }
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        *self.shutdown_tx.lock().await = Some(shutdown_tx);

        // Clone state for the background task
        let state = Arc::clone(&self.state);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = shutdown_rx.recv() => {
                        state.connection.close(CloseReason::Shutdown).await;
                        break;
                    }
                    _ = state.connection.closed() => {
                        break;
                    }
                    result = reader.read_message() => {
                        match result {
                            Ok(message_str) => {
                                if let Some(message) = state.connection.parse(&message_str) {
                                    tokio::select! {
                                        biased;
                                        _ = state.connection.closed() => break,
                                        _ = Self::dispatch_message(&state, message) => {}
                                    }
                                }
                            }
                            Err(error) => {
                                state.connection.read_failed(&error).await;
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the client.
    pub async fn stop(&self) {
        self.state.connection.close(CloseReason::Shutdown).await;
    }

    pub(crate) async fn disconnect(&self) {
        self.state.connection.close(CloseReason::Disconnected).await;
    }

    /// Check if client is running.
    pub fn is_running(&self) -> bool {
        self.state.connection.running.load(Ordering::SeqCst)
    }

    /// Number of malformed inbound messages or frames observed by this client.
    pub fn malformed_message_count(&self) -> u64 {
        self.state.connection.malformed.load(Ordering::Relaxed)
    }

    /// Set handler for incoming notifications.
    pub async fn set_notification_handler<F>(&self, handler: F)
    where
        F: Fn(&str, &Value) + Send + Sync + 'static,
    {
        *self.state.notification_handler.write().await = Some(Arc::new(handler));
    }

    /// Set handler for incoming requests.
    pub async fn set_request_handler<F>(&self, handler: F)
    where
        F: Fn(&str, &Value) -> RequestHandlerFuture + Send + Sync + 'static,
    {
        *self.state.request_handler.write().await = Some(Arc::new(handler));
    }

    /// Send a request and await response.
    pub async fn invoke(&self, method: &str, params: Option<Value>) -> Result<Value> {
        self.invoke_with_timeout(method, params, Duration::from_secs(30))
            .await
    }

    /// Send a request with custom timeout.
    pub async fn invoke_with_timeout(
        &self,
        method: &str,
        params: Option<Value>,
        timeout: Duration,
    ) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let request = JsonRpcRequest::new(method, params, Some(JsonRpcId::Num(id)));
        let request_json = serde_json::to_string(&request)?;
        self.state
            .connection
            .invoke(id, timeout, self.send_raw(&request_json))
            .await
    }

    /// Send a notification (no response expected).
    pub async fn notify(&self, method: &str, params: Option<Value>) -> Result<()> {
        let request = JsonRpcRequest::notification(method, params);
        let request_json = serde_json::to_string(&request)?;
        self.send_raw(&request_json).await
    }

    /// Send a raw JSON-RPC message.
    async fn send_raw(&self, message: &str) -> Result<()> {
        self.state
            .connection
            .send(async {
                let mut writer = self.state.writer.lock().await;
                writer.write_message(message).await
            })
            .await
    }

    /// Dispatch an incoming message.
    async fn dispatch_message(state: &StdioSharedState, message: Value) {
        // Check if it's a response (has id and result/error, no method)
        if message.get("id").is_some()
            && !message.get("id").map(|v| v.is_null()).unwrap_or(true)
            && (message.get("result").is_some() || message.get("error").is_some())
            && message.get("method").is_none()
        {
            Self::handle_response(state, message).await;
            return;
        }

        // Check if it's a request or notification (has method)
        if message.get("method").is_some() {
            if let Ok(request) = serde_json::from_value::<JsonRpcRequest>(message) {
                if request.is_notification() {
                    Self::handle_notification(state, &request).await;
                } else {
                    Self::handle_request(state, &request).await;
                }
            } else {
                state.connection.malformed("request");
            }
        }
    }

    /// Handle an incoming response.
    async fn handle_response(state: &StdioSharedState, message: Value) {
        // Parse response
        let response: JsonRpcResponse = match serde_json::from_value(message) {
            Ok(r) => r,
            Err(_) => {
                state.connection.malformed("response");
                return;
            }
        };
        state.connection.respond(response).await;
    }

    /// Handle an incoming notification.
    async fn handle_notification(state: &StdioSharedState, request: &JsonRpcRequest) {
        let handler = state.notification_handler.read().await;
        if let Some(handler) = handler.as_ref() {
            let params = request.params.as_ref().unwrap_or(&Value::Null);
            handler(&request.method, params);
        }
    }

    /// Handle an incoming request.
    async fn handle_request(state: &StdioSharedState, request: &JsonRpcRequest) {
        let id = match &request.id {
            Some(id) => id.clone(),
            None => return,
        };

        let handler = state.request_handler.read().await;
        let params = request.params.as_ref().unwrap_or(&Value::Null);

        let response = if let Some(handler) = handler.as_ref() {
            // Call the async handler and await result
            match handler(&request.method, params).await {
                Ok(result) => JsonRpcResponse::success(id.clone(), result),
                Err(error) => JsonRpcResponse::error(id.clone(), error),
            }
        } else {
            JsonRpcResponse::error(
                id.clone(),
                JsonRpcError::new(
                    JsonRpcError::METHOD_NOT_FOUND,
                    format!("Method not found: {}", request.method),
                ),
            )
        };

        // Send response
        if let Ok(response_json) = serde_json::to_string(&response) {
            let _ = state
                .connection
                .send(async {
                    let mut writer = state.writer.lock().await;
                    writer.write_message(&response_json).await
                })
                .await;
        }
    }
}

// =============================================================================
// TCP JSON-RPC Client (split read/write paths)
// =============================================================================

/// Shared state for the TCP JSON-RPC client.
struct TcpSharedState {
    writer: Mutex<MessageWriter<OwnedWriteHalf>>,
    connection: ConnectionState,
    notification_handler: RwLock<Option<NotificationHandler>>,
    request_handler: RwLock<Option<RequestHandler>>,
}

/// JSON-RPC client for TCP transports with separate read/write paths.
pub struct TcpJsonRpcClient {
    state: Arc<TcpSharedState>,
    reader: Mutex<Option<MessageReader<OwnedReadHalf>>>,
    next_id: AtomicI64,
    shutdown_tx: Mutex<Option<mpsc::Sender<()>>>,
}

impl TcpJsonRpcClient {
    /// Connect to a TCP JSON-RPC server.
    pub async fn connect(addr: impl AsRef<str>) -> Result<Self> {
        let stream = TcpStream::connect(addr.as_ref())
            .await
            .map_err(CopilotError::Transport)?;
        Ok(Self::new(stream))
    }

    /// Create a new TCP JSON-RPC client from a connected socket.
    pub fn new(stream: TcpStream) -> Self {
        let (reader, writer) = stream.into_split();
        Self {
            state: Arc::new(TcpSharedState {
                writer: Mutex::new(MessageWriter::new(writer)),
                connection: ConnectionState::new(),
                notification_handler: RwLock::new(None),
                request_handler: RwLock::new(None),
            }),
            reader: Mutex::new(Some(MessageReader::new(reader))),
            next_id: AtomicI64::new(1),
            shutdown_tx: Mutex::new(None),
        }
    }

    /// Start the background read loop.
    pub async fn start(&self) -> Result<()> {
        let reader = self.reader.lock().await.take().ok_or_else(|| {
            CopilotError::InvalidConfig("Reader already taken or client already started".into())
        })?;
        self.start_with_reader(reader).await
    }

    async fn start_with_reader(&self, mut reader: MessageReader<OwnedReadHalf>) -> Result<()> {
        if !self.state.connection.start().await? {
            return Ok(()); // Already running
        }
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        *self.shutdown_tx.lock().await = Some(shutdown_tx);

        let state = Arc::clone(&self.state);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = shutdown_rx.recv() => {
                        state.connection.close(CloseReason::Shutdown).await;
                        break;
                    }
                    _ = state.connection.closed() => {
                        break;
                    }
                    result = reader.read_message() => {
                        match result {
                            Ok(message_str) => {
                                if let Some(message) = state.connection.parse(&message_str) {
                                    tokio::select! {
                                        biased;
                                        _ = state.connection.closed() => break,
                                        _ = Self::dispatch_message(&state, message) => {}
                                    }
                                }
                            }
                            Err(error) => {
                                state.connection.read_failed(&error).await;
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the client.
    pub async fn stop(&self) {
        self.state.connection.close(CloseReason::Shutdown).await;
    }

    pub(crate) async fn disconnect(&self) {
        self.state.connection.close(CloseReason::Disconnected).await;
    }

    /// Check if client is running.
    pub fn is_running(&self) -> bool {
        self.state.connection.running.load(Ordering::SeqCst)
    }

    /// Number of malformed inbound messages or frames observed by this client.
    pub fn malformed_message_count(&self) -> u64 {
        self.state.connection.malformed.load(Ordering::Relaxed)
    }

    /// Set handler for incoming notifications.
    pub async fn set_notification_handler<F>(&self, handler: F)
    where
        F: Fn(&str, &Value) + Send + Sync + 'static,
    {
        *self.state.notification_handler.write().await = Some(Arc::new(handler));
    }

    /// Set handler for incoming requests.
    pub async fn set_request_handler<F>(&self, handler: F)
    where
        F: Fn(&str, &Value) -> RequestHandlerFuture + Send + Sync + 'static,
    {
        *self.state.request_handler.write().await = Some(Arc::new(handler));
    }

    /// Send a request and await response.
    pub async fn invoke(&self, method: &str, params: Option<Value>) -> Result<Value> {
        self.invoke_with_timeout(method, params, Duration::from_secs(30))
            .await
    }

    /// Send a request with custom timeout.
    pub async fn invoke_with_timeout(
        &self,
        method: &str,
        params: Option<Value>,
        timeout: Duration,
    ) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);

        let request = JsonRpcRequest::new(method, params, Some(JsonRpcId::Num(id)));
        let request_json = serde_json::to_string(&request)?;
        self.state
            .connection
            .invoke(id, timeout, self.send_raw(&request_json))
            .await
    }

    /// Send a notification (no response expected).
    pub async fn notify(&self, method: &str, params: Option<Value>) -> Result<()> {
        let request = JsonRpcRequest::notification(method, params);
        let request_json = serde_json::to_string(&request)?;
        self.send_raw(&request_json).await
    }

    async fn send_raw(&self, message: &str) -> Result<()> {
        self.state
            .connection
            .send(async {
                let mut writer = self.state.writer.lock().await;
                writer.write_message(message).await
            })
            .await
    }

    async fn dispatch_message(state: &TcpSharedState, message: Value) {
        if message.get("id").is_some()
            && !message.get("id").map(|v| v.is_null()).unwrap_or(true)
            && (message.get("result").is_some() || message.get("error").is_some())
            && message.get("method").is_none()
        {
            Self::handle_response(state, message).await;
            return;
        }

        if message.get("method").is_some() {
            if let Ok(request) = serde_json::from_value::<JsonRpcRequest>(message) {
                if request.is_notification() {
                    Self::handle_notification(state, &request).await;
                } else {
                    Self::handle_request(state, &request).await;
                }
            } else {
                state.connection.malformed("request");
            }
        }
    }

    async fn handle_response(state: &TcpSharedState, message: Value) {
        let response: JsonRpcResponse = match serde_json::from_value(message) {
            Ok(r) => r,
            Err(_) => {
                state.connection.malformed("response");
                return;
            }
        };
        state.connection.respond(response).await;
    }

    async fn handle_notification(state: &TcpSharedState, request: &JsonRpcRequest) {
        let handler = state.notification_handler.read().await;
        if let Some(handler) = handler.as_ref() {
            let params = request.params.as_ref().unwrap_or(&Value::Null);
            handler(&request.method, params);
        }
    }

    async fn handle_request(state: &TcpSharedState, request: &JsonRpcRequest) {
        let id = match &request.id {
            Some(id) => id.clone(),
            None => return,
        };

        let handler = state.request_handler.read().await;
        let params = request.params.as_ref().unwrap_or(&Value::Null);

        let response = if let Some(handler) = handler.as_ref() {
            match handler(&request.method, params).await {
                Ok(result) => JsonRpcResponse::success(id.clone(), result),
                Err(error) => JsonRpcResponse::error(id.clone(), error),
            }
        } else {
            JsonRpcResponse::error(
                id.clone(),
                JsonRpcError::new(
                    JsonRpcError::METHOD_NOT_FOUND,
                    format!("Method not found: {}", request.method),
                ),
            )
        };

        if let Ok(response_json) = serde_json::to_string(&response) {
            let _ = state
                .connection
                .send(async {
                    let mut writer = state.writer.lock().await;
                    writer.write_message(&response_json).await
                })
                .await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::MemoryTransport;
    use serde_json::json;

    #[tokio::test]
    async fn disconnect_cancels_sends_waiting_for_framer() {
        let client = JsonRpcClient::new(MemoryTransport::new(Vec::new()));
        let _framer = client.state.framer.lock().await;
        let request = client.invoke("ping", None);
        let notification = client.notify("ping", None);
        tokio::pin!(request, notification);
        assert!(futures::poll!(&mut request).is_pending());
        assert!(futures::poll!(&mut notification).is_pending());
        client.disconnect().await;
        assert!(matches!(
            tokio::time::timeout(Duration::from_secs(1), request)
                .await
                .unwrap(),
            Err(CopilotError::ConnectionClosed)
        ));
        assert!(matches!(
            tokio::time::timeout(Duration::from_secs(1), notification)
                .await
                .unwrap(),
            Err(CopilotError::ConnectionClosed)
        ));
        assert!(client
            .state
            .connection
            .pending
            .lock()
            .await
            .requests
            .is_empty());
        assert!(matches!(
            client.start().await,
            Err(CopilotError::ConnectionClosed)
        ));
    }

    #[tokio::test]
    async fn disconnected_request_cannot_become_timeout() {
        for reason in [CloseReason::Disconnected, CloseReason::Shutdown] {
            let connection = ConnectionState::new();
            let request = connection.invoke(1, Duration::from_millis(10), async { Ok(()) });
            tokio::pin!(request);
            assert!(futures::poll!(&mut request).is_pending());
            connection.close(reason).await;
            // Even if the timeout is ready when the caller resumes, closure wins.
            tokio::time::sleep(Duration::from_millis(20)).await;
            let error = request.await.unwrap_err();
            match reason {
                CloseReason::Disconnected => {
                    assert!(matches!(error, CopilotError::ConnectionClosed))
                }
                CloseReason::Shutdown => assert!(matches!(error, CopilotError::Shutdown)),
            }
        }
    }

    #[tokio::test]
    async fn first_terminal_reason_wins() {
        let connection = ConnectionState::new();
        connection.close(CloseReason::Shutdown).await;
        connection.close(CloseReason::Disconnected).await;
        assert!(matches!(connection.closed().await, CopilotError::Shutdown));
        assert!(matches!(
            connection.invoke(1, Duration::ZERO, async { Ok(()) }).await,
            Err(CopilotError::Shutdown)
        ));
        assert!(connection.pending.lock().await.requests.is_empty());
    }

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest::new(
            "test_method",
            Some(json!({"key": "value"})),
            Some(JsonRpcId::Num(1)),
        );

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["method"], "test_method");
        assert_eq!(json["params"]["key"], "value");
        assert_eq!(json["id"], 1);
    }

    #[test]
    fn test_json_rpc_notification_serialization() {
        let request = JsonRpcRequest::notification("notify_method", Some(json!([1, 2, 3])));

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["method"], "notify_method");
        assert!(json.get("id").is_none());
    }

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse::success(JsonRpcId::Num(1), json!({"result": "ok"}));

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        assert_eq!(json["result"]["result"], "ok");
        assert!(json.get("error").is_none());
    }

    #[test]
    fn test_json_rpc_response_error() {
        let response = JsonRpcResponse::error(
            JsonRpcId::Num(1),
            JsonRpcError::new(-32600, "Invalid Request"),
        );

        let json = serde_json::to_value(&response).unwrap();
        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        assert_eq!(json["error"]["code"], -32600);
        assert_eq!(json["error"]["message"], "Invalid Request");
    }

    #[test]
    fn test_json_rpc_id_from_i64() {
        let id: JsonRpcId = 42i64.into();
        assert_eq!(id, JsonRpcId::Num(42));
    }

    #[test]
    fn test_json_rpc_id_from_string() {
        let id: JsonRpcId = "test-id".into();
        assert_eq!(id, JsonRpcId::Str("test-id".to_string()));
    }

    #[test]
    fn test_json_rpc_error_constants() {
        assert_eq!(JsonRpcError::PARSE_ERROR, -32700);
        assert_eq!(JsonRpcError::INVALID_REQUEST, -32600);
        assert_eq!(JsonRpcError::METHOD_NOT_FOUND, -32601);
        assert_eq!(JsonRpcError::INVALID_PARAMS, -32602);
        assert_eq!(JsonRpcError::INTERNAL_ERROR, -32603);
    }

    #[test]
    fn test_request_is_notification() {
        let request = JsonRpcRequest::notification("method", None);
        assert!(request.is_notification());

        let request = JsonRpcRequest::new("method", None, Some(JsonRpcId::Num(1)));
        assert!(!request.is_notification());
    }

    #[tokio::test]
    async fn test_large_payload_64kb_boundary() {
        // Create a payload near 64KB (65536 bytes)
        let large_data = "x".repeat(65536 - 50); // account for JSON wrapper
        let msg =
            serde_json::json!({"jsonrpc": "2.0", "method": "test", "params": {"data": large_data}});
        let msg_str = serde_json::to_string(&msg).unwrap();

        // Write with framer
        let transport = MemoryTransport::new(Vec::new());
        let mut framer = MessageFramer::new(transport);
        framer.write_message(&msg_str).await.unwrap();

        // Read back from written data
        let written = framer.transport().written_data().to_vec();
        let transport2 = MemoryTransport::new(written);
        let mut framer2 = MessageFramer::new(transport2);
        let read_back = framer2.read_message().await.unwrap();
        assert_eq!(msg_str, read_back);
    }

    #[tokio::test]
    async fn test_large_payload_100kb() {
        let large_data = "y".repeat(100_000);
        let msg =
            serde_json::json!({"jsonrpc": "2.0", "method": "test", "params": {"data": large_data}});
        let msg_str = serde_json::to_string(&msg).unwrap();

        let transport = MemoryTransport::new(Vec::new());
        let mut framer = MessageFramer::new(transport);
        framer.write_message(&msg_str).await.unwrap();

        let written = framer.transport().written_data().to_vec();
        let transport2 = MemoryTransport::new(written);
        let mut framer2 = MessageFramer::new(transport2);
        let read_back = framer2.read_message().await.unwrap();
        assert_eq!(msg_str, read_back);
    }

    #[tokio::test]
    async fn test_multiple_large_messages_sequential() {
        let msg1_data = "a".repeat(50_000);
        let msg2_data = "b".repeat(80_000);
        let msg1 = serde_json::json!({"jsonrpc": "2.0", "id": 1, "method": "test1", "params": {"data": msg1_data}});
        let msg2 = serde_json::json!({"jsonrpc": "2.0", "id": 2, "method": "test2", "params": {"data": msg2_data}});
        let msg1_str = serde_json::to_string(&msg1).unwrap();
        let msg2_str = serde_json::to_string(&msg2).unwrap();

        // Write both messages
        let transport = MemoryTransport::new(Vec::new());
        let mut framer = MessageFramer::new(transport);
        framer.write_message(&msg1_str).await.unwrap();
        framer.write_message(&msg2_str).await.unwrap();

        // Read both back
        let written = framer.transport().written_data().to_vec();
        let transport2 = MemoryTransport::new(written);
        let mut framer2 = MessageFramer::new(transport2);
        let read1 = framer2.read_message().await.unwrap();
        let read2 = framer2.read_message().await.unwrap();
        assert_eq!(msg1_str, read1);
        assert_eq!(msg2_str, read2);
    }
}
