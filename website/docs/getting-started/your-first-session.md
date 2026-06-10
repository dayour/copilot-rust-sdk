---
id: your-first-session
title: Your First Session
sidebar_label: Your First Session
---

# Your First Session

This walkthrough explains the full lifecycle behind the quick-start snippets: building the client, starting it, creating a session, exchanging messages, consuming events, and shutting down.

## The lifecycle at a glance

```text
ClientBuilder ─▶ Client ─▶ start() ─▶ create_session() ─▶ send()/subscribe()
                                                              │
                                                              ▼
                                                     SessionEvent stream
                                                              │
                                              destroy() / stop()  ◀─┘
```

1. Configure a [`Client`](/docs/api/client) with [`ClientBuilder`](/docs/api/client#clientbuilder).
2. Call [`start`](/docs/api/client) to spawn/connect the CLI and negotiate the protocol.
3. Create one or more [`Session`](/docs/api/session) objects.
4. Send messages and consume the [event stream](/docs/core-concepts/events).
5. Call [`stop`](/docs/api/client) to shut everything down gracefully.

## Step 1: Build the client

The builder is the idiomatic way to configure a client. Every setter returns `Self`, so calls chain.

```rust
use copilot_sdk::{Client, LogLevel};

let client = Client::builder()
    .log_level(LogLevel::Info)
    .cwd(std::env::current_dir()?)
    .build()?;
```

If you only need defaults, `Client::builder().build()?` is enough. See the [Client lifecycle](/docs/core-concepts/client-lifecycle) for every option.

## Step 2: Start the connection

```rust
client.start().await?;
```

`start` spawns the CLI process (in stdio mode), wires up the [transport and JSON-RPC layer](/docs/core-concepts/transport-and-protocol), and negotiates the [protocol version](/docs/core-concepts/transport-and-protocol#protocol-versioning). After this returns, you can query:

```rust
let status = client.get_status().await?;     // CLI version + protocol version
let auth = client.get_auth_status().await?;  // authentication state
let models = client.list_models().await?;    // available models
```

## Step 3: Create a session

```rust
use copilot_sdk::SessionConfig;

let session = client.create_session(SessionConfig {
    streaming: true,
    client_name: Some("first-session".into()),
    ..Default::default()
}).await?;

println!("session id: {}", session.session_id());
```

A [`Session`](/docs/api/session) is returned as an `Arc<Session>`, so it is cheap to clone and share across tasks.

## Step 4: Send a message

There are two ways to send: fire-and-forget with [`send`](/docs/api/session#messaging), or one of the convenience wait helpers.

```rust
// Fire-and-forget: returns the message id immediately; consume events for the reply.
let message_id = session.send("Explain ownership in Rust.").await?;
```

```rust
// Convenience: send and wait for the full text reply.
let reply = session.send_and_collect("Explain ownership in Rust.", None).await?;
```

`send` accepts anything that converts into [`MessageOptions`](/docs/api/types#messageoptions), so a plain `&str` works, and so does a richer struct with attachments and a per-message mode.

## Step 5: Consume events

Every session multiplexes a broadcast channel of [`SessionEvent`](/docs/api/events#sessionevent). Subscribe before sending to avoid missing early events.

```rust
use copilot_sdk::SessionEventData;

let mut events = session.subscribe();
session.send("Explain ownership in Rust.").await?;

while let Ok(event) = events.recv().await {
    match &event.data {
        SessionEventData::AssistantMessage(msg) => {
            println!("\n[assistant] {}", msg.content);
        }
        SessionEventData::AssistantMessageDelta(delta) => {
            print!("{}", delta.delta_content);
        }
        SessionEventData::ToolExecutionStart(t) => {
            println!("[tool] {} starting", t.tool_name);
        }
        SessionEventData::SessionError(e) => {
            eprintln!("[error] {}", e.message);
            break;
        }
        SessionEventData::SessionIdle(_) => {
            // The agent finished its turn.
            break;
        }
        _ => {}
    }
}
```

:::tip Callback style
Prefer callbacks? Use [`session.on(...)`](/docs/api/session#event-handling) to register a handler closure and get back an unsubscribe function, instead of polling a receiver.
:::

### Knowing when a turn is done

The agent signals the end of a turn with `SessionIdle`. The helper [`SessionEvent::is_terminal`](/docs/api/events#methods) returns `true` for idle and error events, which is handy for loops:

```rust
while let Ok(event) = events.recv().await {
    // ... handle event ...
    if event.is_terminal() {
        break;
    }
}
```

Or skip the manual loop entirely with [`wait_for_idle`](/docs/api/session#wait-helpers):

```rust
session.send("Do the thing.").await?;
session.wait_for_idle(Some(std::time::Duration::from_secs(120))).await?;
```

## Step 6: Shut down

```rust
// Destroy a single session when you are done with it (optional).
session.destroy().await?;

// Stop the client; collects any non-fatal shutdown errors.
let errors = client.stop().await;
for e in errors {
    eprintln!("stop warning: {}", e.message);
}
```

`stop` returns a `Vec<StopError>` rather than failing, so you can log shutdown hiccups without aborting your program. Use [`force_stop`](/docs/api/client) if you need an immediate, non-graceful teardown.

## Putting it together

```rust
use copilot_sdk::{Client, SessionConfig, SessionEventData};

#[tokio::main]
async fn main() -> copilot_sdk::Result<()> {
    let client = Client::builder().build()?;
    client.start().await?;

    let session = client.create_session(SessionConfig {
        streaming: true,
        ..Default::default()
    }).await?;

    let mut events = session.subscribe();
    session.send("Give me three uses for Rust.").await?;

    while let Ok(event) = events.recv().await {
        match &event.data {
            SessionEventData::AssistantMessageDelta(d) => print!("{}", d.delta_content),
            _ if event.is_terminal() => break,
            _ => {}
        }
    }

    client.stop().await;
    Ok(())
}
```

## Where to go next

- [Architecture](/docs/core-concepts/architecture) - how the pieces fit together.
- [Events](/docs/core-concepts/events) - the full event model.
- [Messaging](/docs/guides/messaging) - attachments, modes, and wait helpers.
