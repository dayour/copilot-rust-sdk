---
id: process
title: "API: process"
sidebar_label: process
---

# Module `process`

CLI discovery and child-process supervision. Re-exported at the crate root as `copilot_sdk::{find_copilot_cli, find_executable, find_node, is_node_script, CopilotProcess, ProcessOptions}`.

## Discovery functions

```rust
pub fn find_executable(name: &str) -> Option<PathBuf>;
pub fn find_node() -> Option<PathBuf>;
pub fn is_node_script(path: &Path) -> bool;       // true for .js / .mjs
pub fn find_copilot_cli() -> Option<PathBuf>;
```

`find_copilot_cli` resolves the CLI in this order:

1. `COPILOT_CLI_PATH` environment variable, if set and the file exists.
2. `copilot` on `PATH`.
3. Windows only: `copilot.cmd`, then `copilot.exe`.

See [Requirements](/docs/getting-started/requirements#cli-discovery).

## `ProcessOptions`

Options for spawning a subprocess.

```rust
pub struct ProcessOptions {
    pub working_directory: Option<PathBuf>,
    pub environment: HashMap<String, String>,
    pub inherit_environment: bool,
    pub redirect_stdin: bool,
    pub redirect_stdout: bool,
    pub redirect_stderr: bool,
}

impl ProcessOptions {
    pub fn new() -> Self;
    pub fn working_dir(self, dir: impl Into<PathBuf>) -> Self;
    pub fn env(self, key: impl Into<String>, value: impl Into<String>) -> Self;
    pub fn inherit_env(self, inherit: bool) -> Self;
    pub fn stdin(self, redirect: bool) -> Self;
    pub fn stdout(self, redirect: bool) -> Self;
    pub fn stderr(self, redirect: bool) -> Self;
}
```

## `CopilotProcess`

A running Copilot CLI process.

```rust
pub struct CopilotProcess { /* ... */ }

impl CopilotProcess {
    pub fn spawn(executable: impl AsRef<Path>, args: &[&str], options: ProcessOptions) -> Result<Self>;
    pub fn spawn_stdio(cli_path: impl AsRef<Path>) -> Result<Self>;

    pub fn take_transport(&mut self) -> Option<StdioTransport>;
    pub fn take_stdout(&mut self) -> Option<tokio::process::ChildStdout>;
    pub fn take_stderr(&mut self) -> Option<tokio::process::ChildStderr>;

    pub fn id(&self) -> Option<u32>;
    pub async fn is_running(&mut self) -> bool;
    pub async fn try_wait(&mut self) -> Result<Option<i32>>;
    pub async fn wait(&mut self) -> Result<i32>;
    pub fn terminate(&mut self) -> Result<()>;
    pub fn kill(&mut self) -> Result<()>;
}
```

- `spawn_stdio` is the common entry point: launch the CLI wired for stdio JSON-RPC.
- `take_transport` yields a [`StdioTransport`](/docs/api/transport#stdiotransport) for the JSON-RPC layer.
- `terminate` requests graceful shutdown; `kill` forces it.

## Related

- [Transport](/docs/api/transport)
- [Client lifecycle](/docs/core-concepts/client-lifecycle)
