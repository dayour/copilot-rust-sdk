---
id: shell
title: Shell Operations
sidebar_label: Shell
---

# Shell Operations

A session can execute shell commands in its working environment and terminate them. This is how an agent runs builds, tests, linters, or any CLI tool.

## Executing a command

Call [`shell_exec`](/docs/api/session#shell-operations) with [`ShellExecOptions`](/docs/api/types#shellexecoptions):

```rust
use copilot_sdk::ShellExecOptions;

let result = session.shell_exec(ShellExecOptions {
    command: "cargo test".into(),
    cwd: Some("/path/to/project".into()),
    env: None,
}).await?;

println!("started process: {}", result.process_id);
```

`ShellExecOptions` fields:

| Field | Type | Purpose |
|-------|------|---------|
| `command` | `String` | The command line to run. |
| `cwd` | `Option<String>` | Working directory (defaults to the session's). |
| `env` | `Option<HashMap<String, String>>` | Extra environment variables. |

The call returns a [`ShellExecResult`](/docs/api/types#shellexecresult) with a `process_id` you can use to kill the process later.

## Streaming output

Command output is delivered through the [event stream](/docs/core-concepts/events) as the process runs (tool execution events), rather than being returned synchronously. Subscribe to observe progress:

```rust
use copilot_sdk::SessionEventData;

let mut events = session.subscribe();
let result = session.shell_exec(ShellExecOptions {
    command: "npm run build".into(),
    cwd: None,
    env: None,
}).await?;

while let Ok(event) = events.recv().await {
    if let SessionEventData::ToolExecutionPartialResult(p) = &event.data {
        print!("{}", p.partial_output);
    }
    if event.is_terminal() { break; }
}
```

## Killing a process

Send a signal with [`shell_kill`](/docs/api/session#shell-operations) using a [`ShellSignal`](/docs/api/types#shellsignal):

```rust
use copilot_sdk::ShellSignal;

session.shell_kill(&result.process_id, ShellSignal::SIGTERM).await?;
```

| Signal | Use |
|--------|-----|
| `SIGINT` | Polite interrupt (Ctrl+C equivalent). |
| `SIGTERM` | Request graceful termination. |
| `SIGKILL` | Force kill immediately. |

## Safety

Shell execution is powerful. Combine it with [permissions](/docs/guides/permissions) and [hooks](/docs/guides/hooks) to constrain what may run, and avoid `allow_all_tools` when the agent can reach a shell in an untrusted context.

See the `shell_exec` entry in the [examples catalog](/docs/examples).

## Related

- [Permissions](/docs/guides/permissions)
- [Workspace](/docs/guides/workspace)
