use assura::config::loader::ConfigLoader;
use assura::content_repository::{
    ContentFinding, ContentRepository, CreateRecordRequest, UpdateRecordRequest,
};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

const ADAPTER_FIXTURE_ROOT: &str = "tests/fixtures/content_runtime/adapters";

#[test]
fn validates_yaml_and_jsonl_adapter_fixtures() {
    let yaml = validate_fixture("yaml/valid");
    let jsonl = validate_fixture("jsonl/valid");

    assert_eq!(yaml.findings, Vec::new());
    assert!(yaml
        .snapshot
        .objects
        .contains_key(&("goals".to_string(), "goal-portable-structure".to_string())));
    assert_eq!(
        yaml.snapshot
            .objects
            .get(&("specs".to_string(), "spec-portable-structure".to_string()))
            .expect("YAML spec loaded")
            .rel_path,
        PathBuf::from("specs/spec_portable_structure.yml")
    );

    assert_eq!(jsonl.findings, Vec::new());
    assert_eq!(jsonl.snapshot.objects.len(), 3);
    assert_eq!(
        jsonl
            .snapshot
            .objects
            .get(&("specs".to_string(), "spec-portable-structure".to_string()))
            .expect("JSONL spec loaded")
            .rel_path,
        PathBuf::from("specs/specs.jsonl")
    );
    assert!(jsonl
        .snapshot
        .objects
        .contains_key(&("specs".to_string(), "spec-zeta".to_string())));
}

#[test]
fn reports_adapter_shape_and_malformed_input() {
    for fixture in ["yaml/invalid_shape", "jsonl/invalid_shape"] {
        let validation = validate_fixture(fixture);
        let finding = validation
            .findings
            .iter()
            .find(|finding| finding.code == "invalid_object_shape")
            .unwrap_or_else(|| panic!("{fixture} should report shape errors"));

        assert_eq!(finding.object_type.as_deref(), Some("Goal"));
        assert_eq!(finding.field.as_deref(), Some("status"));
    }

    for fixture in ["yaml/malformed", "jsonl/malformed"] {
        let root = fixture_root(fixture);
        let before = tree_snapshot(&root);
        let validation = validate_fixture(fixture);

        assert!(
            validation
                .findings
                .iter()
                .any(|finding| finding.code == "parse_error"),
            "{fixture} should report parse_error"
        );
        assert_eq!(tree_snapshot(&root), before);
    }
}

#[test]
fn assura_check_validates_adapter_fixtures_and_diagnostics() {
    for fixture in ["yaml/valid", "jsonl/valid"] {
        let output = check_fixture(fixture, &["--format", "json"]);
        assert!(
            output.status.success(),
            "fixture: {fixture}\nstdout:\n{}\nstderr:\n{}",
            stdout(&output),
            stderr(&output)
        );
    }

    let output = check_fixture("jsonl/invalid_shape", &["--format", "json"]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    let report: Value = serde_json::from_slice(&output.stdout).expect("json report parses");
    let violation = report["violations"]
        .as_array()
        .expect("violations array")
        .iter()
        .find(|violation| violation["rule"] == "content_runtime:invalid_object_shape")
        .expect("shape violation is emitted");
    assert_eq!(json_path(&violation["path"]), "goals/goals.jsonl");
    assert!(violation["message"]
        .as_str()
        .expect("message")
        .contains("field=status"));
}

#[test]
fn creates_and_updates_yaml_records_deterministically() {
    let repo = fixture_repo("yaml/valid");
    let repository = repository(repo.path());

    repository
        .create_record(
            repo.path(),
            CreateRecordRequest {
                collection: "specs".to_string(),
                id: "spec-alpha".to_string(),
                path: PathBuf::from("specs/spec_alpha.yml"),
                data: data(json!({
                    "title": "Alpha spec",
                    "status": "draft"
                })),
                body: None,
            },
        )
        .expect("valid YAML create succeeds");

    assert_eq!(
        read(repo.path(), "specs/spec_alpha.yml"),
        "id: spec-alpha\nstatus: draft\ntitle: Alpha spec\n"
    );

    repository
        .update_record(
            repo.path(),
            update_request(
                "specs",
                "spec-portable-structure",
                json!({
                    "status": "complete",
                    "title": "Portable structure updated"
                }),
                false,
            ),
        )
        .expect("valid YAML update succeeds");

    let updated = read(repo.path(), "specs/spec_portable_structure.yml");
    assert_eq!(
        updated,
        "id: spec-portable-structure\nstatus: complete\ntitle: Portable structure updated\n"
    );
    assert!(!updated.contains("Adapter writes normalize YAML"));
}

#[test]
fn creates_and_updates_jsonl_records_without_dropping_unrelated_records() {
    let repo = fixture_repo("jsonl/valid");
    let repository = repository(repo.path());

    repository
        .create_record(
            repo.path(),
            CreateRecordRequest {
                collection: "specs".to_string(),
                id: "spec-alpha".to_string(),
                path: PathBuf::from("specs/specs.jsonl"),
                data: data(json!({
                    "title": "Alpha spec",
                    "status": "draft"
                })),
                body: None,
            },
        )
        .expect("valid JSONL create succeeds");

    assert_eq!(
        read(repo.path(), "specs/specs.jsonl"),
        concat!(
            "{\"id\":\"spec-alpha\",\"status\":\"draft\",\"title\":\"Alpha spec\"}\n",
            "{\"id\":\"spec-portable-structure\",\"status\":\"draft\",\"title\":\"Portable structure and frontmatter\"}\n",
            "{\"id\":\"spec-zeta\",\"status\":\"draft\",\"title\":\"Unrelated zeta spec\"}\n"
        )
    );

    let before_dry_run = tree_snapshot(repo.path());
    let dry_run = repository
        .update_record(
            repo.path(),
            update_request(
                "specs",
                "spec-portable-structure",
                json!({
                    "status": "complete",
                    "title": "Portable structure updated"
                }),
                true,
            ),
        )
        .expect("JSONL dry-run update succeeds")
        .dry_run
        .expect("dry run returns content");
    assert_eq!(
        dry_run.content,
        concat!(
            "{\"id\":\"spec-alpha\",\"status\":\"draft\",\"title\":\"Alpha spec\"}\n",
            "{\"id\":\"spec-portable-structure\",\"status\":\"complete\",\"title\":\"Portable structure updated\"}\n",
            "{\"id\":\"spec-zeta\",\"status\":\"draft\",\"title\":\"Unrelated zeta spec\"}\n"
        )
    );
    assert_eq!(tree_snapshot(repo.path()), before_dry_run);

    repository
        .update_record(
            repo.path(),
            update_request(
                "specs",
                "spec-portable-structure",
                json!({
                    "status": "complete",
                    "title": "Portable structure updated"
                }),
                false,
            ),
        )
        .expect("valid JSONL update succeeds");
    assert_eq!(read(repo.path(), "specs/specs.jsonl"), dry_run.content);
}

#[test]
fn adapter_validation_failures_leave_tree_unchanged() {
    let yaml = fixture_repo("yaml/valid");
    let yaml_repo = repository(yaml.path());
    let yaml_before = tree_snapshot(yaml.path());
    let yaml_error = expect_create_error(yaml_repo.create_record(
        yaml.path(),
        CreateRecordRequest {
            collection: "specs".to_string(),
            id: "spec-invalid".to_string(),
            path: PathBuf::from("specs/spec_invalid.yml"),
            data: data(json!({
                "title": "Invalid spec",
                "status": "unknown"
            })),
            body: None,
        },
    ));
    assert!(has_code(&yaml_error, "invalid_object_shape"));
    assert_eq!(tree_snapshot(yaml.path()), yaml_before);

    let jsonl = fixture_repo("jsonl/valid");
    let jsonl_repo = repository(jsonl.path());
    let jsonl_before = tree_snapshot(jsonl.path());
    let jsonl_error = expect_update_error(jsonl_repo.update_record(
        jsonl.path(),
        update_request(
            "specs",
            "spec-portable-structure",
            json!({
                "status": "unknown"
            }),
            false,
        ),
    ));
    assert!(has_code(&jsonl_error, "invalid_object_shape"));
    assert_eq!(tree_snapshot(jsonl.path()), jsonl_before);
}

fn validate_fixture(name: &str) -> assura::content_repository::RepositoryValidation {
    let root = fixture_root(name);
    repository(&root).validate(&root)
}

fn repository(root: &Path) -> ContentRepository {
    let config =
        ConfigLoader::load(&root.join(".assura/config.yml")).expect("fixture config loads");
    ContentRepository::from_config(root, &config).expect("content repository compiles")
}

fn fixture_root(name: &str) -> PathBuf {
    PathBuf::from(ADAPTER_FIXTURE_ROOT).join(name)
}

fn fixture_repo(name: &str) -> TempDir {
    let temp = tempfile::tempdir().expect("temp dir is available");
    copy_dir(&fixture_root(name), temp.path());
    temp
}

fn copy_dir(source: &Path, destination: &Path) {
    for entry in walkdir::WalkDir::new(source) {
        let entry = entry.expect("fixture walk succeeds");
        let rel_path = entry
            .path()
            .strip_prefix(source)
            .expect("entry is under fixture");
        if rel_path.as_os_str().is_empty() {
            continue;
        }
        let target = destination.join(rel_path);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).expect("fixture directory copies");
        } else {
            fs::create_dir_all(target.parent().expect("target has parent"))
                .expect("fixture parent copies");
            fs::copy(entry.path(), target).expect("fixture file copies");
        }
    }
}

fn update_request(
    collection: &str,
    id: &str,
    changes: Value,
    dry_run: bool,
) -> UpdateRecordRequest {
    UpdateRecordRequest {
        collection: collection.to_string(),
        id: id.to_string(),
        path: None,
        changes: data(changes),
        dry_run,
    }
}

fn data(value: Value) -> Map<String, Value> {
    match value {
        Value::Object(map) => map,
        _ => panic!("test data must be an object"),
    }
}

fn expect_create_error(
    result: Result<assura::content_repository::CreateRecordResult, Vec<ContentFinding>>,
) -> Vec<ContentFinding> {
    match result {
        Ok(_) => panic!("create should fail"),
        Err(findings) => findings,
    }
}

fn expect_update_error(
    result: Result<assura::content_repository::UpdateRecordResult, Vec<ContentFinding>>,
) -> Vec<ContentFinding> {
    match result {
        Ok(_) => panic!("update should fail"),
        Err(findings) => findings,
    }
}

fn has_code(findings: &[ContentFinding], code: &str) -> bool {
    findings.iter().any(|finding| finding.code == code)
}

fn read(root: &Path, rel_path: &str) -> String {
    fs::read_to_string(root.join(rel_path)).expect("fixture file reads")
}

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn check_fixture(fixture: &str, args: &[&str]) -> Output {
    let mut command = Command::new(assura_bin());
    command
        .arg("check")
        .arg(fixture_root(fixture))
        .args(args)
        .output()
        .unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n")
}

fn json_path(value: &Value) -> String {
    value.as_str().unwrap().replace('\\', "/")
}

#[derive(Debug, PartialEq, Eq)]
enum TreeEntry {
    Directory,
    File(Vec<u8>),
}

fn tree_snapshot(root: &Path) -> BTreeMap<String, TreeEntry> {
    let mut entries = BTreeMap::new();
    for entry in walkdir::WalkDir::new(root).sort_by_file_name() {
        let entry = entry.expect("tree snapshot walk succeeds");
        let rel_path = entry
            .path()
            .strip_prefix(root)
            .expect("entry is under root");
        if rel_path.as_os_str().is_empty() {
            continue;
        }
        let key = rel_path.to_string_lossy().replace('\\', "/");
        let value = if entry.file_type().is_dir() {
            TreeEntry::Directory
        } else {
            TreeEntry::File(fs::read(entry.path()).expect("snapshot file is readable"))
        };
        entries.insert(key, value);
    }
    entries
}
