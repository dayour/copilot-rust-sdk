---
id: events
title: Events
sidebar_label: Events
---

# Events

Sessions communicate progress through a stream of typed events. This is the heart of the SDK: assistant output, tool execution, reasoning, usage, compaction, and lifecycle transitions all arrive as [`SessionEvent`](/docs/api/events#sessionevent) values.

## The event type

```rust
pub struct SessionEvent {
    pub id: String,
    pub timestamp: String,
    pub event_type: String,         // e.g. "assistant.message"
    pub parent_id: Option<String>,
    pub ephemeral: Option<bool>,
    pub data: SessionEventData,     // typed payload
}
```

The `data` field is a [`SessionEventData`](/docs/api/events#sessioneventdata) enum with one variant per event kind, plus an `Unknown(serde_json::Value)` fallback for forward compatibility with newer CLI versions.

## Subscribing

### Channel style

[`subscribe`](/docs/api/session#event-handling) returns an [`EventSubscription`](/docs/api/session#eventsubscription) backed by a Tokio broadcast receiver. Subscribe before sending so you do not miss early events.

```rust
use copilot_sdk::SessionEventData;

let mut events = session.subscribe();
session.send("Refactor this module.").await?;

while let Ok(event) = events.recv().await {
    match &event.data {
        SessionEventData::AssistantMessageDelta(d) => print!("{}", d.delta_content),
        SessionEventData::SessionIdle(_) => break,
        _ => {}
    }
}
```

### Callback style

[`on`](/docs/api/session#event-handling) registers a closure and returns an unsubscribe function:

```rust
let unsubscribe = session.on(|event| {
    if let SessionEventData::AssistantMessage(msg) = &event.data {
        println!("{}", msg.content);
    }
}).await;

// later
unsubscribe();
```

Because delivery uses a broadcast channel, many subscribers each receive every event.

## Event categories

The 40+ variants group naturally:

### Session lifecycle

`SessionStart`, `SessionResume`, `SessionInfo`, `SessionError`, `SessionIdle`, `SessionModelChange`, `SessionHandoff`, `SessionTruncation`, `SessionUsageInfo`, `SessionShutdown`, `SessionSnapshotRewind`.

### Assistant output

`AssistantTurnStart`, `AssistantIntent`, `AssistantReasoning`, `AssistantReasoningDelta`, `AssistantMessage`, `AssistantMessageDelta`, `AssistantTurnEnd`, `AssistantUsage`.

### Tools

`ToolUserRequested`, `ToolExecutionStart`, `ToolExecutionPartialResult`, `ToolExecutionProgress`, `ToolExecutionComplete`, `ExternalToolRequested`.

### Custom agents

`CustomAgentStarted`, `CustomAgentCompleted`, `CustomAgentFailed`, `CustomAgentSelected`.

### Hooks, system, and skills

`HookStart`, `HookEnd`, `SystemMessage`, `SkillInvoked`.

### Compaction and permissions

`SessionCompactionStart`, `SessionCompactionComplete`, `PermissionRequested`, `PendingMessagesModified`, `UserMessage`, `Abort`.

### Fallback

`Unknown(serde_json::Value)` carries any event the SDK does not yet model.

See the [events API reference](/docs/api/events) for the full payload of every variant.

## Streaming vs. final messages

When `streaming` is enabled on the session:

- `AssistantMessageDelta` events stream incremental text (`delta_content`).
- A final `AssistantMessage` event carries the complete `content`.

When streaming is disabled, you generally receive the consolidated `AssistantMessage` without deltas. Choose based on whether you want token-by-token UX or just the final answer.

## Helper methods

[`SessionEvent`](/docs/api/events#methods) provides predicates and typed accessors so you can avoid manual `match` in common cases:

```rust
if event.is_assistant_message() {
    if let Some(msg) = event.as_assistant_message() {
        println!("{}", msg.content);
    }
}

if event.is_terminal() {
    // idle or error: the turn is over
}

if let Some(text) = event.content() {
    // best-effort text content for message/delta events
}
```

| Predicate | Accessor |
|-----------|----------|
| `is_assistant_message()` | `as_assistant_message()` |
| `is_assistant_message_delta()` | `as_assistant_message_delta()` |
| `is_session_error()` | `as_session_error()` |
| `is_session_idle()` | - |
| `is_terminal()` | - |
| - | `as_tool_execution_complete()` |
| - | `content()` |

## Waiting helpers

If you do not want to drive the loop yourself, the session provides higher-level waits built on the event stream:

```rust
// Block until the session goes idle (optionally with a timeout).
session.wait_for_idle(Some(std::time::Duration::from_secs(60))).await?;

// Send and wait for the terminal event.
let last = session.send_and_wait("Summarize.", None).await?;

// Send and collect the full text reply as a String.
let text = session.send_and_collect("Summarize.", None).await?;
```

## Backpressure and lag

Broadcast receivers have bounded capacity. A subscriber that falls behind may observe a lag error from `recv`. Keep event handlers short - offload heavy work to separate tasks - so you keep up with the stream.

## Next

- [Events API reference](/docs/api/events)
- [Tools](/docs/guides/tools)
