---
id: infinite-sessions
title: Infinite Sessions
sidebar_label: Infinite Sessions
---

# Infinite Sessions

Long conversations eventually exceed a model's context window. Infinite sessions automatically compact older context so the conversation can continue indefinitely, and let you trigger compaction manually.

## Enabling

Enable infinite sessions through [`InfiniteSessionConfig`](/docs/api/types#infinitesessionconfig) on the session config:

```rust
use copilot_sdk::{SessionConfig, InfiniteSessionConfig};

let session = client.create_session(SessionConfig {
    infinite_sessions: Some(InfiniteSessionConfig::enabled()),
    ..Default::default()
}).await?;
```

### Tuning thresholds

`InfiniteSessionConfig` exposes thresholds (as fractions of the context window) that control when background compaction kicks in:

```rust
let cfg = InfiniteSessionConfig::with_thresholds(
    0.8,  // background_compaction_threshold
    0.95, // buffer_exhaustion_threshold
);
```

| Field | Meaning |
|-------|---------|
| `enabled` | Turn the feature on. |
| `background_compaction_threshold` | Start compacting in the background at this fill level. |
| `buffer_exhaustion_threshold` | Hard limit before forced compaction. |

## Manual compaction

Trigger compaction yourself at a natural breakpoint (for example, after finishing a subtask):

```rust
session.compact().await?;
```

## Observing compaction

Compaction emits [events](/docs/core-concepts/events#compaction-and-permissions):

- `SessionCompactionStart` - compaction began.
- `SessionCompactionComplete` - includes pre/post token counts, messages removed, tokens used, and an optional summary.

```rust
use copilot_sdk::SessionEventData;

match &event.data {
    SessionEventData::SessionCompactionStart(_) => println!("compacting..."),
    SessionEventData::SessionCompactionComplete(c) => {
        println!("compacted: success={}, pre={:?}, post={:?}",
            c.success, c.pre_compaction_tokens, c.post_compaction_tokens);
    }
    SessionEventData::SessionUsageInfo(u) => {
        println!("tokens {}/{}", u.current_tokens, u.token_limit);
    }
    _ => {}
}
```

You may also see `SessionTruncation` events when the runtime trims messages, and `SessionUsageInfo` updates with current token usage.

## Workspace and infinite sessions

Infinite sessions have an associated workspace path (`session.workspace_path()`), where the agent can persist artifacts like a `plan.md` across compactions. See [Workspace](/docs/guides/workspace).

See the `compaction_events` entry in the [examples catalog](/docs/examples).

## Related

- [Workspace](/docs/guides/workspace)
- [Events](/docs/core-concepts/events)
