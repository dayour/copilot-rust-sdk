// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Client-provided session filesystem (`sessionFs`).
//!
//! Mirrors the Node.js `sessionFsProvider` module. When a client declares a
//! [`SessionFsConfig`], the Copilot runtime stops touching the local disk and
//! instead issues `sessionFs.*` JSON-RPC **requests back to the client** for
//! every file operation. The client answers them through a
//! [`SessionFsProvider`] implementation registered on the session.
//!
//! This makes it possible to back a session with an in-memory filesystem, a
//! remote object store, or a sandboxed workspace.
//!
//! # Example
//!
//! ```no_run
//! use std::collections::HashMap;
//! use std::pin::Pin;
//! use std::sync::{Arc, Mutex};
//!
//! use copilot_sdk::{
//!     SessionFsDirEntry, SessionFsError, SessionFsFileInfo, SessionFsFuture,
//!     SessionFsProvider,
//! };
//!
//! #[derive(Default)]
//! struct MemoryFs {
//!     files: Mutex<HashMap<String, String>>,
//! }
//!
//! impl SessionFsProvider for MemoryFs {
//!     fn read_file<'a>(&'a self, path: &'a str) -> SessionFsFuture<'a, String> {
//!         Box::pin(async move {
//!             self.files
//!                 .lock()
//!                 .unwrap()
//!                 .get(path)
//!                 .cloned()
//!                 .ok_or_else(|| SessionFsError::not_found(path))
//!         })
//!     }
//!
//!     fn write_file<'a>(
//!         &'a self,
//!         path: &'a str,
//!         content: &'a str,
//!         _mode: Option<u32>,
//!     ) -> SessionFsFuture<'a, ()> {
//!         Box::pin(async move {
//!             self.files
//!                 .lock()
//!                 .unwrap()
//!                 .insert(path.to_string(), content.to_string());
//!             Ok(())
//!         })
//!     }
//!
//!     fn exists<'a>(&'a self, path: &'a str) -> SessionFsFuture<'a, bool> {
//!         Box::pin(async move { Ok(self.files.lock().unwrap().contains_key(path)) })
//!     }
//!
//!     fn stat<'a>(&'a self, path: &'a str) -> SessionFsFuture<'a, SessionFsFileInfo> {
//!         Box::pin(async move { Err(SessionFsError::not_found(path)) })
//!     }
//!
//!     fn readdir<'a>(&'a self, _path: &'a str) -> SessionFsFuture<'a, Vec<String>> {
//!         Box::pin(async move { Ok(Vec::new()) })
//!     }
//!
//!     fn readdir_with_types<'a>(
//!         &'a self,
//!         _path: &'a str,
//!     ) -> SessionFsFuture<'a, Vec<SessionFsDirEntry>> {
//!         Box::pin(async move { Ok(Vec::new()) })
//!     }
//!
//!     fn mkdir<'a>(
//!         &'a self,
//!         _path: &'a str,
//!         _recursive: bool,
//!         _mode: Option<u32>,
//!     ) -> SessionFsFuture<'a, ()> {
//!         Box::pin(async move { Ok(()) })
//!     }
//!
//!     fn rm<'a>(
//!         &'a self,
//!         path: &'a str,
//!         _recursive: bool,
//!         _force: bool,
//!     ) -> SessionFsFuture<'a, ()> {
//!         Box::pin(async move {
//!             self.files.lock().unwrap().remove(path);
//!             Ok(())
//!         })
//!     }
//!
//!     fn rename<'a>(&'a self, _src: &'a str, _dest: &'a str) -> SessionFsFuture<'a, ()> {
//!         Box::pin(async move { Ok(()) })
//!     }
//! }
//!
//! let provider: Arc<dyn SessionFsProvider> = Arc::new(MemoryFs::default());
//! ```

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use serde::{Deserialize, Serialize};

// =============================================================================
// Errors
// =============================================================================

/// Machine-readable classification of a session filesystem failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionFsErrorCode {
    /// The requested path does not exist.
    #[serde(rename = "ENOENT")]
    NotFound,
    /// The filesystem operation failed for an unspecified reason.
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

impl std::fmt::Display for SessionFsErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionFsErrorCode::NotFound => f.write_str("ENOENT"),
            SessionFsErrorCode::Unknown => f.write_str("UNKNOWN"),
        }
    }
}

/// Error returned by a [`SessionFsProvider`] operation.
///
/// Serialized on the wire as `{ "code": "ENOENT" | "UNKNOWN", "message": "..." }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionFsError {
    /// Machine-readable error classification.
    pub code: SessionFsErrorCode,
    /// Free-form detail about the error, for logging/diagnostics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl SessionFsError {
    /// Creates an `ENOENT` error for the given path.
    pub fn not_found(path: impl std::fmt::Display) -> Self {
        Self {
            code: SessionFsErrorCode::NotFound,
            message: Some(format!("ENOENT: no such file or directory, '{path}'")),
        }
    }

    /// Creates an `UNKNOWN` error with the given message.
    pub fn unknown(message: impl Into<String>) -> Self {
        Self {
            code: SessionFsErrorCode::Unknown,
            message: Some(message.into()),
        }
    }

    /// Returns `true` when this error represents a missing path.
    pub fn is_not_found(&self) -> bool {
        self.code == SessionFsErrorCode::NotFound
    }
}

impl std::fmt::Display for SessionFsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.message {
            Some(message) => write!(f, "{}: {}", self.code, message),
            None => write!(f, "{}", self.code),
        }
    }
}

impl std::error::Error for SessionFsError {}

impl From<std::io::Error> for SessionFsError {
    fn from(err: std::io::Error) -> Self {
        let code = if err.kind() == std::io::ErrorKind::NotFound {
            SessionFsErrorCode::NotFound
        } else {
            SessionFsErrorCode::Unknown
        };
        Self {
            code,
            message: Some(err.to_string()),
        }
    }
}

/// Result type for [`SessionFsProvider`] operations.
pub type SessionFsResult<T> = std::result::Result<T, SessionFsError>;

/// Boxed future returned by [`SessionFsProvider`] methods.
///
/// Async trait methods cannot be used with `dyn Trait`, so provider methods
/// return a boxed future — the same convention used by
/// [`Transport`](crate::transport::Transport).
pub type SessionFsFuture<'a, T> = Pin<Box<dyn Future<Output = SessionFsResult<T>> + Send + 'a>>;

// =============================================================================
// Value types
// =============================================================================

/// Metadata about a file or directory in the session filesystem.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionFsFileInfo {
    /// Whether the path is a regular file.
    pub is_file: bool,
    /// Whether the path is a directory.
    pub is_directory: bool,
    /// Size of the entry in bytes.
    pub size: u64,
    /// Last modification time as an ISO-8601 timestamp.
    pub mtime: String,
    /// Creation time as an ISO-8601 timestamp.
    pub birthtime: String,
}

/// Kind of entry returned by [`SessionFsProvider::readdir_with_types`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionFsEntryType {
    /// The entry is a file.
    File,
    /// The entry is a directory.
    Directory,
}

/// A directory entry with type information.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionFsDirEntry {
    /// Entry name (not a full path).
    pub name: String,
    /// Whether the entry is a file or a directory.
    #[serde(rename = "type")]
    pub entry_type: SessionFsEntryType,
}

impl SessionFsDirEntry {
    /// Creates a file entry.
    pub fn file(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entry_type: SessionFsEntryType::File,
        }
    }

    /// Creates a directory entry.
    pub fn directory(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entry_type: SessionFsEntryType::Directory,
        }
    }
}

/// How a SQLite statement should be executed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionFsSqliteQueryType {
    /// Execute DDL or multi-statement SQL without returning rows.
    Exec,
    /// Execute a `SELECT`-style query and return rows.
    Query,
    /// Execute `INSERT`, `UPDATE`, or `DELETE` SQL and return affected-row metadata.
    Run,
}

/// Result of a [`SessionFsSqliteProvider::query`] call.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionFsSqliteQueryResult {
    /// Result rows, each a column-name to value map.
    #[serde(default)]
    pub rows: Vec<serde_json::Map<String, serde_json::Value>>,
    /// Ordered column names for `rows`.
    #[serde(default)]
    pub columns: Vec<String>,
    /// Number of rows affected by the statement.
    #[serde(default)]
    pub rows_affected: u64,
    /// Rowid of the last inserted row, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_insert_rowid: Option<i64>,
    /// Error describing why the statement failed, when it did.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<SessionFsError>,
}

/// Bind parameters for a SQLite statement.
pub type SessionFsSqliteParams = HashMap<String, serde_json::Value>;

// =============================================================================
// Provider traits
// =============================================================================

/// Per-session SQLite database operations.
///
/// Implement this and return it from [`SessionFsProvider::sqlite`] to advertise
/// SQLite support. Declaring `capabilities.sqlite = true` in
/// [`SessionFsConfig`] without a SQLite provider is a configuration error.
pub trait SessionFsSqliteProvider: Send + Sync {
    /// Executes a SQLite query against the per-session database.
    fn query<'a>(
        &'a self,
        query_type: SessionFsSqliteQueryType,
        query: &'a str,
        params: Option<&'a SessionFsSqliteParams>,
    ) -> SessionFsFuture<'a, Option<SessionFsSqliteQueryResult>>;

    /// Checks whether the per-session database already exists, without creating it.
    fn exists(&self) -> SessionFsFuture<'_, bool>;
}

/// Client-side filesystem backing a Copilot session.
///
/// Every method mirrors a `sessionFs.*` JSON-RPC request issued by the runtime.
/// Return [`SessionFsError::not_found`] for missing paths; the SDK maps it to
/// the `ENOENT` wire code so the runtime can react appropriately.
pub trait SessionFsProvider: Send + Sync {
    /// Reads the full content of a file. Errors if the file does not exist.
    fn read_file<'a>(&'a self, path: &'a str) -> SessionFsFuture<'a, String>;

    /// Writes content to a file, creating parent directories if needed.
    fn write_file<'a>(
        &'a self,
        path: &'a str,
        content: &'a str,
        mode: Option<u32>,
    ) -> SessionFsFuture<'a, ()>;

    /// Appends content to a file, creating parent directories if needed.
    ///
    /// The default implementation reads the existing content (treating a
    /// missing file as empty) and writes it back with `content` appended.
    fn append_file<'a>(
        &'a self,
        path: &'a str,
        content: &'a str,
        mode: Option<u32>,
    ) -> SessionFsFuture<'a, ()> {
        Box::pin(async move {
            let existing = match self.read_file(path).await {
                Ok(existing) => existing,
                Err(err) if err.is_not_found() => String::new(),
                Err(err) => return Err(err),
            };
            self.write_file(path, &format!("{existing}{content}"), mode)
                .await
        })
    }

    /// Checks whether a path exists.
    fn exists<'a>(&'a self, path: &'a str) -> SessionFsFuture<'a, bool>;

    /// Gets metadata about a file or directory. Errors if it does not exist.
    fn stat<'a>(&'a self, path: &'a str) -> SessionFsFuture<'a, SessionFsFileInfo>;

    /// Lists entry names in a directory. Errors if it does not exist.
    fn readdir<'a>(&'a self, path: &'a str) -> SessionFsFuture<'a, Vec<String>>;

    /// Lists entries with type info. Errors if the directory does not exist.
    fn readdir_with_types<'a>(
        &'a self,
        path: &'a str,
    ) -> SessionFsFuture<'a, Vec<SessionFsDirEntry>>;

    /// Creates a directory. When `recursive` is true, creates parents as needed.
    fn mkdir<'a>(
        &'a self,
        path: &'a str,
        recursive: bool,
        mode: Option<u32>,
    ) -> SessionFsFuture<'a, ()>;

    /// Removes a file or directory. When `force` is true, missing paths are not an error.
    fn rm<'a>(&'a self, path: &'a str, recursive: bool, force: bool) -> SessionFsFuture<'a, ()>;

    /// Renames or moves a file or directory.
    fn rename<'a>(&'a self, src: &'a str, dest: &'a str) -> SessionFsFuture<'a, ()>;

    /// Optional per-session SQLite database operations.
    ///
    /// Returns `None` by default, meaning the provider does not support SQLite.
    fn sqlite(&self) -> Option<&dyn SessionFsSqliteProvider> {
        None
    }
}

// =============================================================================
// Configuration
// =============================================================================

/// Path conventions used by a client-provided session filesystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionFsConventions {
    /// Paths use Windows path conventions.
    Windows,
    /// Paths use POSIX path conventions.
    Posix,
}

impl Default for SessionFsConventions {
    fn default() -> Self {
        if cfg!(windows) {
            SessionFsConventions::Windows
        } else {
            SessionFsConventions::Posix
        }
    }
}

/// Optional capabilities advertised by a client-provided session filesystem.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionFsCapabilities {
    /// Whether the provider exposes a per-session SQLite database.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sqlite: Option<bool>,
}

impl SessionFsCapabilities {
    /// Returns `true` when SQLite support is advertised.
    pub fn has_sqlite(&self) -> bool {
        self.sqlite.unwrap_or(false)
    }
}

/// Client-level configuration enabling the session filesystem.
///
/// Set on [`ClientOptions::session_fs`](crate::types::ClientOptions::session_fs).
/// When present, the SDK sends `sessionFs.setProvider` after connecting and
/// every session must register a [`SessionFsProvider`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionFsConfig {
    /// Initial working directory presented to the runtime.
    pub initial_cwd: String,
    /// Directory where session state is persisted.
    pub session_state_path: String,
    /// Path conventions used by the provider.
    pub conventions: SessionFsConventions,
    /// Optional capabilities advertised to the runtime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<SessionFsCapabilities>,
}

impl SessionFsConfig {
    /// Creates a configuration with the required paths and default conventions.
    pub fn new(initial_cwd: impl Into<String>, session_state_path: impl Into<String>) -> Self {
        Self {
            initial_cwd: initial_cwd.into(),
            session_state_path: session_state_path.into(),
            conventions: SessionFsConventions::default(),
            capabilities: None,
        }
    }

    /// Overrides the path conventions.
    pub fn conventions(mut self, conventions: SessionFsConventions) -> Self {
        self.conventions = conventions;
        self
    }

    /// Advertises SQLite support.
    pub fn with_sqlite(mut self, enabled: bool) -> Self {
        self.capabilities = Some(SessionFsCapabilities {
            sqlite: Some(enabled),
        });
        self
    }

    /// Returns `true` when SQLite support is advertised.
    pub fn declares_sqlite(&self) -> bool {
        self.capabilities.is_some_and(|c| c.has_sqlite())
    }

    /// Validates that the required paths are non-empty.
    pub fn validate(&self) -> std::result::Result<(), String> {
        if self.initial_cwd.trim().is_empty() {
            return Err("sessionFs.initialCwd is required when sessionFs is configured".into());
        }
        if self.session_state_path.trim().is_empty() {
            return Err(
                "sessionFs.sessionStatePath is required when sessionFs is configured".into(),
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_code_wire_names() {
        assert_eq!(
            serde_json::to_value(SessionFsErrorCode::NotFound).unwrap(),
            serde_json::json!("ENOENT")
        );
        assert_eq!(
            serde_json::to_value(SessionFsErrorCode::Unknown).unwrap(),
            serde_json::json!("UNKNOWN")
        );
    }

    #[test]
    fn error_serializes_without_message() {
        let err = SessionFsError {
            code: SessionFsErrorCode::Unknown,
            message: None,
        };
        assert_eq!(
            serde_json::to_value(&err).unwrap(),
            serde_json::json!({ "code": "UNKNOWN" })
        );
    }

    #[test]
    fn io_error_maps_to_enoent() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let err: SessionFsError = io.into();
        assert!(err.is_not_found());

        let io = std::io::Error::other("boom");
        let err: SessionFsError = io.into();
        assert_eq!(err.code, SessionFsErrorCode::Unknown);
    }

    #[test]
    fn dir_entry_wire_shape() {
        assert_eq!(
            serde_json::to_value(SessionFsDirEntry::directory("src")).unwrap(),
            serde_json::json!({ "name": "src", "type": "directory" })
        );
        assert_eq!(
            serde_json::to_value(SessionFsDirEntry::file("main.rs")).unwrap(),
            serde_json::json!({ "name": "main.rs", "type": "file" })
        );
    }

    #[test]
    fn sqlite_query_type_wire_names() {
        assert_eq!(
            serde_json::to_value(SessionFsSqliteQueryType::Exec).unwrap(),
            serde_json::json!("exec")
        );
        assert_eq!(
            serde_json::to_value(SessionFsSqliteQueryType::Query).unwrap(),
            serde_json::json!("query")
        );
        assert_eq!(
            serde_json::to_value(SessionFsSqliteQueryType::Run).unwrap(),
            serde_json::json!("run")
        );
    }

    #[test]
    fn config_validation_and_builders() {
        let config = SessionFsConfig::new("/work", "/state")
            .conventions(SessionFsConventions::Posix)
            .with_sqlite(true);
        assert!(config.validate().is_ok());
        assert!(config.declares_sqlite());

        let value = serde_json::to_value(&config).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "initialCwd": "/work",
                "sessionStatePath": "/state",
                "conventions": "posix",
                "capabilities": { "sqlite": true }
            })
        );

        let bad = SessionFsConfig::new("", "/state");
        assert!(bad.validate().is_err());
        let bad = SessionFsConfig::new("/work", "  ");
        assert!(bad.validate().is_err());
    }

    #[test]
    fn file_info_wire_shape() {
        let info = SessionFsFileInfo {
            is_file: true,
            is_directory: false,
            size: 12,
            mtime: "2026-01-01T00:00:00.000Z".into(),
            birthtime: "2026-01-01T00:00:00.000Z".into(),
        };
        let value = serde_json::to_value(&info).unwrap();
        assert_eq!(value["isFile"], serde_json::json!(true));
        assert_eq!(value["isDirectory"], serde_json::json!(false));
        assert_eq!(value["size"], serde_json::json!(12));
    }

    struct AppendOnly {
        files: std::sync::Mutex<HashMap<String, String>>,
    }

    impl SessionFsProvider for AppendOnly {
        fn read_file<'a>(&'a self, path: &'a str) -> SessionFsFuture<'a, String> {
            Box::pin(async move {
                self.files
                    .lock()
                    .unwrap()
                    .get(path)
                    .cloned()
                    .ok_or_else(|| SessionFsError::not_found(path))
            })
        }

        fn write_file<'a>(
            &'a self,
            path: &'a str,
            content: &'a str,
            _mode: Option<u32>,
        ) -> SessionFsFuture<'a, ()> {
            Box::pin(async move {
                self.files
                    .lock()
                    .unwrap()
                    .insert(path.to_string(), content.to_string());
                Ok(())
            })
        }

        fn exists<'a>(&'a self, path: &'a str) -> SessionFsFuture<'a, bool> {
            Box::pin(async move { Ok(self.files.lock().unwrap().contains_key(path)) })
        }

        fn stat<'a>(&'a self, path: &'a str) -> SessionFsFuture<'a, SessionFsFileInfo> {
            Box::pin(async move { Err(SessionFsError::not_found(path)) })
        }

        fn readdir<'a>(&'a self, _path: &'a str) -> SessionFsFuture<'a, Vec<String>> {
            Box::pin(async move { Ok(Vec::new()) })
        }

        fn readdir_with_types<'a>(
            &'a self,
            _path: &'a str,
        ) -> SessionFsFuture<'a, Vec<SessionFsDirEntry>> {
            Box::pin(async move { Ok(Vec::new()) })
        }

        fn mkdir<'a>(
            &'a self,
            _path: &'a str,
            _recursive: bool,
            _mode: Option<u32>,
        ) -> SessionFsFuture<'a, ()> {
            Box::pin(async move { Ok(()) })
        }

        fn rm<'a>(
            &'a self,
            _path: &'a str,
            _recursive: bool,
            _force: bool,
        ) -> SessionFsFuture<'a, ()> {
            Box::pin(async move { Ok(()) })
        }

        fn rename<'a>(&'a self, _src: &'a str, _dest: &'a str) -> SessionFsFuture<'a, ()> {
            Box::pin(async move { Ok(()) })
        }
    }

    #[tokio::test]
    async fn default_append_file_creates_then_appends() {
        let fs = AppendOnly {
            files: std::sync::Mutex::new(HashMap::new()),
        };
        fs.append_file("/a.txt", "hello", None).await.unwrap();
        fs.append_file("/a.txt", " world", None).await.unwrap();
        assert_eq!(fs.read_file("/a.txt").await.unwrap(), "hello world");
        assert!(fs.sqlite().is_none());
    }
}
