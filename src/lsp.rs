// Copyright (c) 2026 Elias Bachaalany
// SPDX-License-Identifier: MIT

//! Rust-native Language Server Protocol support.
//!
//! The module intentionally keeps the LSP surface typed and deterministic. Semantic
//! identities are human-readable strong names with an explicit schema version so that
//! clients can safely cache symbols across process restarts.

use crate::error::{CopilotError, Result};
use crate::jsonrpc::{JsonRpcError, JsonRpcId, JsonRpcRequest, JsonRpcResponse};
use crate::transport::{MessageReader, MessageWriter};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::RwLock;

/// LSP protocol version implemented by this module.
pub const LSP_PROTOCOL_VERSION: &str = "3.17";

/// Version of the semantic identity schema.
pub const SEMANTIC_SCHEMA_VERSION: u32 = 1;

/// Prefix used for stable semantic IDs.
pub const SEMANTIC_ID_PREFIX: &str = "rust-lsp-semantic-v1:";

/// Canonical LSP method names.
pub mod methods {
    pub const INITIALIZE: &str = "initialize";
    pub const INITIALIZED: &str = "initialized";
    pub const SHUTDOWN: &str = "shutdown";
    pub const EXIT: &str = "exit";
    pub const DID_OPEN: &str = "textDocument/didOpen";
    pub const DID_CHANGE: &str = "textDocument/didChange";
    pub const DID_CLOSE: &str = "textDocument/didClose";
    pub const DOCUMENT_SYMBOL: &str = "textDocument/documentSymbol";
    pub const WORKSPACE_SYMBOL: &str = "workspace/symbol";
}

/// A canonical, validated semantic name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CanonicalName(String);

impl CanonicalName {
    /// Create a canonical name from a qualified Rust-style name.
    ///
    /// Dots and slashes are normalized to `::`, empty segments are rejected, and
    /// whitespace/control characters are not allowed in semantic names.
    pub fn new(name: impl AsRef<str>) -> Result<Self> {
        let raw = name.as_ref().trim();
        if raw.is_empty() {
            return Err(CopilotError::InvalidConfig(
                "Semantic names cannot be empty".into(),
            ));
        }

        let normalized = raw.replace(['/', '.'], "::");
        let mut segments = Vec::new();
        for segment in normalized.split("::") {
            let segment = segment.trim();
            if segment.is_empty()
                || segment.contains(':')
                || segment.chars().any(|c| c.is_whitespace() || c.is_control())
            {
                return Err(CopilotError::InvalidConfig(format!(
                    "Invalid semantic name: {raw}"
                )));
            }
            segments.push(segment);
        }

        Ok(Self(segments.join("::")))
    }

    /// Return the canonical name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for CanonicalName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A stable semantic ID derived from a strong name.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SemanticId(String);

impl SemanticId {
    /// Return the stable ID.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SemanticId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// LSP symbol kinds used by the semantic identity layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SymbolKind {
    File,
    Module,
    Namespace,
    Package,
    Class,
    Method,
    Property,
    Field,
    Constructor,
    Enum,
    Interface,
    Function,
    Variable,
    Constant,
    String,
    Number,
    Boolean,
    Array,
    Object,
    Key,
    Null,
    EnumMember,
    Struct,
    Event,
    Operator,
    TypeParameter,
}

impl SymbolKind {
    /// Return the numeric LSP representation.
    pub const fn lsp_code(self) -> u8 {
        match self {
            Self::File => 1,
            Self::Module => 2,
            Self::Namespace => 3,
            Self::Package => 4,
            Self::Class => 5,
            Self::Method => 6,
            Self::Property => 7,
            Self::Field => 8,
            Self::Constructor => 9,
            Self::Enum => 10,
            Self::Interface => 11,
            Self::Function => 12,
            Self::Variable => 13,
            Self::Constant => 14,
            Self::String => 15,
            Self::Number => 16,
            Self::Boolean => 17,
            Self::Array => 18,
            Self::Object => 19,
            Self::Key => 20,
            Self::Null => 21,
            Self::EnumMember => 22,
            Self::Struct => 23,
            Self::Event => 24,
            Self::Operator => 25,
            Self::TypeParameter => 26,
        }
    }

    const fn canonical_token(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Module => "module",
            Self::Namespace => "namespace",
            Self::Package => "package",
            Self::Class => "class",
            Self::Method => "method",
            Self::Property => "property",
            Self::Field => "field",
            Self::Constructor => "constructor",
            Self::Enum => "enum",
            Self::Interface => "interface",
            Self::Function => "function",
            Self::Variable => "variable",
            Self::Constant => "constant",
            Self::String => "string",
            Self::Number => "number",
            Self::Boolean => "boolean",
            Self::Array => "array",
            Self::Object => "object",
            Self::Key => "key",
            Self::Null => "null",
            Self::EnumMember => "enum_member",
            Self::Struct => "struct",
            Self::Event => "event",
            Self::Operator => "operator",
            Self::TypeParameter => "type_parameter",
        }
    }
}

/// A strong semantic name for a symbol.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StrongName {
    pub kind: SymbolKind,
    pub name: CanonicalName,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl StrongName {
    /// Build a strong name from its semantic components.
    pub fn new(kind: SymbolKind, name: impl AsRef<str>, signature: Option<String>) -> Result<Self> {
        let signature = signature
            .map(|value| value.split_whitespace().collect::<Vec<_>>().join(" "))
            .filter(|value| !value.is_empty());
        Ok(Self {
            kind,
            name: CanonicalName::new(name)?,
            signature,
        })
    }

    /// Return the canonical strong name.
    pub fn canonical_name(&self) -> String {
        match &self.signature {
            Some(signature) => {
                format!("{}:{}({signature})", self.kind.canonical_token(), self.name)
            }
            None => format!("{}:{}", self.kind.canonical_token(), self.name),
        }
    }

    /// Return the stable semantic ID.
    pub fn semantic_id(&self) -> SemanticId {
        SemanticId(format!("{SEMANTIC_ID_PREFIX}{}", self.canonical_name()))
    }
}

/// A zero-based LSP document position.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// A document range.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

/// A text document currently known to the server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentItem {
    pub uri: String,
    pub language_id: String,
    pub version: i32,
    pub text: String,
}

/// A text document identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentIdentifier {
    pub uri: String,
}

/// A versioned text document identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionedTextDocumentIdentifier {
    pub uri: String,
    pub version: i32,
}

/// Parameters for `textDocument/didOpen`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidOpenTextDocumentParams {
    pub text_document: TextDocumentItem,
}

/// A content change. This implementation advertises full synchronization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextDocumentContentChangeEvent {
    #[serde(default)]
    pub range: Option<Range>,
    #[serde(default)]
    pub range_length: Option<u32>,
    pub text: String,
}

/// Parameters for `textDocument/didChange`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidChangeTextDocumentParams {
    pub text_document: VersionedTextDocumentIdentifier,
    pub content_changes: Vec<TextDocumentContentChangeEvent>,
}

/// Parameters for `textDocument/didClose`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DidCloseTextDocumentParams {
    pub text_document: TextDocumentIdentifier,
}

/// Parameters for `textDocument/documentSymbol`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSymbolParams {
    pub text_document: TextDocumentIdentifier,
}

/// Parameters for `workspace/symbol`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceSymbolParams {
    #[serde(default)]
    pub query: String,
}

/// A workspace folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFolder {
    pub uri: String,
    pub name: String,
}

/// LSP initialization parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    #[serde(default)]
    pub process_id: Option<u32>,
    #[serde(default)]
    pub root_uri: Option<String>,
    #[serde(default)]
    pub workspace_folders: Option<Vec<WorkspaceFolder>>,
    #[serde(default)]
    pub capabilities: Value,
    #[serde(default)]
    pub initialization_options: Option<Value>,
}

/// Server capabilities returned during initialization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_document_sync: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_symbol_provider: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_symbol_provider: Option<bool>,
}

/// LSP server metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// LSP initialization result.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
}

/// A location in a text document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub uri: String,
    pub range: Range,
}

/// A document symbol with semantic identity metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSymbol {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    pub kind: u8,
    pub range: Range,
    pub selection_range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<DocumentSymbol>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// A workspace symbol with semantic identity metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolInformation {
    pub name: String,
    pub kind: u8,
    pub location: Location,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Hover content returned by a semantic provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hover {
    pub contents: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<Range>,
}

/// Semantic provider used by [`LspServer`].
pub trait SemanticProvider: Send + Sync {
    /// Return symbols for one open document.
    fn document_symbols(&self, document: &TextDocumentItem) -> Result<Vec<DocumentSymbol>>;

    /// Return workspace symbols matching a query.
    fn workspace_symbols(
        &self,
        query: &str,
        documents: &[TextDocumentItem],
    ) -> Result<Vec<SymbolInformation>>;
}

/// A deterministic, dependency-free Rust declaration provider.
#[derive(Debug, Default)]
pub struct RustSemanticProvider;

impl RustSemanticProvider {
    fn document_key(uri: &str) -> String {
        let mut key = String::with_capacity(uri.len());
        for character in uri.chars() {
            if character.is_ascii_alphanumeric() || character == '_' {
                key.push(character.to_ascii_lowercase());
            } else {
                key.push('_');
            }
        }
        if key.is_empty() {
            "document".into()
        } else {
            key
        }
    }

    fn declaration(line: &str) -> Option<(SymbolKind, &str, Option<String>)> {
        let mut candidate = line.trim_start();
        for prefix in ["pub(crate) ", "pub ", "async ", "unsafe "] {
            if let Some(stripped) = candidate.strip_prefix(prefix) {
                candidate = stripped;
            }
        }

        let declarations = [
            ("fn ", SymbolKind::Function),
            ("struct ", SymbolKind::Struct),
            ("enum ", SymbolKind::Enum),
            ("trait ", SymbolKind::Interface),
            ("mod ", SymbolKind::Module),
            ("const ", SymbolKind::Constant),
            ("static ", SymbolKind::Variable),
            ("type ", SymbolKind::TypeParameter),
            ("impl ", SymbolKind::Class),
        ];

        for (prefix, kind) in declarations {
            let Some(rest) = candidate.strip_prefix(prefix) else {
                continue;
            };
            let end = rest
                .find(|character: char| {
                    !(character.is_ascii_alphanumeric() || character == '_' || character == '!')
                })
                .unwrap_or(rest.len());
            let name = &rest[..end];
            if name.is_empty() {
                continue;
            }
            let signature = (kind == SymbolKind::Function)
                .then(|| rest.split('{').next().unwrap_or(rest).trim().to_string());
            return Some((kind, name, signature));
        }

        None
    }

    fn build_symbol(
        document: &TextDocumentItem,
        line_number: u32,
        line: &str,
        kind: SymbolKind,
        name: &str,
        signature: Option<String>,
    ) -> Result<DocumentSymbol> {
        let document_name = Self::document_key(&document.uri);
        let strong_name = StrongName::new(
            kind,
            format!("document::{document_name}::{name}"),
            signature,
        )?;
        let end_character = line.chars().count() as u32;
        let range = Range {
            start: Position {
                line: line_number,
                character: 0,
            },
            end: Position {
                line: line_number,
                character: end_character,
            },
        };

        Ok(DocumentSymbol {
            name: name.to_string(),
            detail: Some(strong_name.canonical_name()),
            kind: kind.lsp_code(),
            range,
            selection_range: range,
            children: None,
            data: Some(json!({
                "semanticId": strong_name.semantic_id().as_str(),
                "canonicalName": strong_name.canonical_name(),
                "schemaVersion": SEMANTIC_SCHEMA_VERSION,
            })),
        })
    }
}

impl SemanticProvider for RustSemanticProvider {
    fn document_symbols(&self, document: &TextDocumentItem) -> Result<Vec<DocumentSymbol>> {
        document
            .text
            .lines()
            .enumerate()
            .filter_map(|(line_number, line)| {
                Self::declaration(line).map(|(kind, name, signature)| {
                    Self::build_symbol(document, line_number as u32, line, kind, name, signature)
                })
            })
            .collect()
    }

    fn workspace_symbols(
        &self,
        query: &str,
        documents: &[TextDocumentItem],
    ) -> Result<Vec<SymbolInformation>> {
        let query = query.to_ascii_lowercase();
        let mut symbols = Vec::new();
        for document in documents {
            for symbol in self.document_symbols(document)? {
                if !query.is_empty() && !symbol.name.to_ascii_lowercase().contains(&query) {
                    continue;
                }
                symbols.push(SymbolInformation {
                    name: symbol.name,
                    kind: symbol.kind,
                    location: Location {
                        uri: document.uri.clone(),
                        range: symbol.range,
                    },
                    container_name: None,
                    data: symbol.data,
                });
            }
        }
        Ok(symbols)
    }
}

/// Open documents held by an LSP server.
#[derive(Clone, Default)]
pub struct DocumentStore {
    documents: Arc<RwLock<BTreeMap<String, TextDocumentItem>>>,
}

impl DocumentStore {
    /// Open or replace a document.
    pub async fn open(&self, document: TextDocumentItem) {
        self.documents
            .write()
            .await
            .insert(document.uri.clone(), document);
    }

    /// Apply full-document changes.
    pub async fn apply_changes(
        &self,
        identifier: &VersionedTextDocumentIdentifier,
        changes: &[TextDocumentContentChangeEvent],
    ) -> Result<()> {
        let mut documents = self.documents.write().await;
        let document = documents.get_mut(&identifier.uri).ok_or_else(|| {
            CopilotError::Protocol(format!(
                "Cannot change unopened document: {}",
                identifier.uri
            ))
        })?;

        for change in changes {
            if change.range.is_some() {
                return Err(CopilotError::Protocol(
                    "Incremental text changes are not supported; use full synchronization".into(),
                ));
            }
            document.text = change.text.clone();
        }
        document.version = identifier.version;
        Ok(())
    }

    /// Close a document.
    pub async fn close(&self, identifier: &TextDocumentIdentifier) {
        self.documents.write().await.remove(&identifier.uri);
    }

    /// Read one open document.
    pub async fn get(&self, uri: &str) -> Option<TextDocumentItem> {
        self.documents.read().await.get(uri).cloned()
    }

    /// Return all open documents in deterministic URI order.
    pub async fn all(&self) -> Vec<TextDocumentItem> {
        self.documents.read().await.values().cloned().collect()
    }
}

/// Configuration for a native LSP server.
#[derive(Debug, Clone)]
pub struct LspServerConfig {
    pub server_name: String,
    pub server_version: String,
    pub session_id: Option<String>,
    pub workspace_root: Option<String>,
}

impl Default for LspServerConfig {
    fn default() -> Self {
        Self {
            server_name: "copilot-sdk-rust".into(),
            server_version: env!("CARGO_PKG_VERSION").into(),
            session_id: None,
            workspace_root: None,
        }
    }
}

impl LspServerConfig {
    /// Create a configuration associated with a Copilot session.
    pub fn for_session(session_id: impl Into<String>, workspace_root: Option<&str>) -> Self {
        Self {
            session_id: Some(session_id.into()),
            workspace_root: workspace_root.map(str::to_string),
            ..Self::default()
        }
    }
}

/// A native async LSP server using Content-Length framing.
pub struct LspServer {
    config: LspServerConfig,
    documents: DocumentStore,
    provider: Arc<dyn SemanticProvider>,
}

impl Default for LspServer {
    fn default() -> Self {
        Self::new()
    }
}

impl LspServer {
    /// Create a server with the built-in Rust semantic provider.
    pub fn new() -> Self {
        Self::with_config_and_provider(LspServerConfig::default(), Arc::new(RustSemanticProvider))
    }

    /// Create a server with custom configuration and the Rust provider.
    pub fn with_config(config: LspServerConfig) -> Self {
        Self::with_config_and_provider(config, Arc::new(RustSemanticProvider))
    }

    /// Create a server with custom configuration and semantic provider.
    pub fn with_config_and_provider(
        config: LspServerConfig,
        provider: Arc<dyn SemanticProvider>,
    ) -> Self {
        Self {
            config,
            documents: DocumentStore::default(),
            provider,
        }
    }

    /// Return the server configuration.
    pub fn config(&self) -> &LspServerConfig {
        &self.config
    }

    /// Return the document store used by this server.
    pub fn documents(&self) -> DocumentStore {
        self.documents.clone()
    }

    /// Return the capabilities advertised during initialization.
    pub fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            text_document_sync: Some(1),
            document_symbol_provider: Some(true),
            workspace_symbol_provider: Some(true),
        }
    }

    /// Serve LSP requests until `exit` or end-of-stream.
    pub async fn serve<R, W>(self, input: R, output: W) -> Result<()>
    where
        R: AsyncRead + Unpin + Send,
        W: AsyncWrite + Unpin + Send,
    {
        let mut reader = MessageReader::new(input);
        let mut writer = MessageWriter::new(output);
        let mut initialized = false;
        let mut shutdown_requested = false;

        loop {
            let message = match reader.read_message().await {
                Ok(message) => message,
                Err(CopilotError::ConnectionClosed) => return Ok(()),
                Err(error) => return Err(error),
            };

            let value: Value = match serde_json::from_str(&message) {
                Ok(value) => value,
                Err(error) => {
                    let response = JsonRpcResponse {
                        jsonrpc: "2.0".into(),
                        id: None,
                        result: None,
                        error: Some(JsonRpcError::new(
                            JsonRpcError::PARSE_ERROR,
                            error.to_string(),
                        )),
                    };
                    writer
                        .write_message(&serde_json::to_string(&response)?)
                        .await?;
                    continue;
                }
            };

            let request: JsonRpcRequest = match serde_json::from_value(value) {
                Ok(request) => request,
                Err(error) => {
                    let response = JsonRpcResponse {
                        jsonrpc: "2.0".into(),
                        id: None,
                        result: None,
                        error: Some(JsonRpcError::new(
                            JsonRpcError::INVALID_REQUEST,
                            error.to_string(),
                        )),
                    };
                    writer
                        .write_message(&serde_json::to_string(&response)?)
                        .await?;
                    continue;
                }
            };

            if request.is_notification() {
                let is_exit = request.method == methods::EXIT;
                if let Err(error) = self
                    .handle_notification(&request.method, request.params.unwrap_or(Value::Null))
                    .await
                {
                    tracing::warn!(method = %request.method, error = %error, "LSP notification failed");
                }
                if is_exit {
                    return Ok(());
                }
                continue;
            }

            let id = request.id.clone().unwrap_or(JsonRpcId::Num(0));
            let response = match self
                .handle_request(
                    &request.method,
                    request.params.unwrap_or(Value::Null),
                    &mut initialized,
                    &mut shutdown_requested,
                )
                .await
            {
                Ok(result) => JsonRpcResponse::success(id, result),
                Err(error) => JsonRpcResponse::error(id, error),
            };
            writer
                .write_message(&serde_json::to_string(&response)?)
                .await?;
        }
    }

    async fn handle_request(
        &self,
        method: &str,
        params: Value,
        initialized: &mut bool,
        shutdown_requested: &mut bool,
    ) -> std::result::Result<Value, JsonRpcError> {
        if !*initialized && method != methods::INITIALIZE {
            return Err(JsonRpcError::new(
                JsonRpcError::INVALID_REQUEST,
                "The server must be initialized first",
            ));
        }
        if *shutdown_requested {
            return Err(JsonRpcError::new(
                JsonRpcError::INVALID_REQUEST,
                "The server is shutting down",
            ));
        }

        match method {
            methods::INITIALIZE => {
                if *initialized {
                    return Err(JsonRpcError::new(
                        JsonRpcError::INVALID_REQUEST,
                        "The server was already initialized",
                    ));
                }
                let _: InitializeParams = parse_params(params)?;
                *initialized = true;
                serde_json::to_value(InitializeResult {
                    capabilities: self.capabilities(),
                    server_info: ServerInfo {
                        name: self.config.server_name.clone(),
                        version: self.config.server_version.clone(),
                    },
                })
                .map_err(internal_error)
            }
            methods::SHUTDOWN => {
                *shutdown_requested = true;
                Ok(Value::Null)
            }
            methods::DOCUMENT_SYMBOL => {
                let params: DocumentSymbolParams = parse_params(params)?;
                let document = self.documents.get(&params.text_document.uri).await;
                let symbols = match document {
                    Some(document) => self
                        .provider
                        .document_symbols(&document)
                        .map_err(internal_error)?,
                    None => Vec::new(),
                };
                serde_json::to_value(symbols).map_err(internal_error)
            }
            methods::WORKSPACE_SYMBOL => {
                let params: WorkspaceSymbolParams = parse_params(params)?;
                let symbols = self
                    .provider
                    .workspace_symbols(&params.query, &self.documents.all().await)
                    .map_err(internal_error)?;
                serde_json::to_value(symbols).map_err(internal_error)
            }
            _ => Err(JsonRpcError::new(
                JsonRpcError::METHOD_NOT_FOUND,
                format!("Method not found: {method}"),
            )),
        }
    }

    async fn handle_notification(&self, method: &str, params: Value) -> Result<()> {
        match method {
            methods::INITIALIZED => Ok(()),
            methods::DID_OPEN => {
                let params: DidOpenTextDocumentParams = parse_notification_params(params)?;
                self.documents.open(params.text_document).await;
                Ok(())
            }
            methods::DID_CHANGE => {
                let params: DidChangeTextDocumentParams = parse_notification_params(params)?;
                self.documents
                    .apply_changes(&params.text_document, &params.content_changes)
                    .await
            }
            methods::DID_CLOSE => {
                let params: DidCloseTextDocumentParams = parse_notification_params(params)?;
                self.documents.close(&params.text_document).await;
                Ok(())
            }
            methods::EXIT => Ok(()),
            _ => Ok(()),
        }
    }
}

fn parse_params<T: DeserializeOwned>(params: Value) -> std::result::Result<T, JsonRpcError> {
    serde_json::from_value(params)
        .map_err(|error| JsonRpcError::new(JsonRpcError::INVALID_PARAMS, error.to_string()))
}

fn parse_notification_params<T: DeserializeOwned>(params: Value) -> Result<T> {
    serde_json::from_value(params).map_err(|error| {
        CopilotError::Protocol(format!("Invalid LSP notification parameters: {error}"))
    })
}

fn internal_error(error: impl std::fmt::Display) -> JsonRpcError {
    JsonRpcError::new(JsonRpcError::INTERNAL_ERROR, error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::{MessageReader, MessageWriter};
    use tokio::io::duplex;

    #[test]
    fn canonical_names_are_deterministic() {
        let name = CanonicalName::new("crate/module.Type").unwrap();
        assert_eq!(name.as_str(), "crate::module::Type");
    }

    #[test]
    fn strong_names_include_kind_and_schema_version() {
        let strong_name = StrongName::new(
            SymbolKind::Function,
            "crate::main",
            Some("fn main()".into()),
        )
        .unwrap();
        assert_eq!(
            strong_name.semantic_id().as_str(),
            "rust-lsp-semantic-v1:function:crate::main(fn main())"
        );
    }

    #[tokio::test]
    async fn server_initializes_tracks_documents_and_returns_symbols() {
        let (client, server) = duplex(16 * 1024);
        let (client_read, client_write) = tokio::io::split(client);
        let (server_read, server_write) = tokio::io::split(server);

        let task =
            tokio::spawn(async move { LspServer::new().serve(server_read, server_write).await });
        let mut writer = MessageWriter::new(client_write);
        let mut reader = MessageReader::new(client_read);

        writer
            .write_message(
                &json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": methods::INITIALIZE,
                    "params": {"capabilities": {}}
                })
                .to_string(),
            )
            .await
            .unwrap();
        let initialize_response: Value =
            serde_json::from_str(&reader.read_message().await.unwrap()).unwrap();
        assert_eq!(
            initialize_response["result"]["serverInfo"]["name"],
            "copilot-sdk-rust"
        );

        writer
            .write_message(
                &json!({
                    "jsonrpc": "2.0",
                    "method": methods::INITIALIZED,
                    "params": {}
                })
                .to_string(),
            )
            .await
            .unwrap();
        writer
            .write_message(
                &json!({
                    "jsonrpc": "2.0",
                    "method": methods::DID_OPEN,
                    "params": {
                        "textDocument": {
                            "uri": "file:///src/main.rs",
                            "languageId": "rust",
                            "version": 1,
                            "text": "pub fn main() {}\nstruct App {}"
                        }
                    }
                })
                .to_string(),
            )
            .await
            .unwrap();
        writer
            .write_message(
                &json!({
                    "jsonrpc": "2.0",
                    "id": 2,
                    "method": methods::DOCUMENT_SYMBOL,
                    "params": {"textDocument": {"uri": "file:///src/main.rs"}}
                })
                .to_string(),
            )
            .await
            .unwrap();
        let symbols_response: Value =
            serde_json::from_str(&reader.read_message().await.unwrap()).unwrap();
        assert_eq!(symbols_response["result"].as_array().unwrap().len(), 2);
        assert_eq!(
            symbols_response["result"][0]["data"]["schemaVersion"],
            SEMANTIC_SCHEMA_VERSION
        );

        writer
            .write_message(
                &json!({
                    "jsonrpc": "2.0",
                    "id": 3,
                    "method": methods::SHUTDOWN,
                    "params": null
                })
                .to_string(),
            )
            .await
            .unwrap();
        let shutdown_response: Value =
            serde_json::from_str(&reader.read_message().await.unwrap()).unwrap();
        assert_eq!(shutdown_response["result"], Value::Null);

        writer
            .write_message(
                &json!({
                    "jsonrpc": "2.0",
                    "method": methods::EXIT
                })
                .to_string(),
            )
            .await
            .unwrap();
        assert!(task.await.unwrap().is_ok());
    }
}
