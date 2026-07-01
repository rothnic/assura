use assura::daemon::{DaemonCoreError, DaemonHealthState, LocalDaemonCore};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

#[test]
fn daemon_core_reports_running_health_and_one_shot_fallback() {
    let project = daemon_project();
    let mut daemon = LocalDaemonCore::load(project.path().to_path_buf(), None).unwrap();

    let health = daemon.health();
    assert_eq!(health.state, DaemonHealthState::Running);
    assert_eq!(health.generation, 1);
    assert_eq!(
        health.config_path,
        project.path().join(".assura/config.yml")
    );
    assert_eq!(
        health.runtime_paths.status_file,
        project.path().join(".assura/daemon/status.json")
    );
    assert!(health
        .fallback_command
        .contains("assura check --format json"));
    assert!(
        health
            .fallback_command
            .contains(project.path().file_name().unwrap().to_str().unwrap()),
        "{}",
        health.fallback_command
    );

    let structure = daemon
        .check_changed_path(project.path().join("docs/note.md"))
        .unwrap();
    assert!(structure.success, "{structure:#?}");
}

#[test]
fn daemon_core_fallback_preserves_explicit_config_path() {
    let project = daemon_project();
    let config_path = project.path().join(".assura/custom.yml");
    fs::write(&config_path, config("snake_case")).unwrap();

    let daemon =
        LocalDaemonCore::load(project.path().to_path_buf(), Some(config_path.clone())).unwrap();
    let fallback = daemon.health().fallback_command;

    assert!(fallback.contains("--config"), "{fallback}");
    assert!(fallback.contains(".assura/custom.yml"), "{fallback}");
    assert!(
        fallback.contains(project.path().file_name().unwrap().to_str().unwrap()),
        "{fallback}"
    );
}

#[test]
fn daemon_core_explicit_config_outside_assura_uses_checked_project_root() {
    let project = daemon_project();
    let config_path = project.path().join("assura-alt.yml");
    fs::write(&config_path, config("kebab-case")).unwrap();

    let mut daemon =
        LocalDaemonCore::load(project.path().to_path_buf(), Some(config_path.clone())).unwrap();
    let source = daemon
        .changed_source_references(PathBuf::from("docs/note.md"), 10)
        .unwrap();
    let health = daemon.health();

    assert_eq!(health.project_root, project.path().canonicalize().unwrap());
    assert_eq!(health.config_path, config_path.canonicalize().unwrap());
    assert!(health.fallback_command.contains("--config"));
    assert_eq!(source.bounds.returned, 3);
    assert!(source
        .references
        .iter()
        .any(|reference| reference.target_path == Path::new("docs/guide.md")));
}

#[test]
fn daemon_core_config_change_invalidates_cached_state_until_refresh() {
    let project = daemon_project();
    let mut daemon = LocalDaemonCore::load(project.path().to_path_buf(), None).unwrap();
    let original_generation = daemon.health().generation;

    fs::write(
        project.path().join(".assura/config.yml"),
        config("snake_case"),
    )
    .unwrap();

    let error = daemon
        .changed_source_references(PathBuf::from("docs/note.md"), 10)
        .expect_err("config changes must not return trusted daemon success");
    let DaemonCoreError::Stale(health) = error else {
        panic!("expected stale config error");
    };
    assert_eq!(health.state, DaemonHealthState::Stale);
    assert!(health.reason.contains("configuration changed"));

    let refreshed = daemon.refresh().unwrap();
    assert_eq!(refreshed.state, DaemonHealthState::Running);
    assert!(refreshed.generation > original_generation);
}

#[test]
fn daemon_core_missing_config_returns_structured_stale_health() {
    let project = daemon_project();
    let mut daemon = LocalDaemonCore::load(project.path().to_path_buf(), None).unwrap();

    fs::remove_file(project.path().join(".assura/config.yml")).unwrap();

    let error = daemon
        .changed_source_references(PathBuf::from("docs/note.md"), 10)
        .expect_err("missing config must not return trusted daemon success");
    let DaemonCoreError::Stale(health) = error else {
        panic!("expected stale config error");
    };
    assert_eq!(health.state, DaemonHealthState::Stale);
    assert!(health.reason.contains("configuration unavailable"));
    assert!(health
        .fallback_command
        .contains("assura check --format json"));
}

#[test]
fn daemon_core_reports_changed_source_and_target_reference_context() {
    let project = daemon_project();
    let mut daemon = LocalDaemonCore::load(project.path().to_path_buf(), None).unwrap();

    let source = daemon
        .changed_source_references(PathBuf::from("docs/note.md"), 10)
        .unwrap();
    assert_eq!(source.mode, "source");
    assert_eq!(source.path, PathBuf::from("docs/note.md"));
    assert_eq!(source.health.state, DaemonHealthState::Running);
    assert_eq!(source.bounds.returned, 3);
    assert!(!source.bounds.truncated);
    assert!(source
        .references
        .iter()
        .any(
            |reference| reference.target_path == Path::new("docs/guide.md")
                && reference.target_anchor.as_deref() == Some("install")
                && reference.target_exists
        ));
    assert!(source
        .references
        .iter()
        .any(|reference| reference.target_path == Path::new("src/lib.rs")
            && reference.target_line_start == Some(1)
            && reference.target_line_end == Some(2)));

    let target = daemon
        .changed_target_references(PathBuf::from("docs/guide.md"), 10)
        .unwrap();
    assert_eq!(target.mode, "target");
    assert_eq!(target.path, PathBuf::from("docs/guide.md"));
    assert_eq!(target.bounds.returned, 1);
    assert_eq!(
        target.references[0].source_path,
        PathBuf::from("docs/note.md")
    );
    assert_eq!(
        target.references[0].target_anchor.as_deref(),
        Some("install")
    );
}

#[test]
fn daemon_core_changed_source_refreshes_mutated_source_and_matches_cli() {
    let project = daemon_project();
    let mut daemon = LocalDaemonCore::load(project.path().to_path_buf(), None).unwrap();

    fs::write(project.path().join("docs/other.md"), "# Other\n").unwrap();
    fs::write(
        project.path().join("docs/note.md"),
        "# Note\n\nSee [guide](guide.md#install) and [other](other.md).\n",
    )
    .unwrap();

    let source = daemon
        .changed_source_references(PathBuf::from("docs/note.md"), 10)
        .unwrap();
    let cli = content_references_json(
        project.path(),
        &["--source", "docs/note.md", "--format", "json"],
    );
    let cli_paths = cli_reference_target_paths(&cli);
    let daemon_paths = source
        .references
        .iter()
        .map(|reference| portable_path(&reference.target_path))
        .collect::<Vec<_>>();

    assert_eq!(source.health.state, DaemonHealthState::Running);
    assert_eq!(daemon_paths, cli_paths);
    assert_eq!(
        daemon_paths,
        vec!["docs/guide.md".to_string(), "docs/other.md".to_string()]
    );
}

#[test]
fn daemon_core_changed_target_uses_prior_graph_after_target_delete() {
    let project = daemon_project();
    let mut daemon = LocalDaemonCore::load(project.path().to_path_buf(), None).unwrap();

    fs::remove_file(project.path().join("docs/guide.md")).unwrap();
    let target = daemon
        .changed_target_references(PathBuf::from("docs/guide.md"), 10)
        .unwrap();

    assert_eq!(target.health.state, DaemonHealthState::Degraded);
    assert!(target.health.reason.contains("prior warm reference graph"));
    assert_eq!(target.bounds.returned, 1);
    assert_eq!(
        target.references[0].source_path,
        PathBuf::from("docs/note.md")
    );
    assert_eq!(
        target.references[0].target_path,
        PathBuf::from("docs/guide.md")
    );
}

#[test]
fn daemon_core_changed_target_move_uses_prior_graph_with_new_path_context() {
    let project = daemon_project();
    let mut daemon = LocalDaemonCore::load(project.path().to_path_buf(), None).unwrap();

    fs::rename(
        project.path().join("docs/guide.md"),
        project.path().join("docs/guide-renamed.md"),
    )
    .unwrap();
    let moved = daemon
        .moved_target_references(
            PathBuf::from("docs/guide.md"),
            PathBuf::from("docs/guide-renamed.md"),
            10,
        )
        .unwrap();

    assert_eq!(moved.health.state, DaemonHealthState::Degraded);
    assert_eq!(moved.previous_path, PathBuf::from("docs/guide.md"));
    assert_eq!(moved.new_path, PathBuf::from("docs/guide-renamed.md"));
    assert_eq!(moved.bounds.returned, 1);
    assert_eq!(
        moved.references[0].source_path,
        PathBuf::from("docs/note.md")
    );
}

#[test]
fn daemon_core_exposes_non_running_health_responses() {
    let project = daemon_project();
    let daemon = LocalDaemonCore::load(project.path().to_path_buf(), None).unwrap();
    let config_path = project.path().join(".assura/config.yml");

    assert_eq!(
        daemon.warming_health("warming reference graph").state,
        DaemonHealthState::Warming
    );
    assert_eq!(
        assura::daemon::DaemonHealth::unavailable(
            project.path().to_path_buf(),
            config_path.clone(),
            "daemon process not running"
        )
        .state,
        DaemonHealthState::Unavailable
    );
    assert_eq!(
        assura::daemon::DaemonHealth::incompatible(
            project.path().to_path_buf(),
            config_path,
            "client protocol is newer than daemon protocol"
        )
        .state,
        DaemonHealthState::Incompatible
    );
}

#[test]
fn daemon_core_reference_responses_are_bounded() {
    let project = daemon_project();
    let mut daemon = LocalDaemonCore::load(project.path().to_path_buf(), None).unwrap();

    let response = daemon
        .changed_source_references(project.path().join("docs/note.md"), 1)
        .unwrap();

    assert_eq!(response.bounds.limit, 1);
    assert_eq!(response.bounds.returned, 1);
    assert!(response.bounds.truncated);
    assert_eq!(response.references.len(), 1);
}

fn daemon_project() -> TempDir {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::create_dir_all(project.path().join("docs")).unwrap();
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        config("kebab-case"),
    )
    .unwrap();
    fs::write(
        project.path().join("docs/note.md"),
        "# Note\n\nSee [guide](guide.md#install), [code](../src/lib.rs#L1-L2), and [missing](missing.md).\n",
    )
    .unwrap();
    fs::write(
        project.path().join("docs/guide.md"),
        "# Guide\n\n## Install\n",
    )
    .unwrap();
    fs::write(
        project.path().join("src/lib.rs"),
        "fn one() {}\nfn two() {}\n",
    )
    .unwrap();
    project
}

fn config(naming: &str) -> String {
    format!(
        r#"
structure:
  docs/:
    files:
      naming_patterns:
        "*.md": {naming}
  src/:
    files:
      naming_patterns:
        "*.rs": snake_case
"#
    )
}

fn content_references_json(project_root: &Path, args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_assura"))
        .arg("content")
        .arg("references")
        .arg(project_root)
        .args(args)
        .output()
        .expect("content references command runs");
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("content references emits JSON")
}

fn cli_reference_target_paths(value: &Value) -> Vec<String> {
    value["references"]
        .as_array()
        .expect("references array")
        .iter()
        .map(|reference| {
            reference["target_path"]
                .as_str()
                .expect("target path")
                .to_string()
        })
        .collect()
}

fn portable_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
