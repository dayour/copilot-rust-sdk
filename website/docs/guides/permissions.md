---
id: permissions
title: Permissions
sidebar_label: Permissions
---

# Permissions

When the agent wants to run a tool or take a sensitive action, it can request permission. You control approvals three ways: per-tool skipping, a session-level permission handler, and client-level allow/deny lists.

## Per-tool skipping

The simplest control: mark a trusted tool to skip prompts.

```rust
let tool = copilot_sdk::Tool::new("read_config")
    .description("Read the app config")
    .skip_permission(true);
```

## Session-level permission handler

Register a [`PermissionHandler`](/docs/api/session#type-aliases) to decide each request programmatically. It receives a [`PermissionRequest`](/docs/api/types#permissionrequest) and returns a [`PermissionRequestResult`](/docs/api/types#permissionrequestresult).

```rust
use copilot_sdk::PermissionRequestResult;

session.register_permission_handler(|req| {
    println!("permission requested: kind={}", req.kind);

    // Approve safe kinds, deny the rest.
    if req.kind == "read" {
        PermissionRequestResult::approved()
    } else {
        PermissionRequestResult::denied()
    }
}).await;
```

To route requests to your handler, enable it on the session config:

```rust
use copilot_sdk::SessionConfig;

let session = client.create_session(SessionConfig {
    request_permission: Some(true),
    ..Default::default()
}).await?;
```

`PermissionRequestResult` provides constructors and predicates:

```rust
PermissionRequestResult::approved();
PermissionRequestResult::denied();
let r = PermissionRequestResult::approved();
assert!(r.is_approved());
assert!(!r.is_denied());
```

The request's `extension_data` map carries kind-specific details (for example the tool name and arguments) so you can make fine-grained decisions.

## Client-level allow/deny

For coarse, declarative policy, configure allow/deny lists on the [client builder](/docs/core-concepts/client-lifecycle#building-a-client). These accept tool specifications.

```rust
let client = copilot_sdk::Client::builder()
    .allow_tool("read_file")          // auto-approve
    .allow_tools(["grep", "list_dir"])
    .deny_tool("delete_file")         // always block
    .deny_tools(["shell", "write_file"])
    .build()?;
```

To auto-approve everything (use only in trusted, sandboxed contexts):

```rust
let client = copilot_sdk::Client::builder()
    .allow_all_tools(true)
    .build()?;
```

This is the basis of the "YOLO" pattern shown in the `yolo` example.

## Precedence and strategy

A robust setup combines layers:

1. **Deny lists** block dangerous tools outright at the client.
2. **Allow lists** silently approve known-safe tools.
3. **A permission handler** adjudicates anything left, with full request context.
4. **`skip_permission`** marks individual trusted tools you register.

Prefer explicit allow/deny lists for irreversible actions over `allow_all_tools`.

## Permission events

You may also observe `PermissionRequested` on the [event stream](/docs/core-concepts/events#compaction-and-permissions) for visibility, independent of how the decision is made.

See the `permission_callback`, `deny_allow_tools`, and `yolo` entries in the [examples catalog](/docs/examples).

## Related

- [Tools](/docs/guides/tools)
- [User Input](/docs/guides/user-input)
- [Hooks](/docs/guides/hooks) - the pre-tool-use hook can also gate tool calls.
