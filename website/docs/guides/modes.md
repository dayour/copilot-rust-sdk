---
id: modes
title: Modes
sidebar_label: Modes
---

# Modes

A session runs in one of three modes, represented by the [`SessionMode`](/docs/api/types#sessionmode) enum. The mode controls how autonomously the agent acts.

| Mode | Behavior |
|------|----------|
| `Interactive` | The agent collaborates turn by turn, asking before taking impactful actions. |
| `Plan` | The agent focuses on producing and refining a plan rather than executing. |
| `Autopilot` | The agent acts autonomously, executing steps with minimal prompting. |

## Reading the current mode

```rust
let mode = session.get_mode().await?;
println!("mode: {mode:?}");
```

## Switching modes

```rust
use copilot_sdk::SessionMode;

session.set_mode(SessionMode::Plan).await?;
// ... agent drafts a plan ...
session.set_mode(SessionMode::Autopilot).await?;
// ... agent executes ...
session.set_mode(SessionMode::Interactive).await?;
```

## Per-message mode

[`MessageOptions`](/docs/api/types#messageoptions) has an optional `mode` string, letting you hint a mode for a single message without changing the session's mode:

```rust
use copilot_sdk::MessageOptions;

session.send(MessageOptions {
    prompt: "Draft an approach first.".into(),
    attachments: None,
    mode: Some("plan".into()),
}).await?;
```

## Combining modes with plans

`Plan` mode pairs naturally with [plan management](/docs/guides/plans): switch to `Plan`, let the agent populate the plan, review it with `read_plan`, then switch to `Autopilot` to execute. See the `mode_switching` and `plan_ops` entries in the [examples catalog](/docs/examples).

## Related

- [Plans](/docs/guides/plans)
- [Permissions](/docs/guides/permissions) - control what autopilot may do.
