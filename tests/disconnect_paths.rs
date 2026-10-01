// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

use copilot_sdk::error::{CopilotError, Result};
use copilot_sdk::jsonrpc::{JsonRpcClient, TcpJsonRpcClient};
use copilot_sdk::transport::Transport;
use serde_json::Value;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt, DuplexStream};
use tokio::net::{TcpListener, TcpStream};

const DEADLINE: Duration = Duration::from_secs(2);

struct TestTransport {
    stream: DuplexStream,
    fail_read: bool,
    fail_write: Arc<AtomicBool>,
    reads: Arc<AtomicUsize>,
}

impl Transport for TestTransport {
    fn read<'a>(
        &'a mut self,
        buffer: &'a mut [u8],
    ) -> Pin<Box<dyn Future<Output = Result<usize>> + Send + 'a>> {
        Box::pin(async move {
            self.reads.fetch_add(1, Ordering::SeqCst);
            if self.fail_read {
                return Err(std::io::Error::from(std::io::ErrorKind::ConnectionReset).into());
            }
            Ok(self.stream.read(buffer).await?)
        })
    }

    fn write<'a>(
        &'a mut self,
        bytes: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            if self.fail_write.load(Ordering::SeqCst) {
                return Err(std::io::Error::from(std::io::ErrorKind::BrokenPipe).into());
            }
            Ok(self.stream.write_all(bytes).await?)
        })
    }

    fn close(&mut self) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async { Ok(()) })
    }

    fn is_open(&self) -> bool {
        true
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Generic,
    Tcp,
    #[cfg(unix)]
    Stdio,
}

const KINDS: &[Kind] = &[
    Kind::Generic,
    Kind::Tcp,
    #[cfg(unix)]
    Kind::Stdio,
];

enum Client {
    Generic(JsonRpcClient<TestTransport>),
    Tcp(TcpJsonRpcClient),
    #[cfg(unix)]
    Stdio(copilot_sdk::jsonrpc::StdioJsonRpcClient),
}

macro_rules! client_call {
    ($self:expr, $client:ident, $body:expr) => {
        match $self {
            Client::Generic($client) => $body,
            Client::Tcp($client) => $body,
            #[cfg(unix)]
            Client::Stdio($client) => $body,
        }
    };
}

impl Client {
    async fn start(&self) {
        client_call!(self, client, client.start().await.unwrap());
    }

    async fn stop(&self) {
        client_call!(self, client, client.stop().await);
    }

    async fn invoke(&self, timeout: Duration) -> Result<Value> {
        client_call!(
            self,
            client,
            client.invoke_with_timeout("ping", None, timeout).await
        )
    }

    async fn notify(&self) -> Result<()> {
        client_call!(self, client, client.notify("ping", None).await)
    }

    fn running(&self) -> bool {
        client_call!(self, client, client.is_running())
    }

    fn malformed(&self) -> u64 {
        client_call!(self, client, client.malformed_message_count())
    }
}

enum Peer {
    Generic(DuplexStream),
    Tcp(TcpStream),
    #[cfg(unix)]
    Stdio(tokio::process::Child),
}

impl Peer {
    async fn close(self) {
        match self {
            Self::Generic(stream) => drop(stream),
            Self::Tcp(stream) => drop(stream),
            #[cfg(unix)]
            Self::Stdio(mut child) => {
                child.kill().await.unwrap();
                child.wait().await.unwrap();
            }
        }
    }
}

fn test_transport() -> (TestTransport, DuplexStream) {
    let (stream, peer) = tokio::io::duplex(16 * 1024);
    (
        TestTransport {
            stream,
            fail_read: false,
            fail_write: Arc::new(AtomicBool::new(false)),
            reads: Arc::new(AtomicUsize::new(0)),
        },
        peer,
    )
}

async fn tcp_pair() -> (TcpStream, TcpStream) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let (stream, accepted) = tokio::join!(
        TcpStream::connect(listener.local_addr().unwrap()),
        listener.accept()
    );
    (stream.unwrap(), accepted.unwrap().0)
}

async fn fixture(kind: Kind, inbound: &str) -> (Client, Peer) {
    match kind {
        Kind::Generic => {
            let (transport, mut peer) = test_transport();
            peer.write_all(inbound.as_bytes()).await.unwrap();
            (
                Client::Generic(JsonRpcClient::new(transport)),
                Peer::Generic(peer),
            )
        }
        Kind::Tcp => {
            let (stream, mut peer) = tcp_pair().await;
            peer.write_all(inbound.as_bytes()).await.unwrap();
            (Client::Tcp(TcpJsonRpcClient::new(stream)), Peer::Tcp(peer))
        }
        #[cfg(unix)]
        Kind::Stdio => {
            let mut child = tokio::process::Command::new("sh")
                .args([
                    "-c",
                    "printf '%s' \"$1\"; while IFS= read -r line; do :; done",
                    "sh",
                    inbound,
                ])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .kill_on_drop(true)
                .spawn()
                .unwrap();
            let transport = copilot_sdk::transport::StdioTransport::new(
                child.stdin.take().unwrap(),
                child.stdout.take().unwrap(),
            );
            (
                Client::Stdio(copilot_sdk::jsonrpc::StdioJsonRpcClient::new(transport)),
                Peer::Stdio(child),
            )
        }
    }
}

fn frame(message: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{}", message.len(), message)
}

async fn promptly<T>(future: impl Future<Output = T>) -> T {
    tokio::time::timeout(DEADLINE, future)
        .await
        .expect("operation did not complete promptly")
}

async fn stopped(client: &Client) {
    promptly(async {
        while client.running() {
            tokio::task::yield_now().await;
        }
    })
    .await;
}

#[tokio::test]
async fn eof_drains_pending_and_rejects_late_requests() {
    for &kind in KINDS {
        let (client, peer) = fixture(kind, "").await;
        let first = client.invoke(Duration::from_secs(60));
        let second = client.invoke(Duration::from_secs(60));
        tokio::pin!(first, second);
        assert!(futures::poll!(&mut first).is_pending());
        assert!(futures::poll!(&mut second).is_pending());
        client.start().await;
        peer.close().await;
        assert!(matches!(
            promptly(first).await,
            Err(CopilotError::ConnectionClosed)
        ));
        assert!(matches!(
            promptly(second).await,
            Err(CopilotError::ConnectionClosed)
        ));
        assert!(matches!(
            promptly(client.invoke(Duration::ZERO)).await,
            Err(CopilotError::ConnectionClosed)
        ));
        assert!(matches!(
            client.notify().await,
            Err(CopilotError::ConnectionClosed)
        ));
        assert!(!client.running());
        assert_eq!(client.malformed(), 0);
    }
}

#[tokio::test]
async fn shutdown_drains_pending_and_rejects_late_requests() {
    for &kind in KINDS {
        let (client, peer) = fixture(kind, "").await;
        let request = client.invoke(Duration::from_secs(60));
        tokio::pin!(request);
        assert!(futures::poll!(&mut request).is_pending());
        client.start().await;
        client.stop().await;
        assert!(matches!(
            promptly(request).await,
            Err(CopilotError::Shutdown)
        ));
        assert!(matches!(
            promptly(client.invoke(Duration::ZERO)).await,
            Err(CopilotError::Shutdown)
        ));
        assert!(matches!(client.notify().await, Err(CopilotError::Shutdown)));
        assert!(!client.running());
        peer.close().await;
    }
}

#[tokio::test]
async fn malformed_messages_are_counted_and_valid_response_recovers() {
    let malformed = [
        "{secret_payload",
        "[]",
        r#"{"jsonrpc":"1.0","id":1,"result":0}"#,
        r#"{"jsonrpc":"2.0","id":1,"result":0,"error":{"code":1,"message":"secret"}}"#,
        r#"{"jsonrpc":"2.0","method":false}"#,
        r#"{"jsonrpc":"2.0","method":"ping","id":{}}"#,
        r#"{"jsonrpc":"2.0","id":1,"error":{"code":"secret","message":"secret"}}"#,
    ];
    let inbound = malformed
        .iter()
        .map(|message| frame(message))
        .collect::<String>()
        + &frame(r#"{"jsonrpc":"2.0","id":1,"result":null}"#);
    for &kind in KINDS {
        let (client, peer) = fixture(kind, &inbound).await;
        let request = client.invoke(Duration::from_secs(60));
        tokio::pin!(request);
        assert!(futures::poll!(&mut request).is_pending());
        client.start().await;
        assert_eq!(promptly(request).await.unwrap(), Value::Null);
        assert_eq!(client.malformed(), malformed.len() as u64);
        assert!(client.running());
        client.stop().await;
        peer.close().await;
    }
}

#[tokio::test]
async fn malformed_framing_disconnects_without_retrying() {
    for &kind in KINDS {
        let (client, peer) = fixture(kind, "Content-Length: secret\r\n\r\n").await;
        let request = client.invoke(Duration::from_secs(60));
        tokio::pin!(request);
        assert!(futures::poll!(&mut request).is_pending());
        client.start().await;
        assert!(matches!(
            promptly(request).await,
            Err(CopilotError::ConnectionClosed)
        ));
        stopped(&client).await;
        assert_eq!(client.malformed(), 1);
        peer.close().await;
    }
}

#[tokio::test]
async fn genuine_server_error_is_not_reclassified_as_disconnect() {
    let inbound = frame(
        r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32801,"message":"server error","data":{"retry":true}}}"#,
    );
    for &kind in KINDS {
        let (client, peer) = fixture(kind, &inbound).await;
        let request = client.invoke(Duration::from_secs(60));
        tokio::pin!(request);
        assert!(futures::poll!(&mut request).is_pending());
        client.start().await;
        match promptly(request).await {
            Err(CopilotError::JsonRpc {
                code,
                message,
                data,
            }) => {
                assert_eq!(code, -32801);
                assert_eq!(message, "server error");
                assert_eq!(data, Some(serde_json::json!({"retry": true})));
            }
            other => panic!("expected server error, got {other:?}"),
        }
        assert!(client.running());
        client.stop().await;
        peer.close().await;
    }
}

#[tokio::test]
async fn live_unanswered_requests_still_time_out() {
    for &kind in KINDS {
        let (client, peer) = fixture(kind, "").await;
        let duration = Duration::from_millis(10);
        let request = client.invoke(duration);
        tokio::pin!(request);
        assert!(futures::poll!(&mut request).is_pending());
        client.start().await;
        assert!(matches!(promptly(request).await, Err(CopilotError::Timeout(t)) if t == duration));
        assert!(client.running());
        client.stop().await;
        peer.close().await;
    }
}

#[tokio::test]
async fn generic_fatal_read_error_is_not_retried() {
    let (mut transport, _peer) = test_transport();
    transport.fail_read = true;
    let reads = transport.reads.clone();
    let client = JsonRpcClient::new(transport);
    let request = client.invoke("ping", None);
    tokio::pin!(request);
    assert!(futures::poll!(&mut request).is_pending());
    client.start().await.unwrap();
    assert!(matches!(
        promptly(request).await,
        Err(CopilotError::ConnectionClosed)
    ));
    tokio::task::yield_now().await;
    assert_eq!(reads.load(Ordering::SeqCst), 1);
    assert_eq!(client.malformed_message_count(), 0);
    assert!(!client.is_running());
}

#[tokio::test]
async fn generic_write_failure_drains_other_pending_requests() {
    let (transport, _peer) = test_transport();
    let fail_write = transport.fail_write.clone();
    let client = JsonRpcClient::new(transport);
    let request = client.invoke("ping", None);
    tokio::pin!(request);
    assert!(futures::poll!(&mut request).is_pending());
    fail_write.store(true, Ordering::SeqCst);
    assert!(matches!(
        client.notify("ping", None).await,
        Err(CopilotError::ConnectionClosed)
    ));
    assert!(matches!(
        promptly(request).await,
        Err(CopilotError::ConnectionClosed)
    ));
    assert!(matches!(
        client.invoke("ping", None).await,
        Err(CopilotError::ConnectionClosed)
    ));
}

#[tokio::test]
async fn tcp_failed_write_closes_connection() {
    let (mut stream, _peer) = tcp_pair().await;
    stream.shutdown().await.unwrap();
    let client = TcpJsonRpcClient::new(stream);
    assert!(matches!(
        promptly(client.invoke("ping", None)).await,
        Err(CopilotError::ConnectionClosed)
    ));
    assert!(matches!(
        client.notify("ping", None).await,
        Err(CopilotError::ConnectionClosed)
    ));
}

#[cfg(unix)]
#[tokio::test]
async fn stdio_failed_write_closes_connection() {
    let mut child = tokio::process::Command::new("sh")
        .args(["-c", "exec 0<&-; printf ready; exec sleep 60"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .unwrap();
    let mut stdout = child.stdout.take().unwrap();
    let mut ready = [0; 5];
    promptly(stdout.read_exact(&mut ready)).await.unwrap();
    let client = copilot_sdk::jsonrpc::StdioJsonRpcClient::new(
        copilot_sdk::transport::StdioTransport::new(child.stdin.take().unwrap(), stdout),
    );
    assert!(matches!(
        promptly(client.invoke("ping", None)).await,
        Err(CopilotError::ConnectionClosed)
    ));
    assert!(matches!(
        client.notify("ping", None).await,
        Err(CopilotError::ConnectionClosed)
    ));
    child.kill().await.unwrap();
    child.wait().await.unwrap();
}

#[tokio::test]
async fn generic_sends_waiting_for_read_mutex_observe_shutdown() {
    let (transport, _peer) = test_transport();
    let reads = transport.reads.clone();
    let client = JsonRpcClient::new(transport);
    client.start().await.unwrap();
    promptly(async {
        while reads.load(Ordering::SeqCst) == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await;
    let request = client.invoke("ping", None);
    let notification = client.notify("ping", None);
    tokio::pin!(request, notification);
    assert!(futures::poll!(&mut request).is_pending());
    assert!(futures::poll!(&mut notification).is_pending());
    client.stop().await;
    assert!(matches!(
        promptly(request).await,
        Err(CopilotError::Shutdown)
    ));
    assert!(matches!(
        promptly(notification).await,
        Err(CopilotError::Shutdown)
    ));
}

#[tokio::test]
async fn generic_sends_waiting_for_read_mutex_observe_eof() {
    let (transport, peer) = test_transport();
    let reads = transport.reads.clone();
    let client = JsonRpcClient::new(transport);
    client.start().await.unwrap();
    promptly(async {
        while reads.load(Ordering::SeqCst) == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await;
    let request = client.invoke("ping", None);
    let notification = client.notify("ping", None);
    tokio::pin!(request, notification);
    assert!(futures::poll!(&mut request).is_pending());
    assert!(futures::poll!(&mut notification).is_pending());
    drop(peer);
    assert!(matches!(
        promptly(request).await,
        Err(CopilotError::ConnectionClosed)
    ));
    assert!(matches!(
        promptly(notification).await,
        Err(CopilotError::ConnectionClosed)
    ));
}

#[tokio::test]
async fn dropping_generic_client_releases_its_reader() {
    let (transport, mut peer) = test_transport();
    let client = JsonRpcClient::new(transport);
    client.start().await.unwrap();
    drop(client);
    assert_eq!(promptly(peer.read(&mut [0; 1])).await.unwrap(), 0);
}

#[tokio::test]
async fn dropping_tcp_client_releases_its_reader() {
    let (stream, mut peer) = tcp_pair().await;
    let client = TcpJsonRpcClient::new(stream);
    client.start().await.unwrap();
    drop(client);
    assert_eq!(promptly(peer.read(&mut [0; 1])).await.unwrap(), 0);
}
