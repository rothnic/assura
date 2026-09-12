use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::process::{Command, Output};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};
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

    let inspected = json(run(
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
    assert_eq!(inspected["feedback"]["messages_last_hour"], 1);
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
fn automatic_delivery_keeps_hourly_message_budget_after_process_restart() {
    let root = fixture("");
    fs::write(
        root.path().join(".assura/config.yml"),
        "structure: {}\nagent_feedback:\n  mode: periodic\n  periodic_seconds: 1\n  min_interval_seconds: 1\n  max_messages_per_hour: 1\n  max_bytes_per_hour: 1024\n",
    )
    .expect("budget config");
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
    let first = json(run(
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
    assert!(first["nudges"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["category"] == "trajectory"));
    thread::sleep(Duration::from_millis(1_100));
    let second = json(run(
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
    assert_eq!(second["feedback"]["reason"], "hourly_message_budget");
    assert!(!second["nudges"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["category"] == "trajectory"));
}

#[test]
fn concurrent_automatic_delivery_shares_one_message_budget() {
    let root = fixture("");
    fs::write(
        root.path().join(".assura/config.yml"),
        "structure: {}\nagent_feedback:\n  mode: periodic\n  periodic_seconds: 1\n  min_interval_seconds: 1\n  max_messages_per_hour: 1\n  max_bytes_per_hour: 1024\n",
    )
    .expect("budget config");
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
    let barrier = Arc::new(Barrier::new(3));
    let outputs = (0..2)
        .map(|_| {
            let barrier = Arc::clone(&barrier);
            let project = root.path().to_path_buf();
            thread::spawn(move || {
                barrier.wait();
                let output = Command::new(bin())
                    .args([
                        "agent",
                        "nudge",
                        project.to_str().unwrap(),
                        "--delivery",
                        "automatic",
                        "--format",
                        "json",
                    ])
                    .current_dir(&project)
                    .output()
                    .expect("concurrent automatic delivery runs");
                json(output)
            })
        })
        .collect::<Vec<_>>();
    barrier.wait();
    let outputs = outputs
        .into_iter()
        .map(|handle| handle.join().expect("delivery thread joins"))
        .collect::<Vec<_>>();
    let delivered = outputs
        .iter()
        .flat_map(|output| output["nudges"].as_array().into_iter().flatten())
        .filter(|item| item["category"] == "trajectory")
        .count();
    assert_eq!(delivered, 1);
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

    let queued = Command::new("sh")
        .arg(&wrapper)
        .current_dir(root.path())
        .env("ASSURA_BIN", bin())
        .env("ASSURA_AGENT_MODE", "nudge")
        .env("ASSURA_AGENT_EVENT", "after-tool")
        .env("ASSURA_AGENT_LOG", "0")
        .output()
        .expect("queued wrapper runs");
    assert!(queued.status.success());
    let queued_json: Value = serde_json::from_slice(&queued.stdout).expect("queued wrapper JSON");
    assert!(matches!(
        queued_json["feedback"]["refresh"].as_str(),
        Some("in_flight" | "queued" | "cooldown" | "debounced" | "scheduled" | "not_requested")
    ));

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

    let mut second_json = None;
    for _ in 0..100 {
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
        let parsed: Value = serde_json::from_slice(&second.stdout).expect("warm wrapper JSON");
        if parsed["feedback"]["snapshot"] == "fresh"
            && parsed["nudges"]
                .as_array()
                .unwrap()
                .iter()
                .any(|item| item["category"] == "trajectory")
        {
            second_json = Some(parsed);
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }
    let second_json = second_json.expect("warm wrapper did not reuse trajectory cache");
    assert_eq!(second_json["feedback"]["snapshot"], "fresh");
}

#[test]
fn queued_refresh_runs_after_the_active_refresh_releases_its_lease() {
    let root = fixture("");
    let path = root.path().to_str().unwrap();
    let identity = root.path().join(".git").canonicalize().unwrap();
    let digest = format!(
        "{:x}",
        Sha256::digest(identity.to_string_lossy().as_bytes())
    );
    let feedback_dir = root.path().join(".git/assura/feedback");
    fs::create_dir_all(&feedback_dir).expect("feedback directory");
    let lock = feedback_dir.join(format!("{digest}.refresh"));
    let queued = lock.with_extension("queued");
    fs::write(&lock, "active").expect("active refresh lease");

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
    assert_eq!(output["feedback"]["refresh"], "in_flight");
    assert!(
        queued.is_file(),
        "active refresh did not queue a generation"
    );
    let inspect = Command::new(bin())
        .args([
            "agent",
            "nudge",
            path,
            "--delivery",
            "inspect",
            "--format",
            "json",
        ])
        .current_dir(root.path())
        .env("ASSURA_FEEDBACK_REFRESH_LOCK", &lock)
        .env("ASSURA_FEEDBACK_REFRESH_TOKEN", "active")
        .env("ASSURA_FEEDBACK_REFRESH_TIMEOUT_MS", "2000")
        .output()
        .expect("release inspect runs");
    assert!(inspect.status.success());

    for _ in 0..100 {
        if !queued.exists() {
            break;
        }
        thread::sleep(Duration::from_millis(25));
    }
    assert!(!queued.exists(), "queued refresh did not run after release");
    assert!(root.path().join(".git/assura/trajectory").is_dir());
}

#[test]
fn expired_refresh_does_not_publish_a_snapshot() {
    let root = fixture("");
    let path = root.path().to_str().unwrap();
    let lock = root.path().join(".git/assura/feedback/expired.refresh");
    fs::create_dir_all(lock.parent().unwrap()).expect("feedback directory");
    fs::write(&lock, "expired").expect("expired refresh lease");
    let deadline = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_millis()
        .saturating_sub(1)
        .to_string();
    let output = Command::new(bin())
        .args([
            "agent",
            "nudge",
            path,
            "--delivery",
            "inspect",
            "--format",
            "json",
        ])
        .current_dir(root.path())
        .env("ASSURA_FEEDBACK_REFRESH_LOCK", &lock)
        .env("ASSURA_FEEDBACK_REFRESH_TOKEN", "expired")
        .env("ASSURA_FEEDBACK_REFRESH_DEADLINE_MS", deadline)
        .output()
        .expect("expired inspect runs");
    assert!(!output.status.success());
    let cache_dir = root.path().join(".git/assura/trajectory");
    assert!(!cache_dir.exists() || fs::read_dir(cache_dir).unwrap().next().is_none());

    let recovery = json(run(
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
    assert_eq!(recovery["feedback"]["refresh"], "scheduled");
}

#[test]
fn collector_configuration_change_does_not_reuse_an_old_snapshot() {
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
    fs::write(
        root.path().join(".assura/config.yml"),
        "structure: {}\nagent_feedback:\n  mode: periodic\n  periodic_seconds: 1\n  min_interval_seconds: 1\n  max_bytes_per_hour: 1024\n  trajectory:\n    source_paths: [different/**]\n",
    )
    .expect("changed config");
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
    assert_eq!(output["feedback"]["reason"], "cold_snapshot");
    assert_eq!(output["feedback"]["refresh"], "scheduled");
    assert!(!output["nudges"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["category"] == "trajectory"));
}
