// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

use copilot_sdk::{CopilotError, Session, SessionEvent};
use serde_json::{json, Value};
use std::sync::{Arc, Weak};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(1);

fn event(kind: &str, data: Value) -> SessionEvent {
    SessionEvent::from_json(&json!({
        "id": "event",
        "timestamp": "2026-01-01T00:00:00Z",
        "type": kind,
        "data": data,
    }))
    .unwrap()
}

fn message(id: &str, content: &str) -> SessionEvent {
    event(
        "assistant.message",
        json!({"messageId": id, "content": content}),
    )
}

fn delta(id: &str, content: &str) -> SessionEvent {
    event(
        "assistant.message_delta",
        json!({"messageId": id, "deltaContent": content}),
    )
}

fn idle() -> SessionEvent {
    event("session.idle", json!({}))
}

fn session() -> Session {
    Session::new("test-session".into(), None, |method, _| {
        assert_eq!(method, "session.send");
        Box::pin(async { Ok(json!({"messageId": "user-message"})) })
    })
}

async fn collect(events: Vec<SessionEvent>) -> copilot_sdk::Result<String> {
    let session = session();
    let response = session.send_and_collect("hello", Some(TIMEOUT));
    tokio::pin!(response);
    assert!(futures::poll!(&mut response).is_pending());
    for event in events {
        session.dispatch_event(event).await;
    }
    response.await
}

#[tokio::test]
async fn collects_events_dispatched_before_send_returns() {
    let session = Arc::new_cyclic(|session: &Weak<Session>| {
        let session = session.clone();
        Session::new("test-session".into(), None, move |method, _| {
            assert_eq!(method, "session.send");
            let session = session.upgrade().unwrap();
            Box::pin(async move {
                session.dispatch_event(message("a", "early response")).await;
                session.dispatch_event(idle()).await;
                Ok(json!({"messageId": "user-message"}))
            })
        })
    });

    assert_eq!(
        session
            .send_and_collect("hello", Some(TIMEOUT))
            .await
            .unwrap(),
        "early response"
    );
}

#[tokio::test]
async fn does_not_duplicate_deltas_and_full_message() {
    assert_eq!(
        collect(vec![
            delta("a", "Hello "),
            delta("a", "world"),
            message("a", "Hello world"),
            idle(),
        ])
        .await
        .unwrap(),
        "Hello world"
    );
}

#[tokio::test]
async fn collects_full_messages_without_deltas() {
    assert_eq!(
        collect(vec![message("a", "Hello "), message("b", "world"), idle()])
            .await
            .unwrap(),
        "Hello world"
    );
}

#[tokio::test]
async fn collects_deltas_without_full_message() {
    assert_eq!(
        collect(vec![delta("a", "Hello "), delta("a", "world"), idle()])
            .await
            .unwrap(),
        "Hello world"
    );
}

#[tokio::test]
async fn chooses_deltas_per_message_in_mixed_response() {
    assert_eq!(
        collect(vec![
            message("a", "First. "),
            delta("b", "Second. "),
            message("b", "Second. "),
            message("c", "Third."),
            idle(),
        ])
        .await
        .unwrap(),
        "First. Second. Third."
    );
}

#[tokio::test]
async fn empty_delta_still_selects_streamed_content() {
    assert_eq!(
        collect(vec![delta("a", ""), message("a", "not a delta"), idle()])
            .await
            .unwrap(),
        ""
    );
}

#[tokio::test]
async fn idle_without_content_returns_empty_string() {
    assert_eq!(collect(vec![idle()]).await.unwrap(), "");
}

#[tokio::test]
async fn collect_reports_lag_instead_of_partial_content() {
    let mut events = vec![delta("a", "x"); 1025];
    events.push(idle());
    let error = collect(events).await.unwrap_err();
    assert!(error.to_string().contains("2 session events"));
    assert!(matches!(error, CopilotError::EventsLagged(2)));
}

#[tokio::test]
async fn wait_for_idle_reports_lag() {
    let session = session();
    let response = session.wait_for_idle(Some(TIMEOUT));
    tokio::pin!(response);
    assert!(futures::poll!(&mut response).is_pending());
    for _ in 0..1025 {
        session.dispatch_event(message("a", "x")).await;
    }
    session.dispatch_event(idle()).await;
    let error = response.await.unwrap_err();
    assert!(error.to_string().contains("2 session events"));
    assert!(matches!(error, CopilotError::EventsLagged(2)));
}

#[tokio::test]
async fn collect_propagates_session_errors() {
    let error = collect(vec![event(
        "session.error",
        json!({"errorType": "test", "message": "response failed"}),
    )])
    .await
    .unwrap_err();
    assert!(
        matches!(error, CopilotError::Protocol(message) if message.contains("response failed"))
    );
}

#[tokio::test]
async fn collect_times_out_without_idle() {
    let error = session()
        .send_and_collect("hello", Some(Duration::ZERO))
        .await
        .unwrap_err();
    assert!(matches!(error, CopilotError::Timeout(Duration::ZERO)));
}

#[tokio::test]
async fn collect_propagates_send_errors() {
    let session = Session::new("test-session".into(), None, |_, _| {
        Box::pin(async { Err(CopilotError::NotConnected) })
    });
    let error = session
        .send_and_collect("hello", Some(TIMEOUT))
        .await
        .unwrap_err();
    assert!(matches!(error, CopilotError::NotConnected));
}
