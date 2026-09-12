use serde_json::Value;
use std::fs;
use std::process::{Command, Output};
use std::thread;
use std::time::Duration;
use tempfile::TempDir;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}
fn git(root: &TempDir, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root.path())
        .env("GIT_AUTHOR_NAME", "Assura Test")
        .env("GIT_AUTHOR_EMAIL", "assura@example.com")
        .env("GIT_COMMITTER_NAME", "Assura Test")
        .env("GIT_COMMITTER_EMAIL", "assura@example.com")
        .output()
        .expect("git runs");
    assert!(
        output.status.success(),
        "git {:?}: {}",
        args,
        String::from_utf8_lossy(&output.stderr)
    );
}
fn run(root: &TempDir, args: &[&str]) -> Output {
    Command::new(bin())
        .args(args)
        .current_dir(root.path())
        .output()
        .expect("assura runs")
}
fn json(output: Output) -> Value {
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("JSON output")
}
fn fixture(extra: &str) -> TempDir {
    let root = tempfile::tempdir().expect("fixture");
    fs::create_dir_all(root.path().join(".assura")).expect("assura dir");
    fs::write(root.path().join(".assura/config.yml"), format!("structure: {{}}\nagent_feedback:\n  mode: periodic\n  periodic_seconds: 1\n  min_interval_seconds: 1\n  max_messages_per_hour: 4\n  max_bytes_per_hour: 1024\n{extra}")).expect("config");
    fs::write(root.path().join("README.md"), "# fixture\n").expect("file");
    git(&root, &["init"]);
    git(&root, &["add", "."]);
    git(&root, &["commit", "-m", "initial"]);
    root
}

#[test]
fn automatic_delivery_reads_warm_snapshot_and_respects_periodic_line_cap() {
    let root = fixture("");
    let path = root.path().to_str().unwrap();
    let _ = json(run(
        &root,
        &[
            "agent",
            "nudge",
            path,
            "--delivery",
            "inspect",
            "--format",
            "json",
        ],
    ));
    let output = json(run(
        &root,
        &[
            "agent",
            "nudge",
            path,
            "--delivery",
            "automatic",
            "--format",
            "json",
        ],
    ));
    assert_eq!(output["feedback"]["mode"], "periodic");
    assert_eq!(output["feedback"]["snapshot"], "fresh");
    let messages = output["nudges"].as_array().unwrap();
    let line = messages
        .iter()
        .find(|item| item["category"] == "trajectory")
        .expect("trajectory line");
    assert!(line["message"].as_str().unwrap().len() <= 256);
}

#[test]
fn automatic_delivery_is_silent_when_feedback_is_not_configured() {
    let root = fixture("");
    let config = root.path().join(".assura/config.yml");
    fs::write(&config, "structure: {}\n").expect("disable config");
    let path = root.path().to_str().unwrap();
    let output = json(run(
        &root,
        &[
            "agent",
            "nudge",
            path,
            "--delivery",
            "automatic",
            "--format",
            "json",
        ],
    ));
    assert!(output["feedback"].is_null());
    assert!(!output["nudges"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["category"] == "trajectory"));
}

#[test]
fn feedback_config_rejects_unknown_nested_fields() {
    let root = fixture("  unexpected: true\n");
    let path = root.path().to_str().unwrap();
    let output = run(
        &root,
        &[
            "agent",
            "nudge",
            path,
            "--delivery",
            "automatic",
            "--format",
            "json",
        ],
    );
    let value = json(output);
    assert_eq!(value["daemon"]["state"], "unavailable");
    assert!(value["daemon"]["reason"]
        .as_str()
        .unwrap()
        .contains("unknown field"));
}

#[test]
fn generated_codex_wrapper_returns_before_refresh_and_reuses_cache() {
    let root = fixture("");
    let path = root.path().to_str().unwrap();
    let install = run(&root, &["agent", "integration", "install", "codex", path]);
    assert!(
        install.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&install.stdout),
        String::from_utf8_lossy(&install.stderr)
    );
    let wrapper = root
        .path()
        .join(".assura/integrations/codex/assura-agent.sh");
    let wrapper_text = fs::read_to_string(&wrapper).expect("generated wrapper");
    assert!(wrapper_text.contains("--delivery automatic"));

    let first = Command::new("sh")
        .arg(&wrapper)
        .current_dir(root.path())
        .env("ASSURA_BIN", bin())
        .env("ASSURA_AGENT_MODE", "nudge")
        .env("ASSURA_AGENT_EVENT", "session-start")
        .env("ASSURA_AGENT_LOG", "0")
        .output()
        .expect("wrapper runs");
    assert!(first.status.success());
    let first_json: Value = serde_json::from_slice(&first.stdout).expect("cold wrapper JSON");
    assert_eq!(first_json["feedback"]["refresh"], "scheduled");

    let cache_dir = root.path().join(".git/assura/trajectory");
    for _ in 0..100 {
        if cache_dir.is_dir()
            && fs::read_dir(&cache_dir)
                .map(|entries| {
                    entries.flatten().any(|entry| {
                        entry
                            .path()
                            .extension()
                            .is_some_and(|extension| extension == "json")
                    })
                })
                .unwrap_or(false)
        {
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    assert!(cache_dir.is_dir(), "refresh child did not create cache");

    let second = Command::new("sh")
        .arg(&wrapper)
        .current_dir(root.path())
        .env("ASSURA_BIN", bin())
        .env("ASSURA_AGENT_MODE", "nudge")
        .env("ASSURA_AGENT_EVENT", "session-start")
        .env("ASSURA_AGENT_LOG", "0")
        .output()
        .expect("wrapper runs with warm cache");
    assert!(second.status.success());
    let second_json: Value = serde_json::from_slice(&second.stdout).expect("warm wrapper JSON");
    assert_eq!(second_json["feedback"]["snapshot"], "fresh");
    assert!(second_json["nudges"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["category"] == "trajectory"));
}
