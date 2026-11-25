//! Persistent JSON-line project-intelligence query sessions.

use super::agent_query::{diagnostics, safe_fixes};
use super::context::ContentQueryError;
use super::context_pack::{context_pack, ContextPackRequest};
use super::session_context::{SessionContext, SessionReloadOutput};
use super::{collections, expand, missing_relations, search};
use crate::cli::ExitCode;
use crate::intelligence::project_intelligence_agent_context;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

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
    project: SessionContext,
    sequence: usize,
}

impl ContentSession {
    fn load(path: PathBuf, config: Option<PathBuf>) -> Result<Self, ContentQueryError> {
        Ok(Self {
            project: SessionContext::load(path, config)?,
            sequence: 0,
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
        let reload = match self.project.ensure_fresh() {
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

    fn run_request(&self, request: &SessionRequest) -> Result<Value, ContentQueryError> {
        let context = self.project.context();
        match request.request_type.as_str() {
            "agent-context" => to_value(project_intelligence_agent_context(context.store.facts())),
            "collections" => to_value(collections(context)),
            "context-pack" => {
                let limit = request.limit.unwrap_or(20);
                to_value(context_pack(
                    context,
                    ContextPackRequest {
                        collection: request.collection.as_ref(),
                        id: request.id.as_ref(),
                        text: request.text.as_ref(),
                        limit,
                    },
                )?)
            }
            "diagnostics" => to_value(diagnostics(context)),
            "expand" => {
                let collection = request.required("collection")?;
                let id = request.required("id")?;
                to_value(expand(
                    context,
                    collection,
                    id,
                    request.limit.unwrap_or(20),
                )?)
            }
            "missing-relations" => to_value(missing_relations(context)),
            "safe-fixes" => to_value(safe_fixes(context)),
            "search" => {
                let text = request.required("text")?;
                to_value(search(context, text, request.limit.unwrap_or(20)))
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
struct SessionErrorOutput {
    code: &'static str,
    message: String,
}

fn to_value<T: Serialize>(value: T) -> Result<Value, ContentQueryError> {
    serde_json::to_value(value).map_err(|error| ContentQueryError::runtime(error.to_string()))
}
