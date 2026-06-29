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

fn json_from_success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("command emits JSON")
}

struct JsonLineSession {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
}

impl JsonLineSession {
    fn start(args: &[&str]) -> Self {
        let mut child = Command::new(assura_bin())
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("session starts");
        let stdin = child.stdin.take().expect("session stdin");
        let stdout = BufReader::new(child.stdout.take().expect("session stdout"));
        Self {
            child,
            stdin,
            stdout,
        }
    }

    fn request(&mut self, request: Value) -> Value {
        writeln!(self.stdin, "{request}").expect("write request");
        self.stdin.flush().expect("flush request");
        let mut line = String::new();
        self.stdout.read_line(&mut line).expect("read response");
        assert!(!line.is_empty(), "session emitted response");
        serde_json::from_str(&line).expect("response is JSON")
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
fn project_intelligence_supported_schemas_match_live_output() {
    let project = tempfile::tempdir().expect("tempdir");
    let project_path = project.path().to_str().expect("project path");

    let init = run_assura(&[
        "init",
        project_path,
        "--project-intelligence",
        "--no-git-hooks",
    ]);
    assert!(
        init.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&init.stdout),
        String::from_utf8_lossy(&init.stderr)
    );
    assert!(project
        .path()
        .join(".assura/models/project-intelligence/starter.schema.json")
        .is_file());

    let check = json_from_success(run_assura(&["check", project_path, "--format", "json"]));
    assert_eq!(check["success"], true);

    let agent_context = json_from_success(run_assura(&[
        "content",
        "agent-context",
        project_path,
        "--format",
        "json",
    ]));
    assert_eq!(
        agent_context["schema"],
        "assura.project-intelligence.agent-context.v1"
    );

    let context_pack = json_from_success(run_assura(&[
        "content",
        "context-pack",
        project_path,
        "--collection",
        "goals",
        "--id",
        "goal-project-intelligence-starter",
        "--text",
        "Project Intelligence",
        "--limit",
        "5",
        "--format",
        "json",
    ]));
    assert_eq!(
        context_pack["schema"],
        "assura.project-intelligence.context-pack.v1"
    );
    assert_eq!(context_pack["request"]["mode"], "object");

    let agent_query = json_from_success(run_assura(&["agent", "diagnostics", MISSING_REFERENCE]));
    assert_eq!(
        agent_query["schema"],
        "assura.project-intelligence.agent-query.v1"
    );

    let dry_run = json_from_success(run_assura(&[
        "fix",
        "markdown",
        BEACON_INVALID,
        "--dry-run",
        "--format",
        "json",
    ]));
    assert_eq!(dry_run["schema"], "assura.safe-fix.markdown.v1");
    assert_eq!(dry_run["dry_run"], true);

    let mut content_session = JsonLineSession::start(&["content", "session", project_path]);
    let content_response = content_session.request(serde_json::json!({
        "request_id": "ctx",
        "type": "context-pack",
        "collection": "goals",
        "id": "goal-project-intelligence-starter",
        "text": "Project Intelligence",
        "limit": 5
    }));
    assert_eq!(
        content_response["schema"],
        "assura.project-intelligence.session.response.v1"
    );
    assert_eq!(
        content_response["response"]["schema"],
        "assura.project-intelligence.context-pack.v1"
    );
    content_session.finish();

    let mut editor_session = JsonLineSession::start(&["editor", "session", project_path]);
    let editor_response = editor_session.request(serde_json::json!({
        "request_id": "diag",
        "method": "textDocument/diagnostics",
        "params": {
            "uri": "docs/goals/goal_project_intelligence_starter.md"
        }
    }));
    assert_eq!(
        editor_response["schema"],
        "assura.project-intelligence.editor.response.v1"
    );
    assert_eq!(editor_response["method"], "textDocument/diagnostics");
    assert_eq!(editor_response["ok"], true);
    editor_session.finish();
}

#[test]
fn documented_editor_diagnostic_example_matches_live_output_shape() {
    let demo = fs::read_to_string("website/src/content/docs/examples/project-intelligence-demo.md")
        .expect("project intelligence demo");
    assert!(demo.contains("\"method\": \"textDocument/diagnostics\""));
    assert!(demo.contains("\"schema\": \"assura.project-intelligence.editor.response.v1\""));
    assert!(demo.contains("\"code\": \"content_runtime:missing_reference\""));

    let mut editor_session = JsonLineSession::start(&["editor", "session", MISSING_REFERENCE]);
    let response = editor_session.request(serde_json::json!({
        "request_id": "diag",
        "method": "textDocument/diagnostics",
        "params": {
            "textDocument": {
                "uri": "docs/goals/goal_portable_structure.md"
            }
        }
    }));
    let diagnostic = response["result"]["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .iter()
        .find(|item| item["code"] == "content_runtime:missing_reference")
        .expect("missing-reference diagnostic");

    assert_eq!(
        response["schema"],
        "assura.project-intelligence.editor.response.v1"
    );
    assert_eq!(response["method"], "textDocument/diagnostics");
    assert_eq!(response["reload"]["state"], "initial_load");
    assert_eq!(response["ok"], true);
    assert_eq!(diagnostic["source"], "assura");
    assert_eq!(diagnostic["severity"], 1);
    assert_eq!(
        diagnostic["data"]["path"],
        "docs/goals/goal_portable_structure.md"
    );
    editor_session.finish();
}

#[test]
fn release_readiness_docs_cover_project_intelligence_surfaces() {
    let release_readiness =
        fs::read_to_string("website/src/content/docs/reference/release-readiness.md")
            .expect("release readiness docs");
    for required in [
        "assura agent",
        "assura editor session",
        "assura content session",
        "assura content context-pack",
        "assura fix markdown --apply --format json",
        ".assura/models/**",
        "assura.project-intelligence.agent-context.v1",
        "assura.project-intelligence.agent-query.v1",
        "assura.project-intelligence.context-pack.v1",
        "assura.project-intelligence.session.response.v1",
        "assura.project-intelligence.editor.response.v1",
        "MCP",
        "Full LSP server",
        "editor marketplace",
    ] {
        assert!(
            release_readiness.contains(required),
            "release readiness docs should mention {required}"
        );
    }

    let support_policy = fs::read_to_string("docs/support-policy.md").expect("support policy");
    let compatibility =
        fs::read_to_string("docs/compatibility-and-surface.md").expect("compatibility matrix");
    for supported_surface in [
        "assura agent",
        "assura editor session",
        "assura content session",
        ".assura/models/**",
    ] {
        assert!(
            support_policy.contains(supported_surface),
            "support policy missing {supported_surface}"
        );
        assert!(
            compatibility.contains(supported_surface),
            "compatibility matrix missing {supported_surface}"
        );
    }
}

#[test]
fn project_intelligence_release_hardening_audit_is_checked_in() {
    let audit = fs::read_to_string(
        Path::new("docs/analysis").join("2026-06-29-project-intelligence-usability-final-audit.md"),
    )
    .expect("final audit");
    for requirement in [
        "install/init to first useful content query",
        "starter template",
        "non-Assura project package",
        "bounded context-pack workflow",
        "warm-session path",
        "Agent and editor integrations",
        "Safe fixes",
        ".assura/",
        "schemas and support levels",
        "independent review evidence",
    ] {
        assert!(
            audit.contains(requirement),
            "final audit should cover {requirement}"
        );
    }
}
