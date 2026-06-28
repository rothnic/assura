use assura::config::loader::ConfigLoader;
use assura::content_repository::{ContentFinding, ContentRepository, CreateRecordRequest};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const VALID_FIXTURE: &str = "tests/fixtures/content_runtime/valid";

#[test]
fn creates_valid_markdown_record_after_shape_path_and_reference_validation() {
    let repo = fixture_repo();
    let repository = repository(repo.path());
    let request = CreateRecordRequest {
        collection: "goals".to_string(),
        id: "goal-new".to_string(),
        path: PathBuf::from("docs/goals/goal_new.md"),
        data: data(json!({
            "title": "New portable goal",
            "status": "planned",
            "specs": ["spec-portable-structure"]
        })),
        body: Some(
            "# New Portable Goal\n\nThis goal is written through the content runtime.\n"
                .to_string(),
        ),
    };

    let result = repository
        .create_record(repo.path(), request)
        .expect("valid create succeeds");

    assert_eq!(result.path, PathBuf::from("docs/goals/goal_new.md"));
    assert_eq!(result.validation.findings, Vec::new());
    assert!(result
        .validation
        .snapshot
        .objects
        .contains_key(&("goals".to_string(), "goal-new".to_string())));

    let content = fs::read_to_string(repo.path().join("docs/goals/goal_new.md"))
        .expect("created markdown can be read");
    assert!(content.starts_with("---\n"));
    assert!(content.contains("id: goal-new\n"));
    assert!(content.contains("specs:\n- spec-portable-structure\n"));
    assert!(normalize_newlines(&content)
        .ends_with("# New Portable Goal\n\nThis goal is written through the content runtime.\n"));
}

#[test]
fn creates_valid_json_record_after_shape_and_path_validation() {
    let repo = fixture_repo();
    let repository = repository(repo.path());
    let request = CreateRecordRequest {
        collection: "specs".to_string(),
        id: "spec-new".to_string(),
        path: PathBuf::from("specs/spec_new.json"),
        data: data(json!({
            "title": "New portable spec",
            "status": "draft"
        })),
        body: None,
    };

    let result = repository
        .create_record(repo.path(), request)
        .expect("valid JSON create succeeds");

    assert_eq!(result.path, PathBuf::from("specs/spec_new.json"));
    assert_eq!(result.validation.findings, Vec::new());
    assert!(result
        .validation
        .snapshot
        .objects
        .contains_key(&("specs".to_string(), "spec-new".to_string())));

    let content =
        fs::read_to_string(repo.path().join("specs/spec_new.json")).expect("created JSON exists");
    let value: Value = serde_json::from_str(&content).expect("created JSON parses");
    assert_eq!(
        value,
        json!({
            "id": "spec-new",
            "status": "draft",
            "title": "New portable spec"
        })
    );
    assert!(content.ends_with('\n'));
}

#[test]
fn rejects_invalid_shape_before_writing() {
    let repo = fixture_repo();
    let repository = repository(repo.path());
    let before = tree_snapshot(repo.path());
    let request = CreateRecordRequest {
        collection: "specs".to_string(),
        id: "spec-new".to_string(),
        path: PathBuf::from("specs/spec_new.json"),
        data: data(json!({
            "title": "New Spec",
            "status": "unknown"
        })),
        body: None,
    };

    let findings = expect_create_error(repository.create_record(repo.path(), request));

    assert!(has_code(&findings, "invalid_object_shape"));
    assert_eq!(tree_snapshot(repo.path()), before);
}

#[test]
fn rejects_missing_reference_before_writing() {
    let repo = fixture_repo();
    let repository = repository(repo.path());
    let before = tree_snapshot(repo.path());
    let request = CreateRecordRequest {
        collection: "goals".to_string(),
        id: "goal-missing-reference".to_string(),
        path: PathBuf::from("docs/goals/goal_missing_reference.md"),
        data: data(json!({
            "title": "Missing reference",
            "status": "planned",
            "specs": ["missing-spec"]
        })),
        body: Some("# Missing Reference\n".to_string()),
    };

    let findings = expect_create_error(repository.create_record(repo.path(), request));

    assert!(has_code(&findings, "missing_reference"));
    assert_eq!(tree_snapshot(repo.path()), before);
}

#[test]
fn rejects_duplicate_id_before_writing() {
    let repo = fixture_repo();
    let repository = repository(repo.path());
    let before = tree_snapshot(repo.path());
    let request = CreateRecordRequest {
        collection: "specs".to_string(),
        id: "spec-portable-structure".to_string(),
        path: PathBuf::from("specs/spec_duplicate.json"),
        data: data(json!({
            "title": "Duplicate Spec",
            "status": "draft"
        })),
        body: None,
    };

    let findings = expect_create_error(repository.create_record(repo.path(), request));

    assert!(has_code(&findings, "duplicate_object_id"));
    assert_eq!(tree_snapshot(repo.path()), before);
}

#[test]
fn rejects_existing_destination_before_writing() {
    let repo = fixture_repo();
    let repository = repository(repo.path());
    let before = tree_snapshot(repo.path());
    let request = CreateRecordRequest {
        collection: "specs".to_string(),
        id: "spec-new".to_string(),
        path: PathBuf::from("specs/spec_portable_structure.json"),
        data: data(json!({
            "title": "New Spec",
            "status": "draft"
        })),
        body: None,
    };

    let findings = expect_create_error(repository.create_record(repo.path(), request));

    assert!(has_code(&findings, "content_create_path_exists"));
    assert_eq!(tree_snapshot(repo.path()), before);
}

#[test]
fn rejects_path_outside_collection_policy_before_writing() {
    let repo = fixture_repo();
    let repository = repository(repo.path());
    let before = tree_snapshot(repo.path());
    let request = CreateRecordRequest {
        collection: "specs".to_string(),
        id: "spec-new".to_string(),
        path: PathBuf::from("docs/spec_new.json"),
        data: data(json!({
            "title": "New Spec",
            "status": "draft"
        })),
        body: None,
    };

    let findings = expect_create_error(repository.create_record(repo.path(), request));

    assert!(has_code(&findings, "invalid_object_path"));
    assert_eq!(tree_snapshot(repo.path()), before);
}

fn repository(root: &Path) -> ContentRepository {
    let config =
        ConfigLoader::load(&root.join(".assura/config.yml")).expect("fixture config loads");
    ContentRepository::from_config(root, &config).expect("content repository compiles")
}

fn fixture_repo() -> TempDir {
    let temp = tempfile::tempdir().expect("temp dir is available");
    copy_dir(Path::new(VALID_FIXTURE), temp.path());
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

fn has_code(findings: &[ContentFinding], code: &str) -> bool {
    findings.iter().any(|finding| finding.code == code)
}

fn normalize_newlines(value: &str) -> String {
    value.replace("\r\n", "\n")
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
