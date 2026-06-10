---
id: hooks
title: Hooks
sidebar_label: Hooks
---

# Hooks

Hooks let you intercept the session at six well-defined lifecycle points. Unlike events (which are observational), hooks can **modify** behavior - alter tool arguments, inject context, change a prompt, or suppress output.

## The six hooks

Configure hooks with [`SessionHooks`](/docs/api/types#sessionhooks), passed via [`SessionConfig.hooks`](/docs/api/types#sessionconfig) or [`session.register_hooks`](/docs/api/session#hooks).

| Hook | Input | Output | Fires |
|------|-------|--------|-------|
| `on_pre_tool_use` | [`PreToolUseHookInput`](/docs/api/types) | [`PreToolUseHookOutput`](/docs/api/types) | Before a tool runs |
| `on_post_tool_use` | `PostToolUseHookInput` | `PostToolUseHookOutput` | After a tool runs |
| `on_user_prompt_submitted` | `UserPromptSubmittedHookInput` | `UserPromptSubmittedHookOutput` | When a prompt is submitted |
| `on_session_start` | `SessionStartHookInput` | `SessionStartHookOutput` | At session start |
| `on_session_end` | `SessionEndHookInput` | `SessionEndHookOutput` | At session end |
| `on_error_occurred` | `ErrorOccurredHookInput` | `ErrorOccurredHookOutput` | When an error occurs |

## Registering hooks

```rust
use std::sync::Arc;
use copilot_sdk::{SessionConfig, SessionHooks, PreToolUseHookOutput};

let config = SessionConfig {
    hooks: Some(SessionHooks {
        on_pre_tool_use: Some(Arc::new(|input| {
            println!("about to run tool: {}", input.tool_name);
            PreToolUseHookOutput::default()
        })),
        ..Default::default()
    }),
    ..Default::default()
};

let session = client.create_session(config).await?;
```

Or register after creation:

```rust
session.register_hooks(SessionHooks {
    on_post_tool_use: Some(Arc::new(|input| {
        println!("tool {} finished", input.tool_name);
        Default::default()
    })),
    ..Default::default()
}).await;
```

Check whether any hooks are registered:

```rust
if session.has_hooks().await {
    println!("hooks active");
}
```

## Modifying behavior

The power of hooks is in their outputs. For example, the pre-tool-use hook can rewrite arguments, inject context, or deny the call:

```rust
use copilot_sdk::PreToolUseHookOutput;

let pre = Arc::new(|input: &copilot_sdk::PreToolUseHookInput| {
    let mut out = PreToolUseHookOutput::default();

    // Add context the agent should consider.
    out.additional_context = Some("Prefer read-only operations.".into());

    // Block destructive tools.
    if input.tool_name == "delete_file" {
        out.permission_decision = Some("deny".into());
        out.permission_decision_reason = Some("deletes are disabled".into());
    }

    // Rewrite arguments if needed.
    // out.modified_args = Some(new_args);

    out
});
```

Common output fields across hooks:

- `additional_context` - extra text to inject for the agent.
- `suppress_output` - hide the default output.
- `modified_args` / `modified_result` / `modified_prompt` / `modified_config` - replace inputs/outputs.
- `permission_decision` / `permission_decision_reason` - gate tool use (pre-tool-use).
- `cleanup_actions` / `session_summary` - session-end housekeeping.
- `error_handling` / `retry_count` / `user_notification` - error-hook behavior.

See the API reference for the exact fields of each [hook input/output type](/docs/api/types#hooks).

## Hooks vs. permissions vs. events

| Mechanism | Can modify? | Best for |
|-----------|-------------|----------|
| [Events](/docs/core-concepts/events) | No | Observing progress |
| [Permissions](/docs/guides/permissions) | Approve/deny | Gating individual tool calls |
| Hooks | Yes (args/context/output) | Cross-cutting policy and transforms |

You also observe `HookStart`/`HookEnd` [events](/docs/core-concepts/events#hooks-system-and-skills) as hooks run.

See the `hooks` entry in the [examples catalog](/docs/examples).

## Related

- [Permissions](/docs/guides/permissions)
- [Types API reference](/docs/api/types#hooks)
