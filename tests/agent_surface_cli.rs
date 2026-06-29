use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Output, Stdio};

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

fn agent_json(args: &[&str]) -> Value {
    let mut command = vec!["agent"];
    command.extend_from_slice(args);
    json_output(run_assura(&command))
}

fn content_json(args: &[&str]) -> Value {
    let mut command = vec!["content"];
    command.extend_from_slice(args);
    json_output(run_assura(&command))
}

fn copy_dir(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create destination");
    for entry in fs::read_dir(source).expect("read fixture dir") {
        let entry = entry.expect("fixture entry");
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir(&source_path, &destination_path);
        } else {
            fs::copy(&source_path, &destination_path).expect("copy file");
        }
    }
}

struct SessionProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
}

impl SessionProcess {
    fn start(args: &[&str]) -> Self {
        let mut child = Command::new(assura_bin())
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("session process starts");
        let stdin = child.stdin.take().expect("session stdin");
        let stdout = BufReader::new(child.stdout.take().expect("session stdout"));
        Self {
            child,
            stdin,
            stdout,
        }
    }

    fn request(&mut self, request: Value) -> Value {
        writeln!(self.stdin, "{request}").expect("write session request");
        self.stdin.flush().expect("flush session request");
        let mut line = String::new();
        self.stdout
            .read_line(&mut line)
            .expect("read session response");
        assert!(!line.is_empty(), "session emitted a response");
        serde_json::from_str(&line).expect("session response is JSON")
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
fn agent_surface_help_exposes_local_project_intelligence_commands() {
    let output = run_assura(&["agent", "--help"]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("diagnostics"));
    assert!(stdout.contains("context-pack"));
    assert!(stdout.contains("safe-fixes"));
    assert!(stdout.contains("missing-relations"));
}

#[test]
fn agent_surface_session_alias_reuses_json_line_session_contract() {
    let mut session = SessionProcess::start(&[
        "agent",
        "session",
        "tests/fixtures/content_runtime/missing_reference",
    ]);
    let response = session.request(serde_json::json!({
        "request_id": "diagnostics",
        "type": "diagnostics"
    }));

    assert_eq!(
        response["schema"],
        "assura.project-intelligence.session.response.v1"
    );
    assert_eq!(response["request_id"], "diagnostics");
    assert_eq!(response["reload"]["state"], "initial_load");
    assert!(response["ok"].as_bool().expect("ok"));
    assert!(response["response"]["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .iter()
        .any(|item| item["rule"] == "content_runtime:missing_reference"));

    session.finish();
}

#[test]
fn agent_surface_defaults_to_json_and_reuses_content_contracts() {
    let context = agent_json(&["context", "tests/fixtures/content_runtime/code_symbols"]);
    assert_eq!(
        context["schema"],
        "assura.project-intelligence.agent-context.v1"
    );
    assert_eq!(
        context,
        content_json(&[
            "agent-context",
            "tests/fixtures/content_runtime/code_symbols",
            "--format",
            "json",
        ])
    );

    assert_eq!(
        agent_json(&["search", "Portable", "tests/fixtures/content_runtime/valid",]),
        content_json(&[
            "search",
            "Portable",
            "tests/fixtures/content_runtime/valid",
            "--format",
            "json",
        ])
    );

    assert_eq!(
        agent_json(&[
            "show",
            "goals",
            "goal-portable-structure",
            "tests/fixtures/content_runtime/valid",
        ]),
        content_json(&[
            "show",
            "goals",
            "goal-portable-structure",
            "tests/fixtures/content_runtime/valid",
            "--format",
            "json",
        ])
    );
}

#[test]
fn agent_surface_wraps_diagnostics_graph_relations_and_safe_fixes() {
    assert_eq!(
        agent_json(&[
            "diagnostics",
            "tests/fixtures/content_runtime/missing_reference",
        ]),
        content_json(&[
            "agent-query",
            "diagnostics",
            "tests/fixtures/content_runtime/missing_reference",
            "--format",
            "json",
        ])
    );

    assert_eq!(
        agent_json(&[
            "expand",
            "goals",
            "goal-portable-structure",
            "tests/fixtures/content_runtime/valid",
        ]),
        content_json(&[
            "expand",
            "goals",
            "goal-portable-structure",
            "tests/fixtures/content_runtime/valid",
            "--format",
            "json",
        ])
    );

    assert_eq!(
        agent_json(&[
            "missing-relations",
            "tests/fixtures/content_runtime/missing_reference",
        ]),
        content_json(&[
            "missing-relations",
            "tests/fixtures/content_runtime/missing_reference",
            "--format",
            "json",
        ])
    );

    let temp = tempfile::tempdir().expect("tempdir");
    copy_dir(
        Path::new("tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid"),
        temp.path(),
    );
    let epic_path = temp.path().join("docs/epics/epic_checkout.md");
    let drifted = fs::read_to_string(&epic_path)
        .expect("epic markdown")
        .replace("# Checkout Onboarding\n\n", "# Checkout Onboarding\n   \n");
    fs::write(&epic_path, drifted).expect("write deterministic markdown drift");
    let path = temp.path().to_str().expect("temp path");

    let dry_run = json_output(run_assura(&[
        "fix",
        "markdown",
        path,
        "--dry-run",
        "--format",
        "json",
    ]));
    let safe_fixes = agent_json(&["safe-fixes", path]);
    assert_eq!(
        safe_fixes,
        content_json(&["agent-query", "safe-fixes", path, "--format", "json"])
    );
    assert_eq!(
        safe_fixes["response"]["safe_fixes"][0]["audit_id"],
        dry_run["fixes"][0]["id"]
    );
}

#[test]
fn agent_surface_context_pack_reuses_bounded_handoff_contract() {
    assert_eq!(
        agent_json(&[
            "context-pack",
            "tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid",
            "--collection",
            "epics",
            "--id",
            "epic-checkout",
            "--text",
            "checkout",
            "--limit",
            "5",
        ]),
        content_json(&[
            "context-pack",
            "tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid",
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
        ])
    );
}
