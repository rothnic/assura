//! Unit tests for watch batching, invalidation, and fallback decisions.

use super::*;
use std::fs;

#[test]
fn validation_detects_config_content_change_without_config_event() {
    let project = tempfile::tempdir().unwrap();
    fs::create_dir(project.path().join(".assura")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        config_with_naming("kebab-case"),
    )
    .unwrap();
    let source = project.path().join("good-name.ts");
    fs::write(&source, "export {};\n").unwrap();
    let mut prepared =
        PreparedStructureCheck::load_for_path(Some(project.path().to_path_buf()), None, false)
            .unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        config_with_naming("snake_case"),
    )
    .unwrap();
    let context = test_watch_context(project.path());

    let event = validate_batch(
        2,
        100,
        &context,
        &mut prepared,
        DirtyTake {
            generation: 1,
            config_changed: false,
            project: DirtyProject::Paths(vec![source]),
        },
        WatchBatch {
            invalidating_events: 1,
            watcher_error: None,
            watcher_failed: false,
            max_window_reached: false,
        },
        true,
    );
    let event = serde_json::to_value(event).unwrap();

    assert_eq!(event["trigger"], "config");
    assert_eq!(event["cache_state"], "reloaded");
    assert_eq!(event["report_scope"], "requested_path");
    assert_eq!(event["report"]["success"], false);
}

#[test]
fn watcher_overflow_forces_an_observable_full_scope_fallback() {
    let project = tempfile::tempdir().unwrap();
    fs::create_dir(project.path().join(".assura")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        config_with_naming("kebab-case"),
    )
    .unwrap();
    fs::write(project.path().join("good-name.ts"), "export {};\n").unwrap();
    let mut prepared =
        PreparedStructureCheck::load_for_path(Some(project.path().to_path_buf()), None, false)
            .unwrap();
    let context = test_watch_context(project.path());

    let event = validate_batch(
        2,
        100,
        &context,
        &mut prepared,
        DirtyTake {
            generation: 1,
            config_changed: false,
            project: DirtyProject::Full,
        },
        WatchBatch {
            invalidating_events: 257,
            watcher_error: Some("event_channel_overflow".into()),
            watcher_failed: false,
            max_window_reached: false,
        },
        true,
    );
    let event = serde_json::to_value(event).unwrap();

    assert_eq!(event["runtime_mode"], "warm_full");
    assert_eq!(event["report_scope"], "requested_path");
    assert_eq!(event["fallback_reason"], "event_channel_overflow");
    assert_eq!(event["report"]["success"], true);
}

#[test]
fn assura_parent_events_only_invalidate_when_config_content_changed() {
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
    let context = test_watch_context(project.path());
    let dirty = DirtyState::new();
    dirty.take();
    let mut batch = WatchBatch::default();

    record_message(
        WatchMessage::Event(
            Event::new(EventKind::Modify(notify::event::ModifyKind::Any))
                .add_path(project.path().join(".assura")),
        ),
        &context,
        &prepared,
        &dirty,
        &mut batch,
    );
    assert_eq!(batch.invalidating_events, 0);
    assert_eq!(dirty.take().project, DirtyProject::Clean);

    fs::write(
        project.path().join(".assura/config.yml"),
        config_with_naming("snake_case"),
    )
    .unwrap();
    record_message(
        WatchMessage::Event(
            Event::new(EventKind::Modify(notify::event::ModifyKind::Any))
                .add_path(project.path().join(".assura")),
        ),
        &context,
        &prepared,
        &dirty,
        &mut batch,
    );
    let taken = dirty.take();
    assert!(taken.config_changed);
    assert_eq!(taken.project, DirtyProject::Full);
}

#[test]
fn watcher_backend_error_marks_terminal_full_rescan() {
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
    let context = test_watch_context(project.path());
    let dirty = DirtyState::new();
    dirty.take();
    let mut batch = WatchBatch::default();

    record_message(
        WatchMessage::Error("backend subscription lost".into()),
        &context,
        &prepared,
        &dirty,
        &mut batch,
    );

    assert!(batch.watcher_failed);
    assert_eq!(
        batch.watcher_error.as_deref(),
        Some("backend subscription lost")
    );
    assert_eq!(batch.invalidating_events, 1);
    assert_eq!(dirty.take().project, DirtyProject::Full);
}

fn test_watch_context(root: &Path) -> WatchContext {
    WatchContext {
        root: root.to_path_buf(),
        watch_scope: root.to_path_buf(),
        config_path: root.join(".assura/config.yml"),
        config_watch_parent: None,
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
