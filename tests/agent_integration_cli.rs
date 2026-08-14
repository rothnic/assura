use serde_json::Value;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

fn assura_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_assura"))
}

fn project_fixture() -> TempDir {
    let project = TempDir::new().expect("temp project");
    fs::create_dir_all(project.path().join(".assura")).expect("assura dir");
    fs::write(
        project.path().join(".assura/config.yml"),
        "structure:\n  ./:\n    extra: true\n",
    )
    .expect("config");
    project
}

fn adapter_fixture() -> TempDir {
    let project = project_fixture();
    fs::create_dir_all(project.path().join("src")).expect("source dir");
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"structure:
  ./:
    extra: true
    children:
      src/:
        files:
          naming: kebab-case
          extensions: ["rs"]
"#,
    )
    .expect("adapter config");
    fs::write(project.path().join("src/BadName.rs"), "fn main() {}\n").expect("violating source");
    project
}

fn run_agent(project: &Path, args: &[&str]) -> Output {
    Command::new(assura_bin())
        .args(["agent"])
        .args(args)
        .arg(project)
        .arg("--format")
        .arg("json")
        .output()
        .expect("run assura agent command")
}

fn agent_json(project: &Path, args: &[&str]) -> Value {
    let output = run_agent(project, args);
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("agent JSON")
}

fn activation_path(project: &Path, agent: &str) -> PathBuf {
    match agent {
        "codex" => project.join(".codex/hooks.json"),
        "claude" => project.join(".claude/settings.json"),
        "opencode" => project.join(".opencode/plugins/assura.js"),
        "pi" => project.join(".pi/extensions/assura.ts"),
        _ => panic!("unknown agent"),
    }
}

fn activate(project: &Path, agent: &str) {
    agent_json(project, &["integration", "activate", agent]);
}

fn run_python_adapter(project: &Path, agent: &str, event: &str) -> Value {
    activate(project, agent);
    let script = project
        .join(".assura/integrations")
        .join(agent)
        .join("assura-hook.py");
    let mut child = Command::new("python3")
        .arg(script)
        .current_dir(project)
        .env("ASSURA_BIN", assura_bin())
        .env("ASSURA_AGENT_LOG", "0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("python host adapter starts");
    write!(
        child.stdin.as_mut().expect("adapter stdin"),
        "{}",
        serde_json::json!({
            "session_id": format!("{agent}-fixture"),
            "cwd": project,
            "hook_event_name": event,
            "tool_name": "Write",
            "tool_input": {"file_path": "src/BadName.rs"}
        })
    )
    .expect("write event payload");
    let output = child.wait_with_output().expect("python adapter exits");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("adapter context JSON")
}

fn run_node_module(project: &Path, module: &Path, source: &str) -> Value {
    let output = Command::new("node")
        .args(["--input-type=module", "--eval", source])
        .current_dir(project)
        .env("ASSURA_BIN", assura_bin())
        .env("ASSURA_AGENT_LOG", "0")
        .env("ASSURA_TEST_MODULE", module)
        .env("ASSURA_TEST_PROJECT", project)
        .output()
        .expect("node host adapter starts");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("node adapter context JSON")
}

#[test]
fn managed_activation_lifecycle_is_explicit_and_idempotent_for_all_hosts() {
    for agent in ["codex", "claude", "opencode", "pi"] {
        let project = project_fixture();

        let install = agent_json(project.path(), &["integration", "install", agent]);
        assert_eq!(install["installed"], true);
        assert_eq!(install["activation"]["generated"], true);
        assert_eq!(install["activation"]["activated"], false);
        assert_eq!(install["activation"]["verified"], false);
        assert_eq!(install["activation"]["conflicted"], false);
        assert_eq!(
            install["activation"]["verification_scope"],
            "managed files and project host configuration"
        );
        assert_eq!(install["manifest"]["adapter_contract_version"], 1);
        assert_eq!(
            install["manifest"]["adapter_version"],
            env!("CARGO_PKG_VERSION")
        );
        assert!(install["manifest"]["managed_files"]
            .as_array()
            .unwrap()
            .iter()
            .all(|file| file["sha256"].as_str().unwrap().len() == 64));
        assert!(!activation_path(project.path(), agent).exists());

        let activate = agent_json(project.path(), &["integration", "activate", agent]);
        assert_eq!(activate["action"], "activate");
        assert_eq!(activate["changed"], true);
        assert_eq!(activate["activation"]["activated"], true);
        assert_eq!(activate["activation"]["verified"], true);
        assert!(activation_path(project.path(), agent).is_file());

        let repeated = agent_json(project.path(), &["integration", "activate", agent]);
        assert_eq!(repeated["changed"], false);
        assert_eq!(repeated["activation"]["verified"], true);

        let status = agent_json(project.path(), &["integration", "status", agent]);
        assert_eq!(status["installed"], true);
        assert_eq!(status["activation"]["activated"], true);
        assert_eq!(status["activation"]["verified"], true);

        let deactivate = agent_json(project.path(), &["integration", "deactivate", agent]);
        assert_eq!(deactivate["action"], "deactivate");
        assert_eq!(deactivate["changed"], true);
        assert_eq!(deactivate["installed"], true);
        assert_eq!(deactivate["activation"]["activated"], false);
        assert!(!activation_path(project.path(), agent).exists());

        let remove = agent_json(project.path(), &["integration", "remove", agent]);
        assert_eq!(remove["installed"], false);
        assert_eq!(remove["activation"]["generated"], false);
        assert_eq!(remove["activation"]["activated"], false);
    }
}

#[test]
fn codex_post_tool_event_injects_bounded_assura_context() {
    let project = adapter_fixture();
    let output = run_python_adapter(project.path(), "codex", "PostToolUse");
    let context = output["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .expect("Codex additional context");
    assert_eq!(output["hookSpecificOutput"]["hookEventName"], "PostToolUse");
    assert!(context.contains("<assura-feedback>"));
    assert!(context.contains("src/BadName.rs"));
}

#[test]
fn claude_pre_tool_event_injects_bounded_assura_context() {
    let project = adapter_fixture();
    let output = run_python_adapter(project.path(), "claude", "PreToolUse");
    let context = output["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .expect("Claude additional context");
    assert_eq!(output["hookSpecificOutput"]["hookEventName"], "PreToolUse");
    assert!(context.contains("<assura-feedback>"));
    assert!(context.contains("src/BadName.rs"));
}

#[test]
fn opencode_after_tool_event_appends_bounded_assura_context() {
    let project = adapter_fixture();
    activate(project.path(), "opencode");
    let plugin = activation_path(project.path(), "opencode");
    let module = project.path().join("assura-opencode-fixture.mjs");
    fs::copy(plugin, &module).expect("copy JavaScript plugin for Node fixture");
    let output = run_node_module(
        project.path(),
        &module,
        r#"
import { pathToFileURL } from "node:url";
const plugin = await import(pathToFileURL(process.env.ASSURA_TEST_MODULE).href);
const hooks = await plugin.AssuraPlugin({ directory: process.env.ASSURA_TEST_PROJECT });
const output = { output: "tool output" };
await hooks["tool.execute.after"](
  { sessionID: "opencode-fixture", args: { file_path: "src/BadName.rs" } },
  output,
);
console.log(JSON.stringify(output));
"#,
    );
    let context = output["output"].as_str().expect("OpenCode tool output");
    assert!(context.contains("<assura-feedback>"));
    assert!(context.contains("src/BadName.rs"));
}

#[test]
fn pi_tool_result_event_appends_bounded_assura_context() {
    let project = adapter_fixture();
    activate(project.path(), "pi");
    let extension = activation_path(project.path(), "pi");
    let module = project.path().join("assura-pi-fixture.mjs");
    fs::copy(extension, &module).expect("copy TypeScript-compatible extension for Node fixture");
    let output = run_node_module(
        project.path(),
        &module,
        r#"
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { pathToFileURL } from "node:url";
const runFile = promisify(execFile);
const extension = await import(pathToFileURL(process.env.ASSURA_TEST_MODULE).href);
const handlers = {};
const pi = {
  on: (event, handler) => { handlers[event] = handler; },
  exec: async (command, args) => {
    const result = await runFile(command, args, {
      cwd: process.env.ASSURA_TEST_PROJECT,
      env: process.env,
      timeout: 8000,
    });
    return { code: 0, stdout: result.stdout, stderr: result.stderr };
  },
};
extension.default(pi);
const output = await handlers.tool_result(
  { input: { path: "src/BadName.rs" }, content: [{ type: "text", text: "tool output" }] },
  { cwd: process.env.ASSURA_TEST_PROJECT },
);
console.log(JSON.stringify(output));
"#,
    );
    let content = output["content"].as_array().expect("Pi tool content");
    let context = content.last().unwrap()["text"]
        .as_str()
        .expect("Pi context");
    assert!(context.contains("<assura-feedback>"));
    assert!(context.contains("src/BadName.rs"));
}

#[test]
fn json_host_activation_preserves_unrelated_hooks_and_removes_only_assura_entries() {
    for (agent, relative_path) in [
        ("codex", ".codex/hooks.json"),
        ("claude", ".claude/settings.json"),
    ] {
        let project = project_fixture();
        let config_path = project.path().join(relative_path);
        fs::create_dir_all(config_path.parent().unwrap()).expect("host config dir");
        fs::write(
            &config_path,
            r#"{
  "theme": "dark",
  "hooks": {
    "UserPromptSubmit": [
      {"hooks": [{"type": "command", "command": "python3 custom.py"}]}
    ]
  }
}
"#,
        )
        .expect("existing host config");

        agent_json(project.path(), &["integration", "activate", agent]);
        let active: Value =
            serde_json::from_slice(&fs::read(&config_path).unwrap()).expect("active host config");
        assert_eq!(active["theme"], "dark");
        assert!(active["hooks"]["UserPromptSubmit"]
            .as_array()
            .unwrap()
            .iter()
            .any(|group| group["hooks"][0]["command"] == "python3 custom.py"));

        agent_json(project.path(), &["integration", "deactivate", agent]);
        let inactive: Value =
            serde_json::from_slice(&fs::read(&config_path).unwrap()).expect("inactive host config");
        assert_eq!(inactive["theme"], "dark");
        assert_eq!(
            inactive["hooks"]["UserPromptSubmit"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            inactive["hooks"]["UserPromptSubmit"][0]["hooks"][0]["command"],
            "python3 custom.py"
        );
    }
}

#[test]
fn update_repairs_stale_managed_files_but_rejects_unmanaged_activation_drift() {
    let project = project_fixture();
    agent_json(project.path(), &["integration", "activate", "opencode"]);
    let plugin = activation_path(project.path(), "opencode");
    let managed = fs::read_to_string(&plugin).expect("managed plugin");
    assert!(managed.contains("Generated by Assura agent integration lifecycle"));
    fs::write(&plugin, format!("{managed}\n// stale\n")).expect("stale managed plugin");

    let update = agent_json(project.path(), &["integration", "update", "opencode"]);
    assert_eq!(update["changed"], true);
    assert!(!fs::read_to_string(&plugin).unwrap().contains("// stale"));

    fs::write(&plugin, "export const custom = true;\n").expect("unmanaged drift");
    let blocked = run_agent(project.path(), &["integration", "update", "opencode"]);
    assert!(!blocked.status.success());
    assert!(String::from_utf8_lossy(&blocked.stderr)
        .contains("refusing to overwrite non-Assura-managed file"));
    assert_eq!(
        fs::read_to_string(&plugin).unwrap(),
        "export const custom = true;\n"
    );
}

#[test]
fn onboarding_activation_reports_generated_activated_verified_and_conflicted_states() {
    let project = project_fixture();
    fs::create_dir_all(project.path().join(".codex")).expect("codex marker");
    let output = Command::new(assura_bin())
        .args([
            "agent",
            "onboard",
            project.path().to_str().unwrap(),
            "--agent",
            "auto",
            "--activate",
            "--format",
            "json",
        ])
        .output()
        .expect("agent onboard");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).expect("onboarding JSON");
    assert_eq!(report["integration"]["generated"], true);
    assert_eq!(report["integration"]["activated"], true);
    assert_eq!(report["integration"]["verified"], true);
    assert_eq!(report["integration"]["conflicted"], false);
}

#[test]
fn onboarding_auto_activation_refuses_ambiguous_or_missing_host_evidence() {
    let ambiguous = project_fixture();
    fs::create_dir_all(ambiguous.path().join(".codex")).expect("codex marker");
    fs::create_dir_all(ambiguous.path().join(".claude")).expect("claude marker");
    let output = Command::new(assura_bin())
        .args([
            "agent",
            "onboard",
            ambiguous.path().to_str().unwrap(),
            "--agent",
            "auto",
            "--activate",
            "--format",
            "json",
        ])
        .output()
        .expect("ambiguous onboard");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("multiple agent hosts detected; choose --agent codex, claude, opencode, or pi"));

    let missing = project_fixture();
    let output = Command::new(assura_bin())
        .args([
            "agent",
            "onboard",
            missing.path().to_str().unwrap(),
            "--agent",
            "auto",
            "--activate",
            "--format",
            "json",
        ])
        .output()
        .expect("missing-host onboard");
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("no supported agent host detected; choose --agent"));
}
