//! Persistent JSON-line project-intelligence query sessions.

use super::agent_query::{diagnostics, safe_fixes};
use super::context::{ContentQueryError, QueryContext};
use super::context_pack::{context_pack, ContextPackRequest};
use super::{collections, expand, missing_relations, search};
use crate::cli::ExitCode;
use crate::intelligence::project_intelligence_agent_context;
use crate::stable_hash::stable_hash;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

const SESSION_RESPONSE_SCHEMA: &str = "assura.project-intelligence.session.response.v1";

pub(super) fn content_session_command(path: Option<PathBuf>, config: Option<PathBuf>) -> ExitCode {
    let path = match path {
        Some(path) => path,
        None => match std::env::current_dir() {
            Ok(path) => path,
            Err(error) => {
                eprintln!("Error: failed to read current directory: {error}");
                return ExitCode::RuntimeError;
            }
        },
    };
    let mut session = match ContentSession::load(path, config) {
        Ok(session) => session,
        Err(error) => {
            eprintln!("Error: {error}");
            return error.exit_code;
        }
    };
    if let Ok(ready_file) = std::env::var("ASSURA_CONTENT_SESSION_READY_FILE") {
        if let Err(error) = fs::write(&ready_file, "ready\n") {
            eprintln!("Error: failed to write session ready file {ready_file}: {error}");
            return ExitCode::RuntimeError;
        }
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("Error: failed to read session request: {error}");
                return ExitCode::RuntimeError;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let response = session.handle_line(&line);
        match serde_json::to_string(&response) {
            Ok(rendered) => {
                if writeln!(stdout, "{rendered}").is_err() || stdout.flush().is_err() {
                    eprintln!("Error: failed to write session response");
                    return ExitCode::RuntimeError;
                }
            }
            Err(error) => {
                eprintln!("Error: failed to serialize session response: {error}");
                return ExitCode::RuntimeError;
            }
        }
    }

    ExitCode::Success
}

struct ContentSession {
    path: PathBuf,
    config: Option<PathBuf>,
    context: QueryContext,
    fingerprint: ProjectFingerprint,
    sequence: usize,
    first_response: bool,
}

impl ContentSession {
    fn load(path: PathBuf, config: Option<PathBuf>) -> Result<Self, ContentQueryError> {
        let context = QueryContext::load_for_path(path.clone(), config.clone(), false, true, true)?;
        let fingerprint = ProjectFingerprint::capture(&context.project_root)?;
        Ok(Self {
            path,
            config,
            context,
            fingerprint,
            sequence: 0,
            first_response: true,
        })
    }

    fn handle_line(&mut self, line: &str) -> SessionResponse {
        self.sequence += 1;
        let parsed = serde_json::from_str::<SessionRequest>(line);
        let request = match parsed {
            Ok(request) => request,
            Err(error) => {
                return SessionResponse::error(
                    self.sequence,
                    None,
                    "invalid",
                    SessionReloadOutput::not_checked(),
                    "invalid_request",
                    format!("failed to parse JSON request: {error}"),
                );
            }
        };
        let reload = match self.ensure_fresh() {
            Ok(reload) => reload,
            Err(error) => {
                return SessionResponse::error(
                    self.sequence,
                    request.request_id.clone(),
                    request.request_type.as_str(),
                    SessionReloadOutput::failed(),
                    "reload_failed",
                    error.to_string(),
                );
            }
        };

        match self.run_request(&request) {
            Ok(response) => SessionResponse::ok(
                self.sequence,
                request.request_id,
                request.request_type,
                reload,
                response,
            ),
            Err(error) => SessionResponse::error(
                self.sequence,
                request.request_id,
                request.request_type.as_str(),
                reload,
                "request_failed",
                error.to_string(),
            ),
        }
    }

    fn ensure_fresh(&mut self) -> Result<SessionReloadOutput, ContentQueryError> {
        let latest = ProjectFingerprint::capture(&self.context.project_root)?;
        if self.first_response {
            self.first_response = false;
            if latest == self.fingerprint {
                return Ok(SessionReloadOutput::initial(&self.context));
            }
            self.context = QueryContext::load_for_path(
                self.path.clone(),
                self.config.clone(),
                false,
                true,
                true,
            )?;
            self.fingerprint = ProjectFingerprint::capture(&self.context.project_root)?;
            return Ok(SessionReloadOutput::reloaded(&self.context));
        }
        if latest == self.fingerprint {
            return Ok(SessionReloadOutput::reused(&self.context));
        }

        self.context =
            QueryContext::load_for_path(self.path.clone(), self.config.clone(), false, true, true)?;
        self.fingerprint = ProjectFingerprint::capture(&self.context.project_root)?;
        Ok(SessionReloadOutput::reloaded(&self.context))
    }

    fn run_request(&self, request: &SessionRequest) -> Result<Value, ContentQueryError> {
        match request.request_type.as_str() {
            "agent-context" => to_value(project_intelligence_agent_context(
                self.context.store.facts(),
            )),
            "collections" => to_value(collections(&self.context)),
            "context-pack" => {
                let limit = request.limit.unwrap_or(20);
                to_value(context_pack(
                    &self.context,
                    ContextPackRequest {
                        collection: request.collection.as_ref(),
                        id: request.id.as_ref(),
                        text: request.text.as_ref(),
                        limit,
                    },
                )?)
            }
            "diagnostics" => to_value(diagnostics(&self.context)),
            "expand" => {
                let collection = request.required("collection")?;
                let id = request.required("id")?;
                to_value(expand(
                    &self.context,
                    collection,
                    id,
                    request.limit.unwrap_or(20),
                )?)
            }
            "missing-relations" => to_value(missing_relations(&self.context)),
            "safe-fixes" => to_value(safe_fixes(&self.context)),
            "search" => {
                let text = request.required("text")?;
                to_value(search(&self.context, text))
            }
            other => Err(ContentQueryError::configuration(format!(
                "unsupported session request type: {other}"
            ))),
        }
    }
}

#[derive(Debug, Deserialize)]
struct SessionRequest {
    #[serde(default)]
    request_id: Option<String>,
    #[serde(rename = "type")]
    request_type: String,
    #[serde(default)]
    collection: Option<String>,
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
}

impl SessionRequest {
    fn required(&self, field: &str) -> Result<&str, ContentQueryError> {
        match field {
            "collection" => self.collection.as_deref(),
            "id" => self.id.as_deref(),
            "text" => self.text.as_deref(),
            _ => None,
        }
        .ok_or_else(|| {
            ContentQueryError::configuration(format!(
                "{} request requires `{field}`",
                self.request_type
            ))
        })
    }
}

#[derive(Debug, Serialize)]
struct SessionResponse {
    schema: &'static str,
    sequence: usize,
    request_id: Option<String>,
    request_type: String,
    reload: SessionReloadOutput,
    ok: bool,
    response: Option<Value>,
    error: Option<SessionErrorOutput>,
}

impl SessionResponse {
    fn ok(
        sequence: usize,
        request_id: Option<String>,
        request_type: String,
        reload: SessionReloadOutput,
        response: Value,
    ) -> Self {
        Self {
            schema: SESSION_RESPONSE_SCHEMA,
            sequence,
            request_id,
            request_type,
            reload,
            ok: true,
            response: Some(response),
            error: None,
        }
    }

    fn error(
        sequence: usize,
        request_id: Option<String>,
        request_type: &str,
        reload: SessionReloadOutput,
        code: &'static str,
        message: String,
    ) -> Self {
        Self {
            schema: SESSION_RESPONSE_SCHEMA,
            sequence,
            request_id,
            request_type: request_type.to_string(),
            reload,
            ok: false,
            response: None,
            error: Some(SessionErrorOutput { code, message }),
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct SessionReloadOutput {
    pub(super) state: &'static str,
    pub(super) reason: &'static str,
    pub(super) project_root: Option<PathBuf>,
    pub(super) config_path: Option<PathBuf>,
}

impl SessionReloadOutput {
    pub(super) fn initial(context: &QueryContext) -> Self {
        Self::from_context("initial_load", "session context loaded", context)
    }

    pub(super) fn reused(context: &QueryContext) -> Self {
        Self::from_context("reused", "project fingerprint unchanged", context)
    }

    pub(super) fn reloaded(context: &QueryContext) -> Self {
        Self::from_context(
            "reloaded",
            "project fingerprint changed; context rebuilt",
            context,
        )
    }

    pub(super) fn failed() -> Self {
        Self {
            state: "reload_failed",
            reason: "project fingerprint changed but context could not be rebuilt",
            project_root: None,
            config_path: None,
        }
    }

    pub(super) fn not_checked() -> Self {
        Self {
            state: "not_checked",
            reason: "request did not parse",
            project_root: None,
            config_path: None,
        }
    }

    fn from_context(state: &'static str, reason: &'static str, context: &QueryContext) -> Self {
        Self {
            state,
            reason,
            project_root: Some(context.project_root.clone()),
            config_path: Some(context.config_path.clone()),
        }
    }
}

#[derive(Debug, Serialize)]
struct SessionErrorOutput {
    code: &'static str,
    message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProjectFingerprint {
    entries: BTreeMap<PathBuf, FileFingerprint>,
}

impl ProjectFingerprint {
    pub(super) fn capture(root: &Path) -> Result<Self, ContentQueryError> {
        let mut entries = BTreeMap::new();
        for entry in WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|entry| !is_ignored_fingerprint_entry(entry))
        {
            let entry = entry.map_err(|error| {
                ContentQueryError::runtime(format!("failed to scan project fingerprint: {error}"))
            })?;
            let path = entry.path();
            let metadata = entry.metadata().map_err(|error| {
                ContentQueryError::runtime(format!(
                    "failed to read metadata for {}: {error}",
                    path.display()
                ))
            })?;
            let relative = path.strip_prefix(root).unwrap_or(path).to_path_buf();
            let fingerprint = FileFingerprint::from_path(path, &metadata)?;
            entries.insert(relative, fingerprint);
        }
        Ok(Self { entries })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileFingerprint {
    content_hash: Option<u64>,
    len: u64,
    is_dir: bool,
}

impl FileFingerprint {
    fn from_path(path: &Path, metadata: &std::fs::Metadata) -> Result<Self, ContentQueryError> {
        let content_hash = if metadata.is_file() {
            let content = fs::read(path).map_err(|error| {
                ContentQueryError::runtime(format!(
                    "failed to read fingerprint content for {}: {error}",
                    path.display()
                ))
            })?;
            Some(stable_hash(&content))
        } else {
            None
        };
        Ok(Self {
            content_hash,
            len: metadata.len(),
            is_dir: metadata.is_dir(),
        })
    }
}

fn is_ignored_fingerprint_entry(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_string_lossy();
    matches!(
        name.as_ref(),
        ".git" | "target" | "node_modules" | "dist" | ".astro" | ".next"
    )
}

fn to_value<T: Serialize>(value: T) -> Result<Value, ContentQueryError> {
    serde_json::to_value(value).map_err(|error| ContentQueryError::runtime(error.to_string()))
}
