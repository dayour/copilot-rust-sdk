---
id: messaging
title: Messaging
sidebar_label: Messaging
---

# Messaging

Sending prompts and receiving responses is the core interaction. The [`Session`](/docs/api/session) offers a fire-and-forget primitive plus several convenience wrappers.

## Sending a message

[`send`](/docs/api/session#messaging) accepts anything that converts into [`MessageOptions`](/docs/api/types#messageoptions). A plain string is the simplest form and returns the new message id:

```rust
let message_id = session.send("Explain borrow checking.").await?;
```

The reply arrives asynchronously through the [event stream](/docs/core-concepts/events). Subscribe before sending:

```rust
use copilot_sdk::SessionEventData;

let mut events = session.subscribe();
session.send("Explain borrow checking.").await?;

while let Ok(event) = events.recv().await {
    match &event.data {
        SessionEventData::AssistantMessageDelta(d) => print!("{}", d.delta_content),
        _ if event.is_terminal() => break,
        _ => {}
    }
}
```

## Convenience wait helpers

If you do not want to drive the event loop yourself:

| Method | Returns | Use when |
|--------|---------|----------|
| `send_and_collect(msg, timeout)` | `String` (full text) | You just want the answer text. |
| `send_and_wait(msg, timeout)` | `Option<SessionEvent>` (terminal event) | You want the final event object. |
| `wait_for_idle(timeout)` | `Option<SessionEvent>` | You already sent and want to block until idle. |

```rust
use std::time::Duration;

let text = session.send_and_collect("Summarize this repo.", Some(Duration::from_secs(120))).await?;
println!("{text}");
```

Pass `None` for no explicit timeout.

## Rich messages with `MessageOptions`

For attachments or a per-message mode, build [`MessageOptions`](/docs/api/types#messageoptions) explicitly:

```rust
use copilot_sdk::{MessageOptions, UserMessageAttachment, AttachmentType};

let options = MessageOptions {
    prompt: "Review the attached file.".into(),
    attachments: Some(vec![UserMessageAttachment {
        attachment_type: AttachmentType::File,
        path: "src/main.rs".into(),
        display_name: "main.rs".into(),
    }]),
    mode: None,
};

session.send(options).await?;
```

See [Attachments](/docs/guides/attachments) for files, directories, and text selections.

## Aborting

Stop the current turn without destroying the session:

```rust
session.abort().await?;
```

This is useful for "stop generating" buttons. The session remains usable for the next message.

## Reading history

Retrieve the recorded messages/events for the session:

```rust
let history = session.get_messages().await?; // Vec<SessionEvent>
```

## Choosing streaming vs. non-streaming

Set `streaming` on [`SessionConfig`](/docs/api/types#sessionconfig):

- **`streaming: true`** - receive `AssistantMessageDelta` events for token-by-token UX, then a final `AssistantMessage`.
- **`streaming: false`** - receive the consolidated `AssistantMessage` only.

## Related

- [Events](/docs/core-concepts/events)
- [Attachments](/docs/guides/attachments)
- [Modes](/docs/guides/modes)
