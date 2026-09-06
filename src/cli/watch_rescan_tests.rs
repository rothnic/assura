//! Regression tests for watch rescan classification.

use super::*;
use notify::event::Flag;
use std::fs;

#[test]
fn pathless_rescan_remains_an_observable_full_scope_fallback() {
    let project = tempfile::tempdir().unwrap();
    fs::create_dir(project.path().join(".assura")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        config_with_naming("kebab-case"),
    )
    .unwrap();
    let prepared =
        PreparedStructureCheck::load_for_path(Some(project.path().to_path_buf()), None, false)
            .unwrap();
    let root = project.path().canonicalize().unwrap();
    let context = WatchContext {
        root: root.clone(),
        watch_scope: root.clone(),
        watch_scope_is_file: false,
        config_path: root.join(".assura/config.yml"),
        config_watch_parent: Some(root.join(".assura")),
        no_git: false,
    };
    let dirty = DirtyState::new();
    dirty.take();
    let mut batch = WatchBatch::default();

    record_message(
        WatchMessage::Event(Event::new(EventKind::Any).set_flag(Flag::Rescan)),
        &context,
        &prepared,
        &dirty,
        &mut batch,
    );

    assert_eq!(batch.invalidating_events, 1);
    assert_eq!(dirty.take().project, DirtyProject::Full);
}

#[test]
fn rescan_after_an_unrelated_external_config_sibling_is_ignored() {
    let project = tempfile::tempdir().unwrap();
    let config_home = tempfile::tempdir().unwrap();
    let config_path = config_home.path().join("assura.yml");
    fs::write(&config_path, config_with_naming("kebab-case")).unwrap();
    fs::write(project.path().join("good-name.ts"), "export {};\n").unwrap();
    let prepared = PreparedStructureCheck::load_for_path(
        Some(project.path().to_path_buf()),
        Some(config_path.clone()),
        false,
    )
    .unwrap();
    let root = project.path().canonicalize().unwrap();
    let context = external_config_context(root, config_path, config_home.path());
    let dirty = DirtyState::new();
    dirty.take();
    let mut batch = WatchBatch::default();

    record_message(
        WatchMessage::Event(
            Event::new(EventKind::Any)
                .add_path(config_home.path().join("unrelated.yml"))
                .set_flag(Flag::Rescan),
        ),
        &context,
        &prepared,
        &dirty,
        &mut batch,
    );

    assert_eq!(batch.invalidating_events, 0);
    assert_eq!(dirty.take().project, DirtyProject::Clean);
}

#[test]
fn rescan_coalescing_external_config_and_runtime_output_is_ignored() {
    let project = tempfile::tempdir().unwrap();
    let config_home = tempfile::tempdir().unwrap();
    let config_path = config_home.path().join("assura.yml");
    fs::write(&config_path, config_with_naming("kebab-case")).unwrap();
    fs::write(project.path().join("good-name.ts"), "export {};\n").unwrap();
    let prepared = PreparedStructureCheck::load_for_path(
        Some(project.path().to_path_buf()),
        Some(config_path.clone()),
        false,
    )
    .unwrap();
    let root = project.path().canonicalize().unwrap();
    let context = external_config_context(root.clone(), config_path, config_home.path());
    let dirty = DirtyState::new();
    dirty.take();
    let mut batch = WatchBatch::default();

    record_message(
        WatchMessage::Event(
            Event::new(EventKind::Any)
                .add_path(config_home.path().join("unrelated.yml"))
                .add_path(root.join(".assura/cache/worktree/result.json"))
                .set_flag(Flag::Rescan),
        ),
        &context,
        &prepared,
        &dirty,
        &mut batch,
    );

    assert_eq!(batch.invalidating_events, 0);
    assert_eq!(dirty.take().project, DirtyProject::Clean);
}

fn external_config_context(
    root: PathBuf,
    config_path: PathBuf,
    config_home: &Path,
) -> WatchContext {
    WatchContext {
        root: root.clone(),
        watch_scope: root,
        watch_scope_is_file: false,
        config_path,
        config_watch_parent: Some(config_home.to_path_buf()),
        no_git: false,
    }
}

fn config_with_naming(naming: &str) -> String {
    format!(
        r#"
structure:
  ./:
    files:
      naming_patterns:
        "*.ts": {naming}
"#
    )
}
