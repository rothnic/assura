use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
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

fn git_command(project: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(project)
        .env("GIT_AUTHOR_NAME", "Assura Test")
        .env("GIT_AUTHOR_EMAIL", "assura-test@example.com")
        .env("GIT_COMMITTER_NAME", "Assura Test")
        .env("GIT_COMMITTER_EMAIL", "assura-test@example.com")
        .output()
        .expect("git command runs");
    assert!(
        output.status.success(),
        "git {:?}\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_nudge_fixture() -> TempDir {
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
    fs::write(project.path().join("src/good.rs"), "fn good() {}\n").expect("write good file");
    let path = project.path().to_str().expect("fixture path");
    let install = run_assura(&["agent", "integration", "install", "codex", path]);
    assert!(
        install.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&install.stdout),
        String::from_utf8_lossy(&install.stderr)
    );
    git_command(project.path(), &["init"]);
    git_command(project.path(), &["add", "."]);
    git_command(project.path(), &["commit", "-m", "initial"]);
    project
}

fn codex_hook_script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(".codex")
        .join("hooks")
        .join("assura-agent-nudge.py")
}

fn run_codex_hook(project: &Path, input: Value, session_id: &str) -> Output {
    let mut child = Command::new("python3")
        .arg(codex_hook_script())
        .current_dir(project)
        .env("ASSURA_BIN", assura_bin())
        .env("ASSURA_AGENT_SESSION_ID", session_id)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("codex hook starts");
    {
        let stdin = child.stdin.as_mut().expect("hook stdin");
        write!(stdin, "{input}").expect("write hook input");
    }
    child.wait_with_output().expect("hook exits")
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
        serde_json::json!(["cache_policy.cooldown.suppressed"])
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
fn agent_nudge_accepts_beta_agent_labels_and_events_without_private_validation_paths() {
    let project = nudge_fixture();
    let path = project.path().to_str().expect("fixture path");

    for agent in ["codex", "opencode", "claude", "pi"] {
        for event in [
            "session-start",
            "before-tool",
            "after-tool",
            "file-read",
            "idle",
            "recovery",
        ] {
            let mut args = vec!["nudge", path, "--event", event, "--agent", agent];
            if !["session-start", "idle", "recovery"].contains(&event) {
                args.extend_from_slice(&["--changed", "src/BadName.rs"]);
            }
            let nudge = agent_json(&args);

            assert_eq!(nudge["schema"], "assura.agent-nudge.v1");
            assert_eq!(nudge["target_agent"], agent);
            assert_eq!(
                nudge["event"],
                match event {
                    "session-start" => "session_start",
                    "before-tool" => "before_tool",
                    "after-tool" => "after_tool",
                    "file-read" => "file_read",
                    "idle" => "idle",
                    "recovery" => "recovery",
                    _ => unreachable!(),
                }
            );
            assert_eq!(nudge["cache_policy"]["stable_by_default"], true);

            let suggested_command = nudge["summary"]["suggested_command"]
                .as_str()
                .expect("suggested command");
            assert!(suggested_command.contains("assura check --format agent"));
            if agent == "codex" {
                assert!(suggested_command.contains("--agent codex"));
            } else {
                assert!(
                    !suggested_command.contains("--agent "),
                    "{agent} should label the nudge payload without creating a private check adapter"
                );
            }
        }
    }
}

#[test]
fn agent_integration_lifecycle_installs_reviewable_bundles_for_all_hosts() {
    let project = nudge_fixture();
    let path = project.path().to_str().expect("fixture path");

    for agent in ["codex", "opencode", "claude", "pi"] {
        let dry_run = agent_json(&[
            "integration",
            "install",
            agent,
            path,
            "--dry-run",
            "--format",
            "json",
        ]);
        assert_eq!(dry_run["schema"], "assura.agent-integration.lifecycle.v1");
        assert_eq!(dry_run["action"], "install");
        assert_eq!(dry_run["agent"], agent);
        assert_eq!(dry_run["dry_run"], true);
        assert_eq!(dry_run["changed"], true);
        assert_eq!(dry_run["installed"], false);
        assert!(dry_run["files"]
            .as_array()
            .unwrap()
            .iter()
            .all(|file| file["action"] == "would_write"));

        let install = agent_json(&["integration", "install", agent, path]);
        assert_eq!(install["schema"], "assura.agent-integration.lifecycle.v1");
        assert_eq!(
            install["manifest"]["schema"],
            "assura.agent-integration-manifest.v1"
        );
        assert_eq!(install["manifest"]["target_agent"], agent);
        assert_eq!(
            install["host"]["host_config_status"],
            serde_json::json!("explicit-activation")
        );
        let events = install["manifest"]["event_placements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|event| event["event"].as_str().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            events,
            [
                "session-start",
                "before-tool",
                "after-tool",
                "file-read",
                "idle",
                "recovery"
            ]
        );

        let integration_dir = project.path().join(".assura/integrations").join(agent);
        let manifest_text = fs::read_to_string(integration_dir.join("manifest.json")).unwrap();
        let wrapper_text = fs::read_to_string(integration_dir.join("assura-agent.sh")).unwrap();
        let readme_text = fs::read_to_string(integration_dir.join("README.md")).unwrap();
        let manifest_json: Value =
            serde_json::from_str(&manifest_text).expect("manifest is valid JSON");
        assert_eq!(
            manifest_json["schema"],
            "assura.agent-integration-manifest.v1"
        );
        assert_eq!(
            manifest_json["managed_marker"],
            "Generated by Assura agent integration lifecycle"
        );
        for text in [&manifest_text, &wrapper_text, &readme_text] {
            assert!(text.contains("Generated by Assura agent integration lifecycle"));
        }
        assert!(wrapper_text.contains("\"$ASSURA_BIN\" agent nudge"));
        assert!(wrapper_text.contains("\"$ASSURA_BIN\" check"));
        assert!(wrapper_text.contains("\"$ASSURA_BIN\" daemon status"));
        assert!(wrapper_text.contains("\"$ASSURA_BIN\" daemon doctor"));
        assert!(wrapper_text.contains("ASSURA_AGENT_LOG"));
        assert!(wrapper_text.contains("ASSURA_BIN"));
        assert!(wrapper_text.contains(".assura/agent-sessions"));
        assert!(readme_text.contains("nudges.jsonl"));
        if agent == "codex" {
            assert!(readme_text.contains("PostToolUse"));
        }
        assert_eq!(
            manifest_json["logging"]["log_file"],
            ".assura/agent-sessions/nudges.jsonl"
        );
        assert!(!wrapper_text.contains("assura-codex-feedback"));

        let status = agent_json(&["integration", "status", agent, path]);
        assert_eq!(status["installed"], true);
        assert!(status["files"]
            .as_array()
            .unwrap()
            .iter()
            .all(|file| file["managed"] == true));

        let doctor = agent_json(&["integration", "doctor", agent, path]);
        assert_eq!(doctor["installed"], true);
        assert!(doctor["checks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|check| check["status"] == "pass"));

        let remove = agent_json(&["integration", "remove", agent, path]);
        assert_eq!(remove["action"], "remove");
        assert_eq!(remove["changed"], true);
        assert_eq!(remove["installed"], false);
    }
}

#[test]
fn codex_hooks_config_wires_assura_post_tool_use() {
    let config_text = fs::read_to_string(".codex/hooks.json").expect("codex hooks config");
    let config: Value = serde_json::from_str(&config_text).expect("hooks config is JSON");

    let prompt_hooks = config["hooks"]["UserPromptSubmit"]
        .as_array()
        .expect("prompt hooks");
    assert!(prompt_hooks.iter().any(|group| {
        group["hooks"].as_array().unwrap().iter().any(|hook| {
            hook["command"]
                .as_str()
                .unwrap_or_default()
                .contains("assura-agent-nudge.py")
        })
    }));

    let post_tool_hooks = config["hooks"]["PostToolUse"]
        .as_array()
        .expect("post tool hooks");
    assert!(post_tool_hooks.iter().any(|group| {
        group["matcher"] == "*"
            && group["hooks"].as_array().unwrap().iter().any(|hook| {
                hook["command"]
                    .as_str()
                    .unwrap_or_default()
                    .contains("assura-agent-nudge.py")
                    && hook["statusMessage"] == "Reviewing Assura nudges"
            })
    }));
}

#[test]
fn agent_integration_lifecycle_protects_unmanaged_files_and_doctor_fails_incomplete_bundle() {
    let project = nudge_fixture();
    let path = project.path().to_str().expect("fixture path");
    let integration_dir = project.path().join(".assura/integrations/codex");
    fs::create_dir_all(&integration_dir).expect("integration dir");
    fs::write(
        integration_dir.join("assura-agent.sh"),
        "#!/bin/sh\necho custom\n",
    )
    .expect("custom wrapper");

    let dry_run = agent_json(&[
        "integration",
        "install",
        "codex",
        path,
        "--dry-run",
        "--format",
        "json",
    ]);
    assert_eq!(dry_run["dry_run"], true);
    assert_eq!(dry_run["changed"], true);
    assert!(!integration_dir.join("manifest.json").exists());

    let blocked = run_assura(&["agent", "integration", "install", "codex", path]);
    assert!(!blocked.status.success());
    assert!(String::from_utf8_lossy(&blocked.stderr)
        .contains("refusing to overwrite non-Assura-managed file"));
    assert!(!integration_dir.join("manifest.json").exists());

    let forced = agent_json(&["integration", "install", "codex", path, "--force"]);
    assert_eq!(forced["changed"], true);
    assert!(fs::read_to_string(integration_dir.join("assura-agent.sh"))
        .expect("managed wrapper")
        .contains("Generated by Assura agent integration lifecycle"));

    fs::remove_file(integration_dir.join("assura-agent.sh")).expect("remove wrapper");
    let doctor = run_assura(&[
        "agent",
        "integration",
        "doctor",
        "codex",
        path,
        "--format",
        "json",
    ]);
    assert!(!doctor.status.success());
    let json: Value = serde_json::from_slice(&doctor.stdout).expect("doctor JSON");
    assert!(json["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| { check["name"] == "expected_files" && check["status"] == "fail" }));
    assert!(json["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| { check["name"] == "shared_nudge_contract" && check["status"] == "fail" }));
    assert!(json["checks"]
        .as_array()
        .unwrap()
        .iter()
        .any(|check| { check["name"] == "nudge_logging_contract" && check["status"] == "fail" }));
}

#[test]
fn agent_nudge_logs_compact_jsonl_when_enabled() {
    let project = nudge_fixture();
    let path = project.path().to_str().expect("fixture path");
    let log_dir = project.path().join(".assura/agent-sessions");
    let output = Command::new(assura_bin())
        .args([
            "agent",
            "nudge",
            path,
            "--event",
            "after-tool",
            "--changed",
            "src/BadName.rs",
            "--agent",
            "codex",
        ])
        .env("ASSURA_AGENT_LOG", "1")
        .env("ASSURA_AGENT_LOG_DIR", &log_dir)
        .env("ASSURA_AGENT_SESSION_ID", "test-session")
        .output()
        .expect("assura nudge runs");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log_text = fs::read_to_string(log_dir.join("nudges.jsonl")).expect("nudge log is written");
    let lines = log_text.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 1);
    let record: Value = serde_json::from_str(lines[0]).expect("log record is JSON");
    assert_eq!(record["schema"], "assura.agent-nudge-log.v1");
    assert_eq!(record["session_id"], "test-session");
    assert_eq!(record["target_agent"], "codex");
    assert_eq!(record["event"], "after_tool");
    assert_eq!(record["should_inject"], true);
    assert_eq!(record["payload"]["schema"], "assura.agent-nudge.v1");
    assert_eq!(record["payload"]["summary"]["should_inject"], true);
}

#[test]
fn codex_post_tool_hook_injects_changed_path_nudge_and_logs_state() {
    let project = git_nudge_fixture();
    fs::write(project.path().join("src/BadName.rs"), "fn bad() {}\n").expect("write bad file");

    let output = run_codex_hook(
        project.path(),
        serde_json::json!({
            "session_id": "post-tool-test",
            "cwd": project.path().to_str().expect("project path"),
            "hook_event_name": "PostToolUse",
            "turn_id": "turn-1",
            "tool_use_id": "tool-1",
            "tool_name": "apply_patch",
            "tool_input": {"command": "*** Begin Patch\n*** Add File: src/BadName.rs\n+fn bad() {}\n*** End Patch"},
            "tool_response": {"status": "success"}
        }),
        "post-tool-test",
    );
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let hook_output: Value = serde_json::from_slice(&output.stdout).expect("hook emits JSON");
    assert_eq!(
        hook_output["hookSpecificOutput"]["hookEventName"],
        "PostToolUse"
    );
    let context = hook_output["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .expect("additional context");
    assert!(context.contains("<assura-nudge>"));
    assert!(context.contains("Intent: edit"));
    assert!(context.contains("Git delta: 1 changed since previous hook/message"));
    assert!(context.contains("src/BadName.rs"));
    assert!(context.contains("file_naming"));

    let log_text = fs::read_to_string(project.path().join(".assura/agent-sessions/nudges.jsonl"))
        .expect("nudge log");
    let log_record: Value = serde_json::from_str(log_text.lines().last().unwrap()).unwrap();
    assert_eq!(log_record["event"], "after_tool");
    assert_eq!(log_record["session_id"], "post-tool-test");
    assert_eq!(log_record["changed_path_count"], 1);
    assert_eq!(log_record["should_inject"], true);

    let state_text = fs::read_to_string(
        project
            .path()
            .join(".assura/agent-sessions/codex-hook-state.jsonl"),
    )
    .expect("hook state log");
    let state_record: Value = serde_json::from_str(state_text.lines().last().unwrap()).unwrap();
    assert_eq!(state_record["schema"], "assura.codex-hook-state.v1");
    assert_eq!(state_record["hook_event_name"], "PostToolUse");
    assert_eq!(state_record["tool_intent"], "edit");
    assert_eq!(state_record["changed_since_previous_count"], 1);
}

#[test]
fn codex_post_tool_hook_injects_git_commit_intent_without_new_delta() {
    let project = git_nudge_fixture();
    fs::write(
        project.path().join("src/commit-ready.rs"),
        "fn commit_ready() {}\n",
    )
    .expect("write valid dirty file");

    let prompt = run_codex_hook(
        project.path(),
        serde_json::json!({
            "session_id": "commit-intent-test",
            "cwd": project.path().to_str().expect("project path"),
            "hook_event_name": "UserPromptSubmit",
            "turn_id": "turn-1",
            "prompt": "commit the current work"
        }),
        "commit-intent-test",
    );
    assert!(
        prompt.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&prompt.stdout),
        String::from_utf8_lossy(&prompt.stderr)
    );

    let output = run_codex_hook(
        project.path(),
        serde_json::json!({
            "session_id": "commit-intent-test",
            "cwd": project.path().to_str().expect("project path"),
            "hook_event_name": "PostToolUse",
            "turn_id": "turn-1",
            "tool_use_id": "tool-2",
            "tool_name": "Bash",
            "tool_input": {"command": "git commit -m test"},
            "tool_response": {"exit_code": 1}
        }),
        "commit-intent-test",
    );
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let hook_output: Value = serde_json::from_slice(&output.stdout).expect("hook emits JSON");
    let context = hook_output["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .expect("additional context");
    assert!(context.contains("Intent: git_commit"));
    assert!(context.contains("Git delta: 0 changed since previous hook/message"));
    assert!(context.contains("Git intent: detected git_commit"));
    assert!(context.contains("Summary: 0 nudge(s)"));
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
fn agent_nudge_suppresses_identical_messages_during_the_cooldown() {
    let project = nudge_fixture();
    let path = project.path().to_str().expect("fixture path");
    let args = [
        "nudge",
        path,
        "--event",
        "after-tool",
        "--changed",
        "src/BadName.rs",
        "--cooldown-seconds",
        "600",
    ];

    let first = agent_json(&args);
    let second = agent_json(&args);
    assert_eq!(first["summary"]["nudge_count"], 1);
    assert_eq!(first["cache_policy"]["cooldown"]["suppressed"], 0);
    assert_eq!(second["summary"]["nudge_count"], 0);
    assert_eq!(second["cache_policy"]["cooldown"]["suppressed"], 1);
    assert_eq!(second["summary"]["omitted_count"], 1);
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
