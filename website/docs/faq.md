---
id: faq
title: FAQ and Troubleshooting
sidebar_label: FAQ
---

# FAQ and Troubleshooting

## The crate is `copilot-sdk` but examples import `copilot_sdk` - why?

Cargo crate names use hyphens; Rust identifiers use underscores. The crate `copilot-sdk` is imported as `copilot_sdk`. This is standard Rust behavior.

## I get `ProcessStart` or "CLI not found" on startup

The SDK could not locate or launch the Copilot CLI. Fix one of:

- Ensure `copilot` is on your `PATH`.
- Set `COPILOT_CLI_PATH` to the executable/script.
- Pass an explicit path with `Client::builder().cli_path(...)`.

See [Requirements](/docs/getting-started/requirements#cli-discovery).

## I get `NotConnected` when calling a method

You called an RPC before the connection was ready. Call `client.start().await?` first and await it. See [Client lifecycle](/docs/core-concepts/client-lifecycle#starting).

## I get `ProtocolMismatch`

The CLI and SDK do not share a supported protocol version. The SDK supports protocol version 3 (minimum 2). Update the Copilot CLI (or the SDK) so their ranges overlap. See [Transport and protocol](/docs/core-concepts/transport-and-protocol#protocol-versioning).

## My event loop never exits

Break on a terminal event. The agent signals end-of-turn with `SessionIdle`; errors arrive as `SessionError`. Use the helper:

```rust
if event.is_terminal() {
    break;
}
```

Or use [`wait_for_idle`](/docs/api/session#wait-helpers) instead of a manual loop. See [Events](/docs/core-concepts/events).

## I am missing the first events after sending

Subscribe **before** you send. `subscribe()` starts delivering from the moment it is called, so create the subscription first, then call `send`. See [Your First Session](/docs/getting-started/your-first-session#step-5-consume-events).

## `recv` returns a lag/closed error

Broadcast receivers are bounded. A slow consumer can lag behind and observe a `RecvError`. Keep handlers short and offload heavy work to separate tasks. See [Events](/docs/core-concepts/events#backpressure-and-lag).

## How do I stream tokens instead of waiting for the whole reply?

Set `streaming: true` on [`SessionConfig`](/docs/api/types#sessionconfig) and handle `AssistantMessageDelta` events. See [Messaging](/docs/guides/messaging#choosing-streaming-vs-non-streaming).

## How do I stop a response in progress?

Call `session.abort().await?`. The session stays usable for the next message. See [Messaging](/docs/guides/messaging#aborting).

## How do I use my own model/provider?

Use [BYOK](/docs/guides/byok): set a `ProviderConfig` on the session and, optionally, register `on_list_models` on the client for a custom model list.

## How do I auto-approve tools for a trusted, sandboxed run?

Use `Client::builder().allow_all_tools(true)`, or per-tool `.skip_permission(true)`. Prefer explicit allow/deny lists for irreversible actions. See [Permissions](/docs/guides/permissions).

## How do I see the raw JSON-RPC traffic?

Raise the log level (`LogLevel::Debug`) and look at the `debug_pipe` example. The wire format is `Content-Length`-framed JSON-RPC 2.0. See [Transport and protocol](/docs/core-concepts/transport-and-protocol#debugging-the-wire).

## How do I enable verbose SDK logging?

The SDK uses [`tracing`](https://docs.rs/tracing). Initialize a subscriber in your app and set, for example, `RUST_LOG=copilot_sdk=debug`.

## Does the SDK bundle the Copilot CLI?

No. CLI bundling is planned but not yet available; install and authenticate the CLI yourself. See the [feature parity table](/docs/intro#feature-parity).

## Which Tokio features do I need?

At minimum `rt-multi-thread` and `macros`. See [Installation](/docs/getting-started/installation#required-tokio-features).

## Still stuck?

Open an issue on [GitHub](https://github.com/dayour/copilot-rust-sdk) with the command you ran, the error, and your platform.
