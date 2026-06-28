use assura::config::loader::ConfigLoader;
use assura::content_repository::{ContentFinding, ContentRepository, UpdateRecordRequest};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const VALID_FIXTURE: &str = "tests/fixtures/content_runtime/valid";

#[test]
fn updates_markdown_frontmatter_and_preserves_body_bytes() {
    let repo = fixture_repo();
    let repository = repository(repo.path());
    let path = repo.path().join("docs/goals/goal_portable_structure.md");
    let before = fs::read_to_string(&path).expect("fixture markdown reads");
    let before_body = markdown_body(&before).to_vec();
    let request = update_request(
        "goals",
        "goal-portable-structure",
        Some("docs/goals/goal_portable_structure.md"),
        json!({
            "status": "complete",
            "title": "Portable Structure Policy Updated"
        }),
        false,
    );

    let result = repository
        .update_record(repo.path(), request)
        .expect("valid markdown update succeeds");

    assert_eq!(
        result.path,
        PathBuf::from("docs/goals/goal_portable_structure.md")
    );
    assert_eq!(result.validation.findings, Vec::new());
    assert!(result.dry_run.is_none());

    let after = fs::read_to_string(&path).expect("updated markdown reads");
    assert!(after.contains("status: complete\n"));
    assert!(after.contains("title: Portable Structure Policy Updated\n"));
    assert_eq!(markdown_body(&after), before_body.as_slice());
}

#[test]
fn updates_json_record_deterministically() {
    let repo = fixture_repo();
    let repository = repository(repo.path());
    let request = update_request(
        "specs",
        "spec-portable-structure",
        None,
        json!({
            "status": "complete",
            "title": "Portable structure updated"
        }),
        false,
    );

    let result = repository
        .update_record(repo.path(), request)
        .expect("valid JSON update succeeds");

    assert_eq!(
        result.path,
        PathBuf::from("specs/spec_portable_structure.json")
    );
    assert_eq!(result.validation.findings, Vec::new());
    let content = fs::read_to_string(repo.path().join("specs/spec_portable_structure.json"))
        .expect("updated JSON reads");
    assert_eq!(
        serde_json::from_str::<Value>(&content).expect("updated JSON parses"),
        json!({
            "id": "spec-portable-structure",
            "status": "complete",
            "title": "Portable structure updated"
        })
    );
    assert!(content.ends_with('\n'));
}

#[test]
fn dry_run_returns_proposed_content_without_writing() {
    let repo = fixture_repo();
    let repository = repository(repo.path());
    let before = tree_snapshot(repo.path());
    let request = update_request(
        "specs",
        "spec-portable-structure",
        None,
        json!({
            "status": "complete"
        }),
        true,
    );

    let result = repository
        .update_record(repo.path(), request.clone())
        .expect("dry run validates");
    let preview = result.dry_run.expect("dry run preview is returned");

    assert_eq!(
        preview.path,
        PathBuf::from("specs/spec_portable_structure.json")
    );
    assert_eq!(
        preview.content,
        "{\n  \"id\": \"spec-portable-structure\",\n  \"status\": \"complete\",\n  \"title\": \"Portable structure and frontmatter\"\n}\n"
    );
    let repeat = repository
        .update_record(repo.path(), request)
        .expect("repeat dry run validates")
        .dry_run
        .expect("repeat dry run preview is returned");
    assert_eq!(repeat.content, preview.content);
    assert_eq!(tree_snapshot(repo.path()), before);
    let proposed = result
        .validation
        .snapshot
        .objects
        .get(&("specs".to_string(), "spec-portable-structure".to_string()))
        .expect("proposed object exists");
    assert_eq!(proposed.data.get("status"), Some(&json!("complete")));
}

#[test]
fn rejects_invalid_shape_before_writing() {
    let repo = fixture_repo();
    let repository = repository(repo.path());
    let before = tree_snapshot(repo.path());
    let request = update_request(
        "specs",
        "spec-portable-structure",
        None,
        json!({
            "status": "unknown"
        }),
        false,
    );

    let findings = expect_update_error(repository.update_record(repo.path(), request));

    assert!(has_code(&findings, "invalid_object_shape"));
    assert_eq!(tree_snapshot(repo.path()), before);
}

#[test]
fn rejects_missing_reference_before_writing() {
    let repo = fixture_repo();
    let repository = repository(repo.path());
    let before = tree_snapshot(repo.path());
    let request = update_request(
        "goals",
        "goal-portable-structure",
        None,
        json!({
            "specs": ["missing-spec"]
        }),
        false,
    );

    let findings = expect_update_error(repository.update_record(repo.path(), request));

    assert!(has_code(&findings, "missing_reference"));
    assert_eq!(tree_snapshot(repo.path()), before);
}

#[test]
fn rejects_missing_record_before_writing() {
    let repo = fixture_repo();
    let repository = repository(repo.path());
    let before = tree_snapshot(repo.path());
    let request = update_request(
        "specs",
        "missing-spec",
        None,
        json!({
            "status": "complete"
        }),
        false,
    );

    let findings = expect_update_error(repository.update_record(repo.path(), request));

    assert!(has_code(&findings, "content_update_missing_record"));
    assert_eq!(tree_snapshot(repo.path()), before);
}

#[test]
fn rejects_identity_change_before_writing() {
    let repo = fixture_repo();
    let repository = repository(repo.path());
    let before = tree_snapshot(repo.path());
    let request = update_request(
        "specs",
        "spec-portable-structure",
        None,
        json!({
            "id": "spec-renamed"
        }),
        false,
    );

    let findings = expect_update_error(repository.update_record(repo.path(), request));

    assert!(has_code(&findings, "content_update_identity_change"));
    assert_eq!(tree_snapshot(repo.path()), before);
}

#[test]
fn rejects_path_mismatch_before_writing() {
    let repo = fixture_repo();
    let repository = repository(repo.path());
    let before = tree_snapshot(repo.path());
    let request = update_request(
        "specs",
        "spec-portable-structure",
        Some("specs/other_spec.json"),
        json!({
            "status": "complete"
        }),
        false,
    );

    let findings = expect_update_error(repository.update_record(repo.path(), request));

    assert!(has_code(&findings, "content_update_path_mismatch"));
    assert_eq!(tree_snapshot(repo.path()), before);
}

#[cfg(unix)]
#[test]
fn failed_atomic_update_leaves_original_content() {
    use std::os::unix::fs::PermissionsExt;

    let repo = fixture_repo();
    let repository = repository(repo.path());
    let target = repo.path().join("specs/spec_portable_structure.json");
    let parent = target.parent().expect("target has parent");
    let before = fs::read(&target).expect("target reads before failed update");
    let original_mode = fs::metadata(parent)
        .expect("parent metadata reads")
        .permissions()
        .mode();
    let mut readonly = fs::metadata(parent)
        .expect("parent metadata reads")
        .permissions();
    readonly.set_mode(original_mode & !0o222);
    fs::set_permissions(parent, readonly).expect("parent can be made readonly");

    let request = update_request(
        "specs",
        "spec-portable-structure",
        None,
        json!({
            "status": "complete"
        }),
        false,
    );
    let findings = expect_update_error(repository.update_record(repo.path(), request));

    let mut restored = fs::metadata(parent)
        .expect("parent metadata reads")
        .permissions();
    restored.set_mode(original_mode);
    fs::set_permissions(parent, restored).expect("parent permissions restore");

    assert!(has_code(&findings, "write_error"));
    assert_eq!(
        fs::read(&target).expect("target reads after failed update"),
        before
    );
}

fn repository(root: &Path) -> ContentRepository {
    let config =
        ConfigLoader::load(&root.join(".assura/config.yml")).expect("fixture config loads");
    ContentRepository::from_config(root, &config).expect("content repository compiles")
}

fn update_request(
    collection: &str,
    id: &str,
    path: Option<&str>,
    changes: Value,
    dry_run: bool,
) -> UpdateRecordRequest {
    UpdateRecordRequest {
        collection: collection.to_string(),
        id: id.to_string(),
        path: path.map(PathBuf::from),
        changes: data(changes),
        dry_run,
    }
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

fn markdown_body(content: &str) -> &[u8] {
    let (_, body) = content
        .split_once("\n---\n")
        .expect("markdown fixture has closing frontmatter delimiter");
    body.as_bytes()
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
