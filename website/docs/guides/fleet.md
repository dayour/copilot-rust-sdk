---
id: fleet
title: Fleet Management
sidebar_label: Fleet
---

# Fleet Management

A fleet is a set of agents working a task in parallel. The SDK lets you start a fleet from a session with an optional shared prompt.

## Starting a fleet

```rust
use copilot_sdk::FleetStartOptions;

session.start_fleet(Some(FleetStartOptions {
    prompt: Some("Build and test the project across all crates.".into()),
})).await?;
```

Pass `None` to start with defaults:

```rust
session.start_fleet(None).await?;
```

[`FleetStartOptions`](/docs/api/types#fleetstartoptions) currently exposes:

| Field | Purpose |
|-------|---------|
| `prompt` | An optional shared task prompt for the fleet. |

## Observing fleet activity

Fleet members typically surface as [custom-agent events](/docs/core-concepts/events#custom-agents) on the session stream:

```rust
use copilot_sdk::SessionEventData;

match &event.data {
    SessionEventData::CustomAgentStarted(a)   => println!("agent {} started", a.agent_name),
    SessionEventData::CustomAgentCompleted(a) => println!("agent {} completed", a.agent_name),
    SessionEventData::CustomAgentFailed(a)    => println!("agent {} failed: {}", a.agent_name, a.error),
    _ => {}
}
```

## When to use a fleet

Fleets fit naturally divisible work - independent modules, multiple test suites, parallel research threads. For a single sequential conversation, a normal [session](/docs/core-concepts/sessions) is the right tool.

## Related

- [Custom Agents](/docs/guides/agents)
- [Events](/docs/core-concepts/events#custom-agents)
