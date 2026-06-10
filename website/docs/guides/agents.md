---
id: agents
title: Custom Agents
sidebar_label: Agents
---

# Custom Agents

Custom agents are named personas with their own prompt, tool allowlist, and optional MCP servers. You can define them on a session and select which one is active.

## Defining agents at creation

Provide [`CustomAgentConfig`](/docs/api/types#customagentconfig) values in [`SessionConfig`](/docs/api/types#sessionconfig):

```rust
use copilot_sdk::{SessionConfig, CustomAgentConfig};

let session = client.create_session(SessionConfig {
    custom_agents: Some(vec![
        CustomAgentConfig {
            name: "code-reviewer".into(),
            prompt: "You are a meticulous senior code reviewer.".into(),
            display_name: Some("Code Reviewer".into()),
            description: Some("Reviews diffs for bugs and style.".into()),
            tools: Some(vec!["read_file".into(), "grep".into()]),
            mcp_servers: None,
            infer: None,
        },
    ]),
    ..Default::default()
}).await?;
```

| Field | Purpose |
|-------|---------|
| `name` | Stable identifier used to select the agent. |
| `prompt` | The agent's system prompt/persona. |
| `display_name` | Human-friendly label. |
| `description` | Short description of the agent's purpose. |
| `tools` | Optional allowlist of tool names. |
| `mcp_servers` | Optional MCP servers scoped to this agent. |
| `infer` | Whether the agent may be auto-selected. |

## Listing available agents

```rust
let agents = session.list_agents().await?;
for a in &agents {
    println!("{} - {}", a.name, a.description.clone().unwrap_or_default());
}
```

Each [`AgentInfo`](/docs/api/types#agentinfo) carries `name`, optional `display_name`, and optional `description`.

## Selecting and deselecting

```rust
// Activate an agent by name.
session.select_agent("code-reviewer").await?;

// Inspect the active agent.
if let Some(current) = session.get_current_agent().await? {
    println!("active: {}", current.name);
}

// Return to the default (no custom agent).
session.deselect_agent().await?;
```

## Observing agent events

When agents run, you receive [custom-agent events](/docs/core-concepts/events#custom-agents) on the stream:

- `CustomAgentSelected` - an agent became active (includes its tool list).
- `CustomAgentStarted` / `CustomAgentCompleted` / `CustomAgentFailed` - lifecycle of an agent invocation.

```rust
use copilot_sdk::SessionEventData;

if let SessionEventData::CustomAgentSelected(sel) = &event.data {
    println!("selected {} with tools {:?}", sel.agent_name, sel.tools);
}
```

See the `custom_agents` and `agent_management` entries in the [examples catalog](/docs/examples).

## Related

- [Tools](/docs/guides/tools)
- [MCP Servers](/docs/guides/mcp)
- [Fleet](/docs/guides/fleet) - run multiple agents in parallel.
