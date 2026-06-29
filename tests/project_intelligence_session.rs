use serde_json::Value;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::{Duration, Instant};

const CONTENT_FIXTURE: &str = "tests/fixtures/content_runtime/valid";
const BEACON_INVALID: &str = "tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid";

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

struct SessionProcess {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<std::process::ChildStdout>,
}

impl SessionProcess {
    fn start(path: &Path) -> Self {
        Self::start_with_ready_file(path, None)
    }

    fn start_with_ready_file(path: &Path, ready_file: Option<&Path>) -> Self {
        let mut command = Command::new(assura_bin());
        command.args(["content", "session"]).arg(path);
        if let Some(ready_file) = ready_file {
            command.env("ASSURA_CONTENT_SESSION_READY_FILE", ready_file);
        }
        let mut child = command
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
fn content_session_reuses_context_for_repeated_requests() {
    let mut session = SessionProcess::start(Path::new(BEACON_INVALID));

    let first = session.request(serde_json::json!({
        "request_id": "diagnostics-1",
        "type": "diagnostics"
    }));
    assert_eq!(
        first["schema"],
        "assura.project-intelligence.session.response.v1"
    );
    assert_eq!(first["request_id"], "diagnostics-1");
    assert_eq!(first["reload"]["state"], "initial_load");
    assert!(first["ok"].as_bool().expect("ok"));
    assert!(first["response"]["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .iter()
        .any(|item| item["rule"] == "content_runtime:missing_reference"));

    let second = session.request(serde_json::json!({
        "request_id": "pack-1",
        "type": "context-pack",
        "collection": "epics",
        "id": "epic-checkout",
        "text": "checkout",
        "limit": 3
    }));
    assert_eq!(second["reload"]["state"], "reused");
    assert_eq!(
        second["response"]["schema"],
        "assura.project-intelligence.context-pack.v1"
    );
    assert_eq!(second["response"]["request"]["mode"], "object");
    assert_eq!(second["response"]["bounds"]["limit"], 3);

    session.finish();
}

#[test]
fn content_session_reports_invalid_request_without_exiting() {
    let mut session = SessionProcess::start(Path::new(CONTENT_FIXTURE));

    let error = session.request(serde_json::json!({
        "request_id": "bad",
        "type": "expand",
        "collection": "goals"
    }));
    assert_eq!(error["ok"], false);
    assert_eq!(error["request_id"], "bad");
    assert_eq!(error["error"]["code"], "request_failed");
    assert!(error["error"]["message"]
        .as_str()
        .expect("message")
        .contains("requires `id`"));

    let follow_up = session.request(serde_json::json!({
        "request_id": "search",
        "type": "search",
        "text": "Portable"
    }));
    assert_eq!(follow_up["ok"], true);
    assert_eq!(follow_up["reload"]["state"], "reused");
    assert!(!follow_up["response"]["matches"]
        .as_array()
        .expect("matches")
        .is_empty());

    session.finish();
}

#[test]
fn content_session_reloads_after_modeled_content_changes() {
    let temp = tempfile::tempdir().expect("tempdir");
    copy_dir(Path::new(CONTENT_FIXTURE), temp.path());
    let goal_path = temp.path().join("docs/goals/goal_portable_structure.md");

    let mut session = SessionProcess::start(temp.path());
    let first = session.request(serde_json::json!({
        "request_id": "before",
        "type": "search",
        "text": "session-reload-token"
    }));
    assert_eq!(first["reload"]["state"], "initial_load");
    assert!(first["response"]["matches"]
        .as_array()
        .expect("matches")
        .is_empty());

    let content = fs::read_to_string(&goal_path).expect("read goal").replace(
        "title: Portable Structure Policy",
        "title: Portable Structure Policy session-reload-token",
    );
    fs::write(&goal_path, content).expect("write changed goal");

    let second = session.request(serde_json::json!({
        "request_id": "after",
        "type": "search",
        "text": "session-reload-token"
    }));
    assert_eq!(second["reload"]["state"], "reloaded");
    assert!(second["response"]["matches"]
        .as_array()
        .expect("matches")
        .iter()
        .any(|item| item["text"]
            .as_str()
            .expect("match text")
            .contains("session-reload-token")));

    session.finish();
}

#[test]
fn content_session_reloads_before_first_response_if_files_changed_after_startup() {
    let temp = tempfile::tempdir().expect("tempdir");
    copy_dir(Path::new(CONTENT_FIXTURE), temp.path());
    let goal_path = temp.path().join("docs/goals/goal_portable_structure.md");
    let ready_file = temp.path().join("session-ready.txt");

    let mut session = SessionProcess::start_with_ready_file(temp.path(), Some(&ready_file));
    wait_for_ready_file(&ready_file);

    let content = fs::read_to_string(&goal_path).expect("read goal").replace(
        "title: Portable Structure Policy",
        "title: Portable Structure Policy first-request-token",
    );
    fs::write(&goal_path, content).expect("write changed goal");

    let first = session.request(serde_json::json!({
        "request_id": "first",
        "type": "search",
        "text": "first-request-token"
    }));
    assert_eq!(first["reload"]["state"], "reloaded");
    assert!(first["response"]["matches"]
        .as_array()
        .expect("matches")
        .iter()
        .any(|item| item["text"]
            .as_str()
            .expect("match text")
            .contains("first-request-token")));

    session.finish();
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

fn wait_for_ready_file(path: &Path) {
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(5) {
        if path.exists() {
            return;
        }
        std::thread::sleep(Duration::from_millis(25));
    }
    panic!("session did not write ready file: {}", path.display());
}
