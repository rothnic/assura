use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Output, Stdio};
use tempfile::TempDir;

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

fn nudge_fixture() -> TempDir {
    let project = tempfile::tempdir().expect("temp project");
    fs::create_dir_all(project.path().join(".assura")).expect("create config dir");
    fs::create_dir_all(project.path().join("src")).expect("create src");
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"
structure:
  ./:
    extra: true
    children:
      src/:
        files:
          naming: kebab-case
          extensions: ["rs"]
exclude:
  - target/**
"#,
    )
    .expect("write config");
    fs::write(project.path().join("src/BadName.rs"), "fn main() {}\n").expect("write bad file");
    fs::write(project.path().join("src/AnotherBad.rs"), "fn helper() {}\n")
        .expect("write second bad file");
    fs::write(
        project.path().join("src/BadName.js"),
        "console.log('bad');\n",
    )
    .expect("write multi-violation file");
    project
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
    assert!(stdout.contains("nudge"));
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
fn agent_nudge_session_start_is_compact_and_cache_stable() {
    let project = nudge_fixture();
    let path = project.path().to_str().expect("fixture path");

    let nudge = agent_json(&[
        "nudge",
        path,
        "--event",
        "session-start",
        "--agent",
        "claude",
    ]);

    assert_eq!(nudge["schema"], "assura.agent-nudge.v1");
    assert_eq!(nudge["target_agent"], "claude");
    assert_eq!(nudge["event"], "session_start");
    assert_eq!(nudge["cache_policy"]["stable_by_default"], true);
    assert_eq!(
        nudge["cache_policy"]["volatile_fields"],
        serde_json::json!([])
    );
    assert_eq!(nudge["summary"]["should_inject"], false);
    assert_eq!(nudge["summary"]["nudge_count"], 0);
    assert_eq!(nudge["daemon"]["state"], "running");
    assert!(nudge["summary"]["suggested_command"]
        .as_str()
        .expect("suggested command")
        .contains("assura check --format agent --warn"));
}

#[test]
fn agent_nudge_after_tool_reports_bounded_changed_path_findings() {
    let project = nudge_fixture();
    let path = project.path().to_str().expect("fixture path");

    let nudge = agent_json(&[
        "nudge",
        path,
        "--event",
        "after-tool",
        "--changed",
        "src/BadName.rs",
        "--changed",
        "src/cli/check/rules.rs",
        "--agent",
        "codex",
        "--max-issues",
        "1",
    ]);

    assert_eq!(nudge["schema"], "assura.agent-nudge.v1");
    assert_eq!(nudge["target_agent"], "codex");
    assert_eq!(nudge["event"], "after_tool");
    assert_eq!(nudge["summary"]["should_inject"], true);
    assert_eq!(nudge["summary"]["nudge_count"], 1);
    assert_eq!(nudge["summary"]["affected_rules"][0], "file_naming");
    assert_eq!(nudge["summary"]["omitted_count"], 1);
    assert_eq!(nudge["reference_contexts"].as_array().unwrap().len(), 0);
    assert_eq!(nudge["changed_path_checks"].as_array().unwrap().len(), 1);
    assert_eq!(nudge["changed_path_checks"][0]["path"], "src/BadName.rs");
    assert_eq!(nudge["nudges"][0]["category"], "structure");
    assert_eq!(nudge["nudges"][0]["rule"], "file_naming");
    assert_eq!(nudge["nudges"][0]["severity"], "medium");
    assert!(nudge["nudges"][0]["suggested_command"]
        .as_str()
        .expect("suggested command")
        .contains("--agent codex"));
    assert!(nudge["summary"]["suggested_command"]
        .as_str()
        .expect("suggested command")
        .contains("--agent codex"));
}

#[test]
fn agent_nudge_omitted_count_includes_findings_hidden_by_max_issues() {
    let project = nudge_fixture();
    let path = project.path().to_str().expect("fixture path");

    let nudge = agent_json(&[
        "nudge",
        path,
        "--event",
        "after-tool",
        "--changed",
        "src/BadName.js",
        "--max-issues",
        "1",
    ]);

    assert_eq!(nudge["summary"]["nudge_count"], 1);
    assert_eq!(nudge["summary"]["omitted_count"], 1);
    assert_eq!(nudge["nudges"].as_array().unwrap().len(), 1);
}

#[test]
fn agent_nudge_reports_daemon_fallback_for_unavailable_project() {
    let project = tempfile::tempdir().expect("temp project");
    let path = project.path().to_str().expect("fixture path");

    let nudge = agent_json(&[
        "nudge",
        path,
        "--event",
        "before-tool",
        "--agent",
        "opencode",
    ]);

    assert_eq!(nudge["schema"], "assura.agent-nudge.v1");
    assert_eq!(nudge["target_agent"], "opencode");
    assert_eq!(nudge["daemon"]["state"], "unavailable");
    assert_eq!(nudge["summary"]["should_inject"], true);
    assert_eq!(nudge["nudges"][0]["category"], "daemon");
    assert_eq!(nudge["nudges"][0]["rule"], "daemon_health");
    assert!(nudge["daemon"]["fallback_command"]
        .as_str()
        .expect("fallback command")
        .contains("assura check --format agent"));
    assert!(nudge["nudges"][0]["suggested_command"]
        .as_str()
        .expect("suggested command")
        .contains("assura check --format agent"));
}

#[test]
fn agent_nudge_marks_performance_gate_relevant_paths() {
    let project = nudge_fixture();
    let path = project.path().to_str().expect("fixture path");

    let nudge = agent_json(&[
        "nudge",
        path,
        "--event",
        "after-tool",
        "--changed",
        "src/cli/check/rules.rs",
        "--agent",
        "pi",
    ]);

    assert_eq!(nudge["target_agent"], "pi");
    assert!(nudge["nudges"]
        .as_array()
        .expect("nudges")
        .iter()
        .any(|item| item["rule"] == "performance_no_slower"));
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
