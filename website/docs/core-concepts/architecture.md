---
id: architecture
title: Architecture
sidebar_label: Architecture
---

# Architecture

The SDK is a thin, typed, asynchronous layer over the Copilot CLI's JSON-RPC interface. Understanding the layers makes the rest of the API obvious.

## Layered overview

```text
┌─────────────────────────────────────────────────────────────┐
│ Your application                                             │
├─────────────────────────────────────────────────────────────┤
│ Client / Session            (high-level, typed async API)    │
├─────────────────────────────────────────────────────────────┤
│ JsonRpcClient               (requests, notifications, ids)   │
├─────────────────────────────────────────────────────────────┤
│ MessageFramer               (Content-Length framing)         │
├─────────────────────────────────────────────────────────────┤
│ Transport (StdioTransport / TCP)   (raw byte I/O)            │
├─────────────────────────────────────────────────────────────┤
│ CopilotProcess              (spawns and supervises the CLI)  │
├─────────────────────────────────────────────────────────────┤
│ GitHub Copilot CLI          (the agent runtime)              │
└─────────────────────────────────────────────────────────────┘
```

## Module map

The crate is organized into focused modules, all re-exported at the crate root for convenience.

| Module | Responsibility | Key types |
|--------|----------------|-----------|
| [`client`](/docs/api/client) | Connection + session management, server RPCs | `Client`, `ClientBuilder`, `LifecycleHandler` |
| [`session`](/docs/api/session) | One conversation: messaging, tools, events | `Session`, `EventSubscription`, handler aliases |
| [`types`](/docs/api/types) | Config, request/response, and value types | `SessionConfig`, `Tool`, `ModelInfo`, ... |
| [`events`](/docs/api/events) | The typed streaming event model | `SessionEvent`, `SessionEventData`, ... |
| [`tools`](/docs/api/tools) | Tool definition helpers | `define_tool`, `normalize_result` |
| [`transport`](/docs/api/transport) | Raw byte I/O + framing | `Transport`, `StdioTransport`, `MessageFramer` |
| [`jsonrpc`](/docs/api/jsonrpc) | JSON-RPC 2.0 client | `JsonRpcClient`, `JsonRpcRequest`, `JsonRpcResponse` |
| [`process`](/docs/api/process) | CLI discovery + process supervision | `CopilotProcess`, `find_copilot_cli` |
| [`error`](/docs/api/error) | Error type and `Result` alias | `CopilotError`, `Result` |

## Data flow

### Outbound (you call a method)

1. A `Client` or `Session` method builds a [`JsonRpcRequest`](/docs/api/jsonrpc#jsonrpcrequest) (`jsonrpc: "2.0"`) with a method name and JSON params.
2. The request is serialized, then framed by [`MessageFramer`](/docs/api/transport#messageframer) with a `Content-Length` header.
3. The bytes are written to the [transport](/docs/api/transport) (the CLI's stdin, or a TCP socket).
4. If the request expects a reply, its `id` is registered in a pending-requests table and the call awaits a one-shot channel.

### Inbound (the CLI sends data)

A background read loop continuously:

1. Reads one `Content-Length`-framed message and parses the JSON.
2. Dispatches by shape:
   - **Response** (has `id` + `result`/`error`) - completes the matching pending request.
   - **Notification** (no `id`) - invokes the registered notification handler. Session events arrive this way.
   - **Request** (has `id` + `method`) - invokes the request handler and sends back a [`JsonRpcResponse`](/docs/api/jsonrpc#jsonrpcresponse). Permission prompts, tool invocations, hooks, and user-input requests arrive this way.

### From notifications to typed events

Session event notifications are parsed from a [`RawSessionEvent`](/docs/api/events) into a strongly-typed [`SessionEvent`](/docs/api/events#sessionevent) whose `data` field is a [`SessionEventData`](/docs/api/events#sessioneventdata) enum. Each session owns a [Tokio broadcast channel](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html); parsed events are fanned out to every subscriber created with [`subscribe`](/docs/api/session#event-handling) or [`on`](/docs/api/session#event-handling).

## Server-initiated requests

Unlike a one-way client, the CLI can call back into your program. The SDK turns these into Rust handlers you register on a `Session`:

| Server request | Handler | Guide |
|----------------|---------|-------|
| Tool invocation | `register_tool_with_handler` | [Tools](/docs/guides/tools) |
| Permission prompt | `register_permission_handler` | [Permissions](/docs/guides/permissions) |
| User input question | `register_user_input_handler` | [User Input](/docs/guides/user-input) |
| Lifecycle hook | `register_hooks` | [Hooks](/docs/guides/hooks) |

This bidirectional model is why the [`jsonrpc`](/docs/api/jsonrpc) layer supports both a notification handler and a request handler.

## Concurrency model

- Everything is async and runs on a [Tokio](https://tokio.rs) runtime that you provide (typically via `#[tokio::main]`).
- `Client` and `Session` are `Send + Sync`; sessions are handed out as `Arc<Session>` so they can be shared across tasks.
- Event delivery uses a broadcast channel, so multiple consumers each see every event. Slow consumers can lag; design handlers to be quick or offload work.

## Safety

The crate sets `#![forbid(unsafe_code)]` - there is no `unsafe` anywhere in the library. All process and I/O handling is done through Tokio's safe APIs.

## Next

- [Client lifecycle](/docs/core-concepts/client-lifecycle)
- [Sessions](/docs/core-concepts/sessions)
- [Transport and protocol](/docs/core-concepts/transport-and-protocol)
