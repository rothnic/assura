//! Local JSON-line editor protocol over project-intelligence facts.

use super::agent_query::safe_fixes;
use super::context::{ContentQueryError, QueryContext};
use super::context_pack::{context_pack, ContextPackRequest};
use super::editor_protocol::{
    diagnostic_matches_path, document_target, lsp_diagnostic, paths_match, DocumentTarget,
    EditorRequest, EditorRequestError, EditorResponse,
};
use super::facts::resources_by_id;
use super::session::{ProjectFingerprint, SessionReloadOutput};
use crate::cli::ExitCode;
use crate::intelligence::ProjectFact;
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

pub(crate) fn editor_session_command(path: Option<PathBuf>, config: Option<PathBuf>) -> ExitCode {
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
    let mut session = match EditorSession::load(path, config) {
        Ok(session) => session,
        Err(error) => {
            eprintln!("Error: {error}");
            return error.exit_code;
        }
    };
    if let Ok(ready_file) = std::env::var("ASSURA_EDITOR_SESSION_READY_FILE") {
        if let Err(error) = fs::write(&ready_file, "ready\n") {
            eprintln!("Error: failed to write editor session ready file {ready_file}: {error}");
            return ExitCode::RuntimeError;
        }
    }

    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(error) => {
                eprintln!("Error: failed to read editor session request: {error}");
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
                    eprintln!("Error: failed to write editor session response");
                    return ExitCode::RuntimeError;
                }
            }
            Err(error) => {
                eprintln!("Error: failed to serialize editor session response: {error}");
                return ExitCode::RuntimeError;
            }
        }
    }

    ExitCode::Success
}

struct EditorSession {
    path: PathBuf,
    config: Option<PathBuf>,
    context: QueryContext,
    fingerprint: ProjectFingerprint,
    sequence: usize,
    first_response: bool,
}

impl EditorSession {
    fn load(path: PathBuf, config: Option<PathBuf>) -> Result<Self, ContentQueryError> {
        let context = QueryContext::load_for_path(path.clone(), config.clone(), false, true)?;
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

    fn handle_line(&mut self, line: &str) -> EditorResponse {
        self.sequence += 1;
        let request = match serde_json::from_str::<EditorRequest>(line) {
            Ok(request) => request,
            Err(error) => {
                return EditorResponse::error(
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
                return EditorResponse::error(
                    self.sequence,
                    request.request_id.clone(),
                    request.method.as_str(),
                    SessionReloadOutput::failed(),
                    "reload_failed",
                    error.to_string(),
                );
            }
        };

        match self.run_request(&request) {
            Ok(result) => EditorResponse::ok(
                self.sequence,
                request.request_id,
                request.method,
                reload,
                result,
            ),
            Err(error) => EditorResponse::error(
                self.sequence,
                request.request_id,
                request.method.as_str(),
                reload,
                error.code,
                error.message,
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
            self.context =
                QueryContext::load_for_path(self.path.clone(), self.config.clone(), false, true)?;
            self.fingerprint = ProjectFingerprint::capture(&self.context.project_root)?;
            return Ok(SessionReloadOutput::reloaded(&self.context));
        }
        if latest == self.fingerprint {
            return Ok(SessionReloadOutput::reused(&self.context));
        }

        self.context =
            QueryContext::load_for_path(self.path.clone(), self.config.clone(), false, true)?;
        self.fingerprint = ProjectFingerprint::capture(&self.context.project_root)?;
        Ok(SessionReloadOutput::reloaded(&self.context))
    }

    fn run_request(&self, request: &EditorRequest) -> Result<Value, EditorRequestError> {
        match request.method.as_str() {
            "textDocument/diagnostics" => self.diagnostics(&request.params),
            "textDocument/context" => self.context_pack(&request.params),
            "textDocument/codeAction" => self.code_actions(&request.params),
            other => Err(EditorRequestError::new(
                "unsupported_method",
                format!("unsupported editor method: {other}"),
            )),
        }
    }

    fn diagnostics(&self, params: &Value) -> Result<Value, EditorRequestError> {
        let target = self.document_target(params)?;
        let diagnostics = self
            .context
            .store
            .facts()
            .facts
            .iter()
            .filter_map(|fact| match fact {
                ProjectFact::Diagnostic(diagnostic)
                    if diagnostic_matches_path(diagnostic, &target.path) =>
                {
                    Some(lsp_diagnostic(diagnostic))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        Ok(json!({
            "uri": target.uri,
            "path": target.path,
            "diagnostics": diagnostics,
            "source": "assura"
        }))
    }

    fn context_pack(&self, params: &Value) -> Result<Value, EditorRequestError> {
        let target = self.document_target(params)?;
        let limit = params
            .get("limit")
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or(20);
        let text = params
            .get("text")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| target.path.display().to_string());
        let explicit_collection = params
            .get("collection")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let explicit_id = params
            .get("id")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let inferred = match (&explicit_collection, &explicit_id) {
            (Some(_), Some(_)) => None,
            (None, None) => self.instance_for_path(&target.path),
            _ => {
                return Err(EditorRequestError::new(
                    "invalid_params",
                    "textDocument/context requires collection and id together",
                ));
            }
        };
        let collection = explicit_collection.or_else(|| {
            inferred
                .as_ref()
                .map(|(collection, _)| collection.to_string())
        });
        let id = explicit_id.or_else(|| inferred.as_ref().map(|(_, id)| id.to_string()));
        let pack = context_pack(
            &self.context,
            ContextPackRequest {
                collection: collection.as_ref(),
                id: id.as_ref(),
                text: Some(&text),
                limit,
            },
        )?;
        Ok(json!({
            "uri": target.uri,
            "path": target.path,
            "context_pack": to_value(pack)?
        }))
    }

    fn code_actions(&self, params: &Value) -> Result<Value, EditorRequestError> {
        let target = self.document_target(params)?;
        let actions = safe_fixes(&self.context)
            .safe_fixes
            .into_iter()
            .filter(|fix| {
                fix.path
                    .as_ref()
                    .map(|path| paths_match(path, &target.path))
                    .unwrap_or(false)
            })
            .map(|fix| {
                json!({
                    "title": format!("Preview Assura safe fix: {}", fix.summary),
                    "kind": "quickfix",
                    "isPreferred": false,
                    "diagnostics": [],
                    "data": {
                        "safe_fix_id": fix.id,
                        "audit_id": fix.audit_id,
                        "diagnostic_id": fix.diagnostic_id,
                        "target_id": fix.target_id,
                        "operation": fix.operation,
                        "summary": fix.summary,
                        "path": fix.path,
                        "line": fix.line,
                        "column": fix.column,
                        "field": fix.field,
                        "apply_command": "assura fix markdown --apply --format json"
                    }
                })
            })
            .collect::<Vec<_>>();
        Ok(json!({
            "uri": target.uri,
            "path": target.path,
            "code_actions": actions
        }))
    }

    fn document_target(&self, params: &Value) -> Result<DocumentTarget, EditorRequestError> {
        document_target(&self.context, params)
    }

    fn instance_for_path(&self, path: &Path) -> Option<(String, String)> {
        let resources = resources_by_id(self.context.store.facts());
        self.context
            .store
            .facts()
            .facts
            .iter()
            .find_map(|fact| match fact {
                ProjectFact::ModelInstance(instance) => {
                    let resource = resources.get(&instance.resource_id)?;
                    if paths_match(&resource.path, path) {
                        Some((instance.collection.clone(), instance.instance_id.clone()))
                    } else {
                        None
                    }
                }
                _ => None,
            })
    }
}

fn to_value<T: Serialize>(value: T) -> Result<Value, ContentQueryError> {
    serde_json::to_value(value).map_err(|error| ContentQueryError::runtime(error.to_string()))
}
