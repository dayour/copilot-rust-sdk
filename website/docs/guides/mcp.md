---
id: mcp
title: MCP Servers
sidebar_label: MCP Servers
---

# MCP Servers

The [Model Context Protocol](https://modelcontextprotocol.io) (MCP) lets the agent use tools and resources provided by external servers. The SDK supports both **local** (stdio) and **remote** (HTTP/SSE) MCP servers, configured per session.

## Local (stdio) servers

A local server is a process the runtime launches and talks to over stdio. Configure it with [`McpLocalServerConfig`](/docs/api/types#mcplocalserverconfig):

```rust
use copilot_sdk::McpLocalServerConfig;

let fs_server = McpLocalServerConfig {
    command: "npx".into(),
    args: vec!["-y".into(), "@modelcontextprotocol/server-filesystem".into(), "/data".into()],
    tools: vec![],                 // empty = allow all advertised tools
    server_type: Some("stdio".into()),
    timeout: Some(30_000),
    env: None,
    cwd: None,
};
```

| Field | Purpose |
|-------|---------|
| `command` | Executable to launch. |
| `args` | Command arguments. |
| `tools` | Allowlist of tool names (empty = all). |
| `server_type` | Server type hint (for example `stdio`). |
| `timeout` | Timeout in milliseconds. |
| `env` | Environment variables for the process. |
| `cwd` | Working directory. |

## Remote (HTTP/SSE) servers

A remote server is reached over the network. Configure it with [`McpRemoteServerConfig`](/docs/api/types#mcpremoteserverconfig):

```rust
use copilot_sdk::McpRemoteServerConfig;

let remote = McpRemoteServerConfig {
    url: "https://mcp.example.com/sse".into(),
    server_type: "sse".into(),
    tools: vec![],
    timeout: Some(30_000),
    headers: None,                 // e.g. auth headers
};
```

| Field | Purpose |
|-------|---------|
| `url` | Server endpoint. |
| `server_type` | Transport type (for example `sse` or `http`). |
| `tools` | Allowlist of tool names. |
| `timeout` | Timeout in milliseconds. |
| `headers` | Optional HTTP headers (for example authorization). |

## Attaching servers to a session

[`McpServerConfig`](/docs/api/types#mcpserverconfig) is an enum wrapping either variant:

```rust
use copilot_sdk::{McpServerConfig, McpLocalServerConfig};

let _local = McpServerConfig::Local(McpLocalServerConfig {
    command: "my-mcp-server".into(),
    args: vec![],
    tools: vec![],
    server_type: Some("stdio".into()),
    timeout: None,
    env: None,
    cwd: None,
});
```

The session config accepts MCP servers as a map. Provide them via [`SessionConfig.mcp_servers`](/docs/api/types#sessionconfig) (a `HashMap<String, serde_json::Value>` keyed by server name), serializing your `McpServerConfig` values:

```rust
use std::collections::HashMap;
use copilot_sdk::SessionConfig;

let mut servers = HashMap::new();
servers.insert("filesystem".to_string(), serde_json::to_value(&_local)?);

let session = client.create_session(SessionConfig {
    mcp_servers: Some(servers),
    ..Default::default()
}).await?;
```

## Tools from MCP servers

Once connected, the server's tools appear to the agent like any other tool. When the agent runs one, the [`ToolExecutionComplete`](/docs/core-concepts/events#tools) event includes `mcp_server_name` and `mcp_tool_name` so you can attribute the call:

```rust
use copilot_sdk::SessionEventData;

if let SessionEventData::ToolExecutionComplete(t) = &event.data {
    if let (Some(server), Some(tool)) = (&t.mcp_server_name, &t.mcp_tool_name) {
        println!("MCP tool {tool} on {server} -> success={}", t.success);
    }
}
```

## Scoping servers to an agent

[Custom agents](/docs/guides/agents) can carry their own `mcp_servers`, scoping a server to a single persona rather than the whole session.

See the `mcp_servers` entry in the [examples catalog](/docs/examples).

## Related

- [Tools](/docs/guides/tools)
- [Custom Agents](/docs/guides/agents)
- [Types API reference](/docs/api/types#mcpserverconfig)
