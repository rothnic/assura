use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Output, Stdio};

const MISSING_REFERENCE: &str = "tests/fixtures/content_runtime/missing_reference";
const BEACON_INVALID: &str = "tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid";

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn run_assura(args: &[&str]) -> Output {
    Command::new(assura_bin())
        .args(args)
        .output()
        .expect("assura command runs")
}

fn json_output(output: Output) -> Value {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("command emits JSON")
}

fn content_json(args: &[&str]) -> Value {
    let mut command = vec!["content"];
    command.extend_from_slice(args);
    json_output(run_assura(&command))
}

fn file_uri(path: &Path) -> String {
    let path = path.to_string_lossy().replace('\\', "/");
    if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    }
}

fn copy_dir(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create destination");
    for entry in fs::read_dir(source).expect("read source dir") {
        let entry = entry.expect("dir entry");
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir(&source_path, &destination_path);
        } else {
            fs::copy(&source_path, &destination_path).expect("copy file");
        }
    }
}

struct EditorSession {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
}

impl EditorSession {
    fn start(path: &str) -> Self {
        let mut child = Command::new(assura_bin())
            .args(["editor", "session", path])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("editor session process starts");
        let stdin = child.stdin.take().expect("editor session stdin");
        let stdout = BufReader::new(child.stdout.take().expect("editor session stdout"));
        Self {
            child,
            stdin,
            stdout,
        }
    }

    fn request(&mut self, request: Value) -> Value {
        writeln!(self.stdin, "{request}").expect("write editor request");
        self.stdin.flush().expect("flush editor request");
        let mut line = String::new();
        self.stdout
            .read_line(&mut line)
            .expect("read editor response");
        assert!(!line.is_empty(), "editor session emitted a response");
        serde_json::from_str(&line).expect("editor response is JSON")
    }

    fn finish(self) {
        drop(self.stdin);
        let output = self.child.wait_with_output().expect("session exits");
        assert!(
            output.status.success(),
            "stderr:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn editor_surface_help_exposes_local_session_protocol() {
    let output = run_assura(&["editor", "--help"]);
    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("session"));
    assert!(stdout.contains("editor integrations"));

    let session_help = run_assura(&["editor", "session", "--help"]);
    assert!(
        session_help.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&session_help.stderr)
    );
    let stdout = String::from_utf8_lossy(&session_help.stdout);
    assert!(stdout.contains("JSON-line"));
}

#[test]
fn editor_surface_returns_lsp_shaped_diagnostics_for_file() {
    let mut session = EditorSession::start(MISSING_REFERENCE);
    let response = session.request(serde_json::json!({
        "request_id": "diag-1",
        "method": "textDocument/diagnostics",
        "params": {
            "textDocument": {
                "uri": "docs/goals/goal_portable_structure.md"
            }
        }
    }));

    assert_eq!(
        response["schema"],
        "assura.project-intelligence.editor.response.v1"
    );
    assert_eq!(response["request_id"], "diag-1");
    assert_eq!(response["method"], "textDocument/diagnostics");
    assert_eq!(response["reload"]["state"], "initial_load");
    assert!(response["ok"].as_bool().expect("ok"));
    let diagnostics = response["result"]["diagnostics"]
        .as_array()
        .expect("diagnostics");
    let diagnostic = diagnostics
        .iter()
        .find(|item| item["code"] == "content_runtime:missing_reference")
        .expect("missing-reference diagnostic");
    let lower = content_json(&[
        "agent-query",
        "diagnostics",
        MISSING_REFERENCE,
        "--format",
        "json",
    ]);
    let expected = lower["response"]["diagnostics"]
        .as_array()
        .expect("lower diagnostics")
        .iter()
        .find(|item| {
            item["rule"] == "content_runtime:missing_reference"
                && item["path"] == "docs/goals/goal_portable_structure.md"
        })
        .expect("matching shared diagnostic");
    assert_eq!(diagnostic["source"], "assura");
    assert_eq!(diagnostic["code"], expected["rule"]);
    assert_eq!(diagnostic["message"], expected["message"]);
    assert_eq!(diagnostic["data"]["id"], expected["id"]);
    assert_eq!(diagnostic["severity"], 1);
    assert_eq!(diagnostic["data"]["path"], expected["path"]);
    assert_eq!(diagnostic["range"]["start"]["line"], 0);

    let absolute = fs::canonicalize(MISSING_REFERENCE)
        .expect("canonical fixture")
        .join("docs/goals/goal_portable_structure.md");
    let uri_response = session.request(serde_json::json!({
        "request_id": "diag-uri",
            "method": "textDocument/diagnostics",
            "params": {
                "textDocument": {
                "uri": file_uri(&absolute)
            }
        }
    }));
    assert_eq!(uri_response["reload"]["state"], "reused");
    assert!(uri_response["result"]["diagnostics"]
        .as_array()
        .expect("uri diagnostics")
        .iter()
        .any(|item| item["data"]["id"] == expected["id"]));

    session.finish();
}

#[test]
fn editor_surface_context_infers_modeled_object_from_file() {
    let mut session = EditorSession::start(BEACON_INVALID);
    let response = session.request(serde_json::json!({
        "request_id": "context-1",
        "method": "textDocument/context",
        "params": {
            "uri": "docs/epics/epic_checkout.md",
            "text": "checkout",
            "limit": 5
        }
    }));

    assert!(response["ok"].as_bool().expect("ok"));
    let pack = &response["result"]["context_pack"];
    let expected = content_json(&[
        "context-pack",
        BEACON_INVALID,
        "--collection",
        "epics",
        "--id",
        "epic-checkout",
        "--text",
        "checkout",
        "--limit",
        "5",
        "--format",
        "json",
    ]);
    assert_eq!(pack, &expected);
    assert_eq!(
        pack["schema"],
        "assura.project-intelligence.context-pack.v1"
    );
    assert_eq!(pack["request"]["mode"], "object");
    assert_eq!(pack["request"]["collection"], "epics");
    assert_eq!(pack["request"]["id"], "epic-checkout");
    assert_eq!(pack["instance"]["path"], "docs/epics/epic_checkout.md");
    assert!(pack["missing_relations"]
        .as_array()
        .expect("missing relations")
        .iter()
        .any(|item| item["target_instance_id"] == "adr-missing-payment-risk"));

    session.finish();
}

#[test]
fn editor_surface_code_actions_preview_safe_fixes_without_writes() {
    let temp = tempfile::tempdir().expect("tempdir");
    copy_dir(Path::new(BEACON_INVALID), temp.path());
    let epic_path = temp.path().join("docs/epics/epic_checkout.md");
    let original = fs::read_to_string(&epic_path).expect("epic markdown");
    let drifted = original.replace("# Checkout Onboarding\n\n", "# Checkout Onboarding\n   \n");
    fs::write(&epic_path, &drifted).expect("write deterministic markdown drift");

    let path = temp.path().to_str().expect("temp path");
    let mut session = EditorSession::start(path);
    let response = session.request(serde_json::json!({
        "request_id": "code-action-1",
        "method": "textDocument/codeAction",
        "params": {
            "uri": "docs/epics/epic_checkout.md"
        }
    }));

    assert!(response["ok"].as_bool().expect("ok"));
    let actions = response["result"]["code_actions"]
        .as_array()
        .expect("code actions");
    let action = actions.first().expect("safe-fix code action");
    let safe_fixes = content_json(&["agent-query", "safe-fixes", path, "--format", "json"]);
    let expected = safe_fixes["response"]["safe_fixes"][0].clone();
    let dry_run = json_output(run_assura(&[
        "fix",
        "markdown",
        path,
        "--dry-run",
        "--format",
        "json",
    ]));
    assert_eq!(action["kind"], "quickfix");
    assert_eq!(action["isPreferred"], false);
    assert!(action.get("command").is_none());
    assert!(action.get("edit").is_none());
    assert_eq!(action["data"]["safe_fix_id"], expected["id"]);
    assert_eq!(action["data"]["diagnostic_id"], expected["diagnostic_id"]);
    assert_eq!(action["data"]["operation"], expected["operation"]);
    assert_eq!(action["data"]["audit_id"], expected["audit_id"]);
    assert_eq!(action["data"]["audit_id"], dry_run["fixes"][0]["id"]);
    assert_eq!(
        action["data"]["apply_command"],
        "assura fix markdown --apply --format json"
    );
    assert_eq!(
        fs::read_to_string(&epic_path).expect("epic markdown remains unchanged"),
        drifted
    );

    session.finish();
}

#[test]
fn editor_surface_reports_invalid_methods_and_conservative_reload() {
    let temp = tempfile::tempdir().expect("tempdir");
    copy_dir(Path::new(BEACON_INVALID), temp.path());
    let path = temp.path().to_str().expect("temp path");
    let mut session = EditorSession::start(path);

    let invalid = session.request(serde_json::json!({
        "request_id": "bad-1",
        "method": "workspace/executeCommand",
        "params": {}
    }));
    assert_eq!(invalid["ok"], false);
    assert_eq!(invalid["error"]["code"], "unsupported_method");

    let epic_path = temp.path().join("docs/epics/epic_checkout.md");
    let mut content = fs::read_to_string(&epic_path).expect("epic markdown");
    content.push('\n');
    fs::write(&epic_path, content).expect("touch modeled content");

    let reloaded = session.request(serde_json::json!({
        "request_id": "diag-2",
        "method": "textDocument/diagnostics",
        "params": {
            "uri": "docs/epics/epic_checkout.md"
        }
    }));
    assert_eq!(reloaded["ok"], true);
    assert_eq!(reloaded["reload"]["state"], "reloaded");

    session.finish();
}
