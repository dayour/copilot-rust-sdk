// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Process management for the Copilot SDK.
//!
//! Provides async subprocess spawning and management for the Copilot CLI.

use crate::error::{CopilotError, Result};
use crate::transport::StdioTransport;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::{Child, Command};

// =============================================================================
// Process Options
// =============================================================================

/// Options for spawning a subprocess.
#[derive(Debug, Clone)]
pub struct ProcessOptions {
    /// Working directory for the subprocess (None = inherit from parent).
    pub working_directory: Option<PathBuf>,

    /// Environment variables to set.
    pub environment: HashMap<String, String>,

    /// Whether to inherit the parent's environment variables.
    pub inherit_environment: bool,

    /// Whether to redirect stdin (pipe to subprocess).
    pub redirect_stdin: bool,

    /// Whether to redirect stdout (pipe from subprocess).
    pub redirect_stdout: bool,

    /// Whether to redirect stderr (pipe from subprocess).
    pub redirect_stderr: bool,
}

impl Default for ProcessOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessOptions {
    /// Create new process options with default values.
    pub fn new() -> Self {
        Self {
            working_directory: None,
            environment: HashMap::new(),
            inherit_environment: true,
            redirect_stdin: true,
            redirect_stdout: true,
            redirect_stderr: false,
        }
    }

    /// Set working directory.
    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_directory = Some(dir.into());
        self
    }

    /// Add environment variable.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.insert(key.into(), value.into());
        self
    }

    /// Set whether to inherit parent environment.
    pub fn inherit_env(mut self, inherit: bool) -> Self {
        self.inherit_environment = inherit;
        self
    }

    /// Set stdin redirection.
    pub fn stdin(mut self, redirect: bool) -> Self {
        self.redirect_stdin = redirect;
        self
    }

    /// Set stdout redirection.
    pub fn stdout(mut self, redirect: bool) -> Self {
        self.redirect_stdout = redirect;
        self
    }

    /// Set stderr redirection.
    pub fn stderr(mut self, redirect: bool) -> Self {
        self.redirect_stderr = redirect;
        self
    }
}

// =============================================================================
// Copilot Process
// =============================================================================

/// A running Copilot CLI process.
pub struct CopilotProcess {
    child: Child,
    transport: Option<StdioTransport>,
    stdout: Option<tokio::process::ChildStdout>,
    stderr: Option<tokio::process::ChildStderr>,
}

impl CopilotProcess {
    /// Spawn a new Copilot CLI process.
    pub fn spawn(
        executable: impl AsRef<Path>,
        args: &[&str],
        options: ProcessOptions,
    ) -> Result<Self> {
        let executable = executable.as_ref();

        // Build command
        let mut cmd = Command::new(executable);
        cmd.args(args);

        // Set working directory
        if let Some(dir) = &options.working_directory {
            cmd.current_dir(dir);
        }

        // Set environment
        if !options.inherit_environment {
            cmd.env_clear();
        }
        for (key, value) in &options.environment {
            cmd.env(key, value);
        }

        // Configure stdio
        cmd.stdin(if options.redirect_stdin {
            Stdio::piped()
        } else {
            Stdio::null()
        });
        cmd.stdout(if options.redirect_stdout {
            Stdio::piped()
        } else {
            Stdio::null()
        });
        cmd.stderr(if options.redirect_stderr {
            Stdio::piped()
        } else {
            Stdio::null()
        });

        // Spawn the process
        let mut child = cmd.spawn().map_err(CopilotError::ProcessStart)?;

        // Create transport from stdio handles
        let transport = if options.redirect_stdin && options.redirect_stdout {
            let stdin = child
                .stdin
                .take()
                .ok_or_else(|| CopilotError::InvalidConfig("Failed to capture stdin".into()))?;
            let stdout = child
                .stdout
                .take()
                .ok_or_else(|| CopilotError::InvalidConfig("Failed to capture stdout".into()))?;
            Some(StdioTransport::new(stdin, stdout))
        } else {
            None
        };

        // Capture stdout if redirected but not used for stdio transport.
        let stdout = if transport.is_none() && options.redirect_stdout {
            child.stdout.take()
        } else {
            None
        };

        // Capture stderr if redirected
        let stderr = if options.redirect_stderr {
            child.stderr.take()
        } else {
            None
        };

        Ok(Self {
            child,
            transport,
            stdout,
            stderr,
        })
    }

    /// Spawn the Copilot CLI with default options for stdio mode.
    pub fn spawn_stdio(cli_path: impl AsRef<Path>) -> Result<Self> {
        let options = ProcessOptions::new().stdin(true).stdout(true).stderr(false);

        Self::spawn(cli_path, &["--stdio"], options)
    }

    /// Take the transport (can only be called once).
    ///
    /// Returns the stdio transport for communication with the CLI.
    pub fn take_transport(&mut self) -> Option<StdioTransport> {
        self.transport.take()
    }

    /// Take stdout (can only be called once).
    pub fn take_stdout(&mut self) -> Option<tokio::process::ChildStdout> {
        self.stdout.take()
    }

    /// Get the process ID.
    pub fn id(&self) -> Option<u32> {
        self.child.id()
    }

    /// Check if the process is still running.
    pub async fn is_running(&mut self) -> bool {
        self.child.try_wait().ok().flatten().is_none()
    }

    /// Try to get the exit status without blocking.
    pub async fn try_wait(&mut self) -> Result<Option<i32>> {
        match self.child.try_wait() {
            Ok(Some(status)) => Ok(Some(status.code().unwrap_or(-1))),
            Ok(None) => Ok(None),
            Err(e) => Err(CopilotError::Transport(e)),
        }
    }

    /// Wait for the process to exit.
    pub async fn wait(&mut self) -> Result<i32> {
        let status = self.child.wait().await.map_err(CopilotError::Transport)?;
        Ok(status.code().unwrap_or(-1))
    }

    /// Request termination of the process.
    ///
    /// On Unix, this sends SIGTERM. On Windows, this kills the process.
    pub fn terminate(&mut self) -> Result<()> {
        // Use kill for cross-platform simplicity
        // A more sophisticated implementation could use SIGTERM on Unix
        self.kill()
    }

    /// Forcefully kill the process.
    pub fn kill(&mut self) -> Result<()> {
        self.child.start_kill().map_err(CopilotError::Transport)
    }

    /// Take stderr (can only be called once).
    pub fn take_stderr(&mut self) -> Option<tokio::process::ChildStderr> {
        self.stderr.take()
    }
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Find an executable in the system PATH.
///
/// Returns the full path to the executable if found.
pub fn find_executable(name: &str) -> Option<PathBuf> {
    which::which(name).ok()
}

/// Check if a path looks like a Node.js script.
pub fn is_node_script(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext == "js" || ext == "mjs")
}

/// Get the system's Node.js executable path.
pub fn find_node() -> Option<PathBuf> {
    if let Ok(node_path) = std::env::var("NODE") {
        let node_path = node_path.trim();
        if !node_path.is_empty() {
            return Some(PathBuf::from(node_path));
        }
    }

    find_executable("node")
}

/// A fully resolved Copilot CLI launch target.
///
/// This can either be a native executable that should be spawned directly, or
/// the bundled `@github/copilot/index.js` entrypoint that must be launched via
/// a Node.js interpreter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolvedCopilotCli {
    /// A native executable that can be spawned directly.
    NativeExecutable(PathBuf),

    /// A bundled JavaScript entrypoint that must be launched as
    /// `node <script>`.
    NodeScript {
        /// The Node.js interpreter to execute.
        node_executable: PathBuf,

        /// The bundled `@github/copilot/index.js` script.
        script_path: PathBuf,
    },
}

#[derive(Debug, Clone)]
pub(crate) struct CopilotCliDiscovery {
    resolved_cli: Option<ResolvedCopilotCli>,
    searched_locations: Vec<String>,
}

impl CopilotCliDiscovery {
    pub(crate) fn into_resolved_cli(self) -> Option<ResolvedCopilotCli> {
        self.resolved_cli
    }

    pub(crate) fn not_found_message(&self) -> String {
        format!(
            "Could not find Copilot CLI executable. Searched {}",
            self.searched_locations.join(", ")
        )
    }
}

fn dedup_ancestor_directories(start_dirs: &[PathBuf]) -> Vec<PathBuf> {
    let mut seen = HashSet::new();
    let mut directories = Vec::new();

    for start_dir in start_dirs {
        for ancestor in start_dir.ancestors() {
            let ancestor = ancestor.to_path_buf();
            if seen.insert(ancestor.clone()) {
                directories.push(ancestor);
            }
        }
    }

    directories
}

fn bundled_cli_candidates(start_dirs: &[PathBuf]) -> Vec<PathBuf> {
    dedup_ancestor_directories(start_dirs)
        .into_iter()
        .map(|dir| {
            dir.join("node_modules")
                .join("@github")
                .join("copilot")
                .join("index.js")
        })
        .collect()
}

fn find_bundled_copilot_cli(
    start_dirs: &[PathBuf],
    node_executable: Option<PathBuf>,
) -> Option<ResolvedCopilotCli> {
    let node_executable = node_executable?;

    bundled_cli_candidates(start_dirs)
        .into_iter()
        .find(|candidate| candidate.is_file())
        .map(|script_path| ResolvedCopilotCli::NodeScript {
            node_executable,
            script_path,
        })
}

fn discover_copilot_cli_with<F>(
    cli_override: Option<&Path>,
    cwd_start: Option<&Path>,
    exe_start: Option<&Path>,
    find_in_path: F,
    node_executable: Option<PathBuf>,
) -> CopilotCliDiscovery
where
    F: Fn(&str) -> Option<PathBuf>,
{
    let mut searched_locations = Vec::new();

    if let Some(cli_path) = cli_override {
        searched_locations.push(format!("COPILOT_CLI_PATH={}", cli_path.display()));
        if cli_path.exists() {
            return CopilotCliDiscovery {
                resolved_cli: Some(ResolvedCopilotCli::NativeExecutable(cli_path.to_path_buf())),
                searched_locations,
            };
        }
    } else {
        searched_locations.push("COPILOT_CLI_PATH".to_string());
    }

    searched_locations.push("PATH entry \"copilot\"".to_string());
    if let Some(path) = find_in_path("copilot") {
        return CopilotCliDiscovery {
            resolved_cli: Some(ResolvedCopilotCli::NativeExecutable(path)),
            searched_locations,
        };
    }

    #[cfg(windows)]
    {
        searched_locations.push("PATH entry \"copilot.cmd\"".to_string());
        if let Some(path) = find_in_path("copilot.cmd") {
            return CopilotCliDiscovery {
                resolved_cli: Some(ResolvedCopilotCli::NativeExecutable(path)),
                searched_locations,
            };
        }

        searched_locations.push("PATH entry \"copilot.exe\"".to_string());
        if let Some(path) = find_in_path("copilot.exe") {
            return CopilotCliDiscovery {
                resolved_cli: Some(ResolvedCopilotCli::NativeExecutable(path)),
                searched_locations,
            };
        }
    }

    let mut bundle_start_dirs = Vec::new();
    match cwd_start {
        Some(dir) => bundle_start_dirs.push(dir.to_path_buf()),
        None => searched_locations.push(
            "bundled @github/copilot/index.js via current working directory ancestors (unavailable)"
                .to_string(),
        ),
    }
    match exe_start {
        Some(dir) => bundle_start_dirs.push(dir.to_path_buf()),
        None => searched_locations.push(
            "bundled @github/copilot/index.js via current executable ancestors (unavailable)"
                .to_string(),
        ),
    }

    let bundled_candidates = bundled_cli_candidates(&bundle_start_dirs);
    searched_locations.extend(bundled_candidates.iter().map(|candidate| {
        format!(
            "bundled @github/copilot/index.js at {}",
            candidate.display()
        )
    }));

    let resolved_cli = find_bundled_copilot_cli(&bundle_start_dirs, node_executable);

    CopilotCliDiscovery {
        resolved_cli,
        searched_locations,
    }
}

/// Discover how to launch the Copilot CLI.
///
/// Discovery precedence is:
/// 1. `COPILOT_CLI_PATH`
/// 2. `copilot` on `PATH` (plus `copilot.cmd` / `copilot.exe` on Windows)
/// 3. Bundled `node_modules/@github/copilot/index.js` by walking up from the
///    current working directory and current executable directory
pub(crate) fn discover_copilot_cli() -> CopilotCliDiscovery {
    let cli_override = std::env::var("COPILOT_CLI_PATH")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let cwd_start = std::env::current_dir().ok();
    let exe_start = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf));

    discover_copilot_cli_with(
        cli_override.as_deref(),
        cwd_start.as_deref(),
        exe_start.as_deref(),
        find_executable,
        find_node(),
    )
}

/// Find the Copilot CLI entrypoint path.
///
/// Returns the native executable path when one is discovered directly. For the
/// bundled npm package fallback, returns the `@github/copilot/index.js`
/// script path.
pub fn find_copilot_cli() -> Option<PathBuf> {
    discover_copilot_cli()
        .into_resolved_cli()
        .map(|resolved| match resolved {
            ResolvedCopilotCli::NativeExecutable(path) => path,
            ResolvedCopilotCli::NodeScript { script_path, .. } => script_path,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_options_builder() {
        let options = ProcessOptions::new()
            .working_dir("/tmp")
            .env("FOO", "bar")
            .inherit_env(false)
            .stdin(true)
            .stdout(true)
            .stderr(true);

        assert_eq!(options.working_directory, Some(PathBuf::from("/tmp")));
        assert_eq!(options.environment.get("FOO"), Some(&"bar".to_string()));
        assert!(!options.inherit_environment);
        assert!(options.redirect_stdin);
        assert!(options.redirect_stdout);
        assert!(options.redirect_stderr);
    }

    #[test]
    fn test_process_options_default() {
        let options = ProcessOptions::default();

        assert!(options.working_directory.is_none());
        assert!(options.environment.is_empty());
        assert!(options.inherit_environment);
        assert!(options.redirect_stdin);
        assert!(options.redirect_stdout);
        assert!(!options.redirect_stderr);
    }

    #[test]
    fn test_is_node_script() {
        assert!(is_node_script(Path::new("script.js")));
        assert!(is_node_script(Path::new("script.mjs")));
        assert!(is_node_script(Path::new("/path/to/script.js")));
        assert!(!is_node_script(Path::new("script.ts")));
        assert!(!is_node_script(Path::new("script")));
        assert!(!is_node_script(Path::new("script.py")));
    }

    #[test]
    fn test_find_node() {
        // This test just verifies the function doesn't panic
        // Whether it finds node depends on the system
        let _ = find_node();
    }

    #[test]
    fn test_bundled_cli_walk_up_finds_index_js() {
        let fixture = unique_temp_path("bundled-found");
        let deep_dir = fixture.join("workspace").join("project").join("nested");
        let bundled_index = fixture
            .join("workspace")
            .join("node_modules")
            .join("@github")
            .join("copilot")
            .join("index.js");

        std::fs::create_dir_all(&deep_dir).unwrap();
        std::fs::create_dir_all(bundled_index.parent().unwrap()).unwrap();
        std::fs::write(&bundled_index, "// bundled copilot").unwrap();

        let resolved = find_bundled_copilot_cli(&[deep_dir], Some(PathBuf::from("node")));

        assert_eq!(
            resolved,
            Some(ResolvedCopilotCli::NodeScript {
                node_executable: PathBuf::from("node"),
                script_path: bundled_index.clone(),
            })
        );

        std::fs::remove_dir_all(&fixture).unwrap();
    }

    #[test]
    fn test_bundled_cli_walk_up_returns_none_when_absent() {
        let fixture = unique_temp_path("bundled-missing");
        let deep_dir = fixture.join("workspace").join("project").join("nested");

        std::fs::create_dir_all(&deep_dir).unwrap();

        let resolved = find_bundled_copilot_cli(&[deep_dir], Some(PathBuf::from("node")));

        assert_eq!(resolved, None);

        std::fs::remove_dir_all(&fixture).unwrap();
    }

    #[test]
    fn test_copilot_cli_path_takes_precedence_over_bundled_cli() {
        let fixture = unique_temp_path("bundled-precedence");
        let override_path = fixture.join("custom").join("copilot");
        let cwd_start = fixture.join("workspace").join("project").join("nested");
        let exe_start = fixture.join("tooling").join("bin");
        let bundled_index = fixture
            .join("workspace")
            .join("node_modules")
            .join("@github")
            .join("copilot")
            .join("index.js");

        std::fs::create_dir_all(override_path.parent().unwrap()).unwrap();
        std::fs::create_dir_all(&cwd_start).unwrap();
        std::fs::create_dir_all(&exe_start).unwrap();
        std::fs::create_dir_all(bundled_index.parent().unwrap()).unwrap();
        std::fs::write(&override_path, "").unwrap();
        std::fs::write(&bundled_index, "// bundled copilot").unwrap();

        let discovery = discover_copilot_cli_with(
            Some(&override_path),
            Some(&cwd_start),
            Some(&exe_start),
            |_| None,
            Some(PathBuf::from("node")),
        );

        assert_eq!(
            discovery.into_resolved_cli(),
            Some(ResolvedCopilotCli::NativeExecutable(override_path.clone()))
        );

        std::fs::remove_dir_all(&fixture).unwrap();
    }

    fn unique_temp_path(label: &str) -> PathBuf {
        let unique = format!(
            "copilot-sdk-rust-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        std::env::temp_dir().join(unique)
    }
}
