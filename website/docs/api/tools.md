---
id: tools
title: "API: tools"
sidebar_label: tools
---

# Module `tools`

Helpers for defining tools and normalizing tool results. The [`Tool`](/docs/api/types#tool) and [`ToolResultObject`](/docs/api/types#toolresultobject) types themselves live in [`types`](/docs/api/types); this module provides functions. `define_tool` is re-exported at the crate root.

## `define_tool`

```rust
pub fn define_tool(
    name: &str,
    description: &str,
    parameters_schema: Option<serde_json::Value>,
) -> Tool;
```

Constructs a [`Tool`](/docs/api/types#tool) from a name, description, and optional JSON-schema parameters. A function-style alternative to the [`Tool`](/docs/api/types#tool) builder.

```rust
use copilot_sdk::define_tool;
use serde_json::json;

let tool = define_tool(
    "add",
    "Add two numbers",
    Some(json!({
        "type": "object",
        "properties": { "a": {"type": "number"}, "b": {"type": "number"} },
        "required": ["a", "b"]
    })),
);
```

## `normalize_result`

```rust
pub fn normalize_result(result: serde_json::Value) -> ToolResultObject;
```

Normalizes an arbitrary JSON value returned from a tool handler into a structured [`ToolResultObject`](/docs/api/types#toolresultobject). Useful when bridging loosely-typed handlers into the SDK's tool-result model.

:::note
There is no `define_tool!` macro - `define_tool` is a plain function. For fluent construction, prefer the [`Tool`](/docs/api/types#tool) builder.
:::

## Related

- [Tools guide](/docs/guides/tools)
- [`Tool` and result types](/docs/api/types#tools)
