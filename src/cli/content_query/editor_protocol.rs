//! Editor protocol envelope and LSP-shaped serialization helpers.

use super::context::{ContentQueryError, QueryContext};
use super::session::SessionReloadOutput;
use crate::intelligence::{Diagnostic, SourceLocation};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Component, Path, PathBuf};

pub(super) const EDITOR_RESPONSE_SCHEMA: &str = "assura.project-intelligence.editor.response.v1";

#[derive(Debug, Deserialize)]
pub(super) struct EditorRequest {
    #[serde(default)]
    pub(super) request_id: Option<String>,
    pub(super) method: String,
    #[serde(default)]
    pub(super) params: Value,
}

#[derive(Debug, Serialize)]
pub(super) struct EditorResponse {
    schema: &'static str,
    sequence: usize,
    request_id: Option<String>,
    method: String,
    reload: SessionReloadOutput,
    ok: bool,
    result: Option<Value>,
    error: Option<EditorErrorOutput>,
}

impl EditorResponse {
    pub(super) fn ok(
        sequence: usize,
        request_id: Option<String>,
        method: String,
        reload: SessionReloadOutput,
        result: Value,
    ) -> Self {
        Self {
            schema: EDITOR_RESPONSE_SCHEMA,
            sequence,
            request_id,
            method,
            reload,
            ok: true,
            result: Some(result),
            error: None,
        }
    }

    pub(super) fn error(
        sequence: usize,
        request_id: Option<String>,
        method: &str,
        reload: SessionReloadOutput,
        code: &'static str,
        message: String,
    ) -> Self {
        Self {
            schema: EDITOR_RESPONSE_SCHEMA,
            sequence,
            request_id,
            method: method.to_string(),
            reload,
            ok: false,
            result: None,
            error: Some(EditorErrorOutput { code, message }),
        }
    }
}

#[derive(Debug, Serialize)]
struct EditorErrorOutput {
    code: &'static str,
    message: String,
}

#[derive(Debug)]
pub(super) struct EditorRequestError {
    pub(super) code: &'static str,
    pub(super) message: String,
}

impl EditorRequestError {
    pub(super) fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl From<ContentQueryError> for EditorRequestError {
    fn from(error: ContentQueryError) -> Self {
        Self::new("request_failed", error.to_string())
    }
}

pub(super) struct DocumentTarget {
    pub(super) uri: String,
    pub(super) path: PathBuf,
}

pub(super) fn document_target(
    context: &QueryContext,
    params: &Value,
) -> Result<DocumentTarget, EditorRequestError> {
    let uri = document_uri(params).ok_or_else(|| {
        EditorRequestError::new(
            "invalid_params",
            "editor request requires a document uri or path",
        )
    })?;
    let path = path_from_uri_or_path(&context.project_root, &uri);
    Ok(DocumentTarget { uri, path })
}

pub(super) fn document_uri(params: &Value) -> Option<String> {
    params
        .pointer("/textDocument/uri")
        .or_else(|| params.pointer("/text_document/uri"))
        .or_else(|| params.get("uri"))
        .or_else(|| params.get("path"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn path_from_uri_or_path(root: &Path, uri_or_path: &str) -> PathBuf {
    let decoded = if let Some(stripped) = uri_or_path.strip_prefix("file://") {
        normalize_file_uri_path(percent_decode(stripped))
    } else {
        uri_or_path.to_string()
    };
    let without_localhost = decoded
        .strip_prefix("localhost/")
        .map(ToOwned::to_owned)
        .unwrap_or(decoded);
    let path = PathBuf::from(without_localhost);
    let relative = if path.is_absolute() {
        let absolute_root = if root.is_absolute() {
            root.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|current_dir| current_dir.join(root))
                .unwrap_or_else(|_| root.to_path_buf())
        };
        path.strip_prefix(&absolute_root)
            .unwrap_or(&path)
            .to_path_buf()
    } else {
        path
    };
    normalize_relative_path(&relative)
}

pub(super) fn diagnostic_matches_path(diagnostic: &Diagnostic, target: &Path) -> bool {
    diagnostic
        .location
        .as_ref()
        .map(|location| paths_match(&location.path, target))
        .unwrap_or(false)
}

pub(super) fn lsp_diagnostic(diagnostic: &Diagnostic) -> Value {
    let location = diagnostic
        .location
        .as_ref()
        .cloned()
        .unwrap_or_else(|| SourceLocation::path(""));
    json!({
        "range": lsp_range(&location),
        "severity": lsp_severity(&diagnostic.severity),
        "source": "assura",
        "code": diagnostic.rule,
        "message": diagnostic.message,
        "data": {
            "id": diagnostic.id.to_string(),
            "path": portable_path(&location.path),
            "field": location.field,
            "target_id": diagnostic.target_id.as_ref().map(ToString::to_string)
        }
    })
}

fn lsp_range(location: &SourceLocation) -> Value {
    let line = location.line.unwrap_or(1).saturating_sub(1);
    let character = location.column.unwrap_or(1).saturating_sub(1);
    json!({
        "start": {
            "line": line,
            "character": character
        },
        "end": {
            "line": line,
            "character": character.saturating_add(1)
        }
    })
}

fn lsp_severity(severity: &str) -> u8 {
    match severity.to_ascii_lowercase().as_str() {
        "critical" | "high" | "error" => 1,
        "medium" | "warning" | "warn" => 2,
        "low" | "info" | "information" => 3,
        "hint" => 4,
        _ => 2,
    }
}

pub(super) fn paths_match(left: &Path, right: &Path) -> bool {
    normalize_path_string(left) == normalize_path_string(right)
}

pub(super) fn portable_path(path: &Path) -> String {
    normalize_path_string(path)
}

fn normalize_relative_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

fn normalize_path_string(path: &Path) -> String {
    normalize_relative_path(path)
        .to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let (Some(high), Some(low)) =
                (hex_value(bytes[index + 1]), hex_value(bytes[index + 2]))
            {
                output.push((high << 4) | low);
                index += 3;
                continue;
            }
        }
        output.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&output).into_owned()
}

fn normalize_file_uri_path(path: String) -> String {
    if cfg!(windows) && path.len() >= 3 {
        let bytes = path.as_bytes();
        if bytes[0] == b'/' && bytes[2] == b':' && bytes[1].is_ascii_alphabetic() {
            return path[1..].to_string();
        }
    }
    path
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
