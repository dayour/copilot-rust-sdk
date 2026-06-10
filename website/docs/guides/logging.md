---
id: logging
title: Session Logging
sidebar_label: Logging
---

# Session Logging

You can write log entries into a session's event history. This is useful for annotating a transcript with application-side context - milestones, decisions, or diagnostics - that travels with the conversation.

## Adding a log entry

Call [`log`](/docs/api/session#session-logging) with a message and optional [`LogOptions`](/docs/api/types#logoptions):

```rust
use copilot_sdk::{LogOptions, SessionLogLevel};

let result = session.log("Processing step complete", Some(LogOptions {
    level: Some(SessionLogLevel::Info),
    ephemeral: Some(false),
})).await?;

println!("logged event id: {}", result.event_id);
```

A bare message works too:

```rust
session.log("checkpoint reached", None).await?;
```

## Log levels

[`SessionLogLevel`](/docs/api/types#sessionloglevel) is one of:

| Level | Use |
|-------|-----|
| `Info` | Routine progress. |
| `Warning` | Recoverable concerns. |
| `Error` | Failures worth surfacing. |

## Ephemeral entries

Set `ephemeral: Some(true)` for transient notes that should not be persisted as part of the durable transcript - handy for noisy, moment-in-time diagnostics.

## Return value

[`log`](/docs/api/session#session-logging) returns a [`LogResult`](/docs/api/types#logresult) containing the `event_id` of the created entry, so you can correlate it with the [event stream](/docs/core-concepts/events).

## Session log vs. client log level

This is distinct from the client's [`LogLevel`](/docs/api/types#loglevel) (set via the builder), which controls CLI process log verbosity. Session logging writes entries into the conversation's own event history.

## Related

- [Events](/docs/core-concepts/events)
- [Client lifecycle](/docs/core-concepts/client-lifecycle) - the client-level log level.
