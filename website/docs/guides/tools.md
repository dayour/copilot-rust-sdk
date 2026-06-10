---
id: tools
title: Custom Tools
sidebar_label: Tools
---

# Custom Tools

Tools let the agent call back into your program to perform actions you define - fetch data, hit an API, compute something. You declare a tool's name and parameter schema, register it on a session, and provide an async handler that runs when the agent invokes it.

## Defining a tool

Use the fluent [`Tool`](/docs/api/types#tool) builder:

```rust
use copilot_sdk::Tool;

let tool = Tool::new("get_weather")
    .description("Get the current weather for a city")
    .parameter("city", "string", "City name", true)        // name, type, description, required
    .parameter("units", "string", "celsius or fahrenheit", false)
    .skip_permission(true);                                  // do not prompt for approval
```

Builder methods (see the [tools API reference](/docs/api/tools) and [`Tool` type](/docs/api/types#tool)):

| Method | Purpose |
|--------|---------|
| `Tool::new(name)` | Start a tool with the given name. |
| `.description(text)` | Human/agent-readable description. |
| `.parameter(name, type, description, required)` | Add one JSON-schema parameter. |
| `.schema(value)` | Set the entire parameter JSON schema at once. |
| `.typed_schema::<T>()` | Derive the schema from a Rust type (feature `schemars`). |
| `.overrides_built_in_tool(bool)` | Replace a built-in tool of the same name. |
| `.skip_permission(bool)` | Skip the approval prompt for this tool. |

### Deriving the schema from a type

With the [`schemars` feature](/docs/getting-started/installation#cargo-features) enabled you can derive the parameter schema from a struct:

```rust
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
struct WeatherArgs {
    /// City name
    city: String,
    /// celsius or fahrenheit
    units: Option<String>,
}

let tool = Tool::new("get_weather")
    .description("Get the current weather")
    .typed_schema::<WeatherArgs>();
```

## Writing a handler

A [`ToolHandler`](/docs/api/session#type-aliases) is an `Arc<dyn Fn(&str, &Value) -> ToolResultObject + Send + Sync>`. It receives the tool name and the JSON arguments and returns a [`ToolResultObject`](/docs/api/types#toolresultobject).

```rust
use std::sync::Arc;
use serde_json::Value;
use copilot_sdk::{ToolResultObject};

let handler: copilot_sdk::ToolHandler = Arc::new(|_name: &str, args: &Value| {
    let city = args.get("city").and_then(|v| v.as_str()).unwrap_or("unknown");
    ToolResultObject::text(format!("It is 21C and sunny in {city}."))
});
```

Build results with the helpers on [`ToolResultObject`](/docs/api/types#toolresultobject):

```rust
ToolResultObject::text("plain text result");
ToolResultObject::error("something went wrong");
```

For binary payloads, populate `binary_results_for_llm` with [`ToolBinaryResult`](/docs/api/types#toolbinaryresult) entries (base64 `data`, `mime_type`).

## Registering tools

Register a tool with its handler on the session:

```rust
session.register_tool_with_handler(tool, Some(handler)).await;
```

Other registration methods:

```rust
session.register_tool(tool).await;            // no handler (server-side or invoked manually)
session.register_tools(vec![a, b, c]).await;  // batch
```

You can also pass tools up front in [`SessionConfig.tools`](/docs/api/types#sessionconfig).

## Inspecting and invoking

```rust
let names: Vec<String> = session.get_tools().await.into_iter().map(|t| t.name).collect();
let one = session.get_tool("get_weather").await;            // Option<Tool>

// Invoke a registered tool's handler directly (useful in tests).
let result = session.invoke_tool("get_weather", &serde_json::json!({"city":"Paris"})).await?;
```

## Tool execution events

As the agent runs a tool you receive [tool events](/docs/core-concepts/events#tools): `ToolExecutionStart`, `ToolExecutionProgress`, `ToolExecutionPartialResult`, and `ToolExecutionComplete`.

```rust
use copilot_sdk::SessionEventData;

match &event.data {
    SessionEventData::ToolExecutionStart(t) => println!("start {}", t.tool_name),
    SessionEventData::ToolExecutionComplete(t) => println!("done, success={}", t.success),
    _ => {}
}
```

## The `define_tool` helper

If you prefer a function over the builder, [`define_tool`](/docs/api/tools) constructs a `Tool` from a name, description, and optional schema:

```rust
use copilot_sdk::define_tool;
use serde_json::json;

let tool = define_tool(
    "add",
    "Add two numbers",
    Some(json!({
        "type": "object",
        "properties": {
            "a": {"type": "number"},
            "b": {"type": "number"}
        },
        "required": ["a", "b"]
    })),
);
```

## Permissions

By default, tool calls can require approval. Use `.skip_permission(true)` per tool, a [permission handler](/docs/guides/permissions), or client-level [allow/deny lists](/docs/guides/permissions#client-level-allowdeny) to control this. See the `tool_usage`, `multi_tools`, `fluent_tools`, `tool_progress`, and `deny_allow_tools` entries in the [examples catalog](/docs/examples).

## Related

- [Permissions](/docs/guides/permissions)
- [Tools API reference](/docs/api/tools)
- [MCP Servers](/docs/guides/mcp) - tools provided by external servers.
