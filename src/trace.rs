// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! W3C Trace Context propagation helpers.
//!
//! Mirrors the Node.js `telemetry` module. The SDK does not depend on any
//! OpenTelemetry packages; instead, callers provide a [`TraceContextProvider`]
//! that returns the current [`TraceContext`], which the client injects into
//! `session.create`, `session.resume`, and `session.send` requests so the
//! Copilot CLI can continue a distributed trace.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

/// W3C Trace Context headers used for distributed trace propagation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceContext {
    /// The `traceparent` header value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub traceparent: Option<String>,
    /// The `tracestate` header value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracestate: Option<String>,
}

impl TraceContext {
    /// Returns `true` when neither `traceparent` nor `tracestate` is set.
    pub fn is_empty(&self) -> bool {
        self.traceparent.is_none() && self.tracestate.is_none()
    }
}

/// Boxed future returned by a [`TraceContextProvider`].
pub type TraceContextFuture = Pin<Box<dyn Future<Output = TraceContext> + Send>>;

/// Callback that returns the current W3C Trace Context. Wire this up to your
/// OpenTelemetry (or other tracing) SDK to enable distributed trace
/// propagation between your app and the Copilot CLI.
pub type TraceContextProvider = Arc<dyn Fn() -> TraceContextFuture + Send + Sync>;

/// Calls the user-provided [`TraceContextProvider`] to obtain the current W3C
/// Trace Context. Returns an empty [`TraceContext`] when no provider is
/// configured.
pub async fn get_trace_context(provider: Option<&TraceContextProvider>) -> TraceContext {
    match provider {
        None => TraceContext::default(),
        Some(provider) => provider().await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn no_provider_returns_empty() {
        let ctx = get_trace_context(None).await;
        assert!(ctx.is_empty());
    }

    #[tokio::test]
    async fn provider_result_is_returned() {
        let provider: TraceContextProvider = Arc::new(|| {
            Box::pin(async {
                TraceContext {
                    traceparent: Some("00-trace-span-01".to_string()),
                    tracestate: None,
                }
            })
        });
        let ctx = get_trace_context(Some(&provider)).await;
        assert_eq!(ctx.traceparent.as_deref(), Some("00-trace-span-01"));
        assert!(!ctx.is_empty());
    }

    #[test]
    fn empty_context_skips_serialization() {
        let json = serde_json::to_string(&TraceContext::default()).unwrap();
        assert_eq!(json, "{}");
    }
}
