//! Regression tests for watch rescan classification.

use super::*;
use notify::event::Flag;
use std::fs;
use std::sync::mpsc::sync_channel;

#[test]
fn directory_create_forces_full_validation_only_in_the_requested_scope() {
    let project = tempfile::tempdir().unwrap();
    fs::create_dir(project.path().join(".assura")).unwrap();
    fs::create_dir(project.path().join("src")).unwrap();
    fs::create_dir(project.path().join("docs")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        config_with_naming("kebab-case"),
    )
    .unwrap();
    for directory in ["src", "docs"] {
        fs::write(
            project.path().join(directory).join("BadName.ts"),
            "export {};\n",
        )
        .unwrap();
    }
    let root = project.path().canonicalize().unwrap();
    let scope = root.join("src");
    let mut prepared =
        PreparedStructureCheck::load_for_path(Some(scope.clone()), None, false).unwrap();
    let context = WatchContext {
        root: root.clone(),
        watch_scope: scope.clone(),
        watch_scope_is_file: false,
        config_path: root.join(".assura/config.yml"),
        config_watch_parent: Some(root.join(".assura")),
        no_git: false,
    };
    let dirty = DirtyState::new();
    dirty.take();
    let mut batch = WatchBatch::default();
    record_message(
        WatchMessage::Event(
            Event::new(EventKind::Create(notify::event::CreateKind::Folder))
                .add_path(scope.clone()),
        ),
        &context,
        &prepared,
        &dirty,
        &mut batch,
    );
    assert_eq!(batch.invalidating_events, 1);
    let taken = dirty.take();
    assert_eq!(taken.project, DirtyProject::Full);
    let event = serde_json::to_value(validate_batch(
        2,
        100,
        &context,
        &mut prepared,
        taken,
        batch,
        true,
    ))
    .unwrap();
    assert_eq!(event["runtime_mode"], "warm_full");
    assert_eq!(event["fallback_reason"], "full_rescan_event");
    assert_eq!(event["report_scope"], "requested_path");
    assert_eq!(event["changed_paths"], serde_json::json!([]));
    assert_eq!(
        event["report"]["checked_path"],
        scope.to_string_lossy().replace('\\', "/")
    );
    assert_eq!(event["report"]["success"], false);
    let violations = event["report"]["violations"].as_array().unwrap();
    assert!(violations
        .iter()
        .any(|violation| violation["rule"] == "file_naming"
            && violation["path"]
                .as_str()
                .is_some_and(|path| path.replace('\\', "/").ends_with("src/BadName.ts"))));
    assert!(violations.iter().all(|violation| !violation["path"]
        .as_str()
        .unwrap()
        .replace('\\', "/")
        .contains("docs/")));
}

#[test]
fn root_folder_reports_retain_failed_file_evidence_with_or_without_its_event() {
    for include_file_event in [false, true] {
        let project = tempfile::tempdir().unwrap();
        fs::create_dir(project.path().join(".assura")).unwrap();
        fs::create_dir(project.path().join("src")).unwrap();
        fs::write(
            project.path().join(".assura/config.yml"),
            config_with_naming("kebab-case"),
        )
        .unwrap();
        let root = project.path().canonicalize().unwrap();
        let source = root.join("src/BadName.ts");
        let mut prepared =
            PreparedStructureCheck::load_for_path(Some(root.clone()), None, false).unwrap();
        assert!(prepared.check_path(root.clone()).unwrap().success);
        fs::write(&source, "export {};\n").unwrap();
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
            WatchMessage::Event(
                Event::new(EventKind::Create(notify::event::CreateKind::Folder))
                    .add_path(root.clone()),
            ),
            &context,
            &prepared,
            &dirty,
            &mut batch,
        );
        let capture = take_normalization_capture().unwrap();
        assert_eq!(display_paths(&root, &capture.paths), vec![""]);
        assert_eq!(capture.kind, "Create(Folder)");
        assert!(capture.invalidated);
        assert!(!capture.needs_rescan && !capture.config_changed);
        if include_file_event {
            record_message(
                WatchMessage::Event(
                    Event::new(EventKind::Create(notify::event::CreateKind::File))
                        .add_path(source.clone()),
                ),
                &context,
                &prepared,
                &dirty,
                &mut batch,
            );
            assert_eq!(
                display_paths(&root, &take_normalization_capture().unwrap().paths),
                vec!["src/BadName.ts"]
            );
        }
        let event = serde_json::to_value(validate_batch(
            2,
            100,
            &context,
            &mut prepared,
            dirty.take(),
            batch,
            true,
        ))
        .unwrap();
        assert_eq!(
            event["coalesced_events"],
            if include_file_event { 2 } else { 1 }
        );
        assert_eq!(event["runtime_mode"], "warm_full");
        assert_eq!(event["fallback_reason"], "full_rescan_event");
        assert_eq!(event["report_scope"], "requested_path");
        assert_eq!(event["changed_paths"], serde_json::json!([]));
        assert_eq!(
            event["report"]["checked_path"],
            root.to_string_lossy().replace('\\', "/")
        );
        assert_eq!(event["report"]["success"], false);
        let violations = event["report"]["violations"].as_array().unwrap();
        assert!(violations
            .iter()
            .any(|violation| violation["rule"] == "file_naming"
                && violation["path"] == "src/BadName.ts"));
    }
}

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

#[test]
fn replay_external_native_batch_and_rescan_controls() {
    use notify::event::{CreateKind, DataChange, MetadataKind, ModifyKind};

    for scenario in ["external", "mixed_root", "pathless", "changed_config"] {
        let project = tempfile::tempdir().unwrap();
        let config_home = tempfile::tempdir().unwrap();
        let root = project.path().canonicalize().unwrap();
        let config_home = config_home.path().canonicalize().unwrap();
        let config_path = config_home.join("assura.yml");
        let sibling = config_home.join("unrelated.yml");
        fs::write(&config_path, config_with_naming("kebab-case")).unwrap();
        fs::write(root.join("good-name.ts"), "export {};\n").unwrap();
        let mut prepared = PreparedStructureCheck::load_for_path(
            Some(root.clone()),
            Some(config_path.clone()),
            false,
        )
        .unwrap();
        assert!(prepared.check_path(root.clone()).unwrap().success);
        let context = external_config_context(root.clone(), config_path.clone(), &config_home);
        let dirty = DirtyState::new();
        dirty.take();
        let mut batch = WatchBatch::default();
        // Exact event classes from the one-shot native capture, rebased to this fixture.
        for kind in [
            EventKind::Create(CreateKind::File),
            EventKind::Modify(ModifyKind::Metadata(MetadataKind::Extended)),
            EventKind::Modify(ModifyKind::Data(DataChange::Content)),
        ] {
            record_message(
                WatchMessage::Event(Event::new(kind).add_path(sibling.clone())),
                &context,
                &prepared,
                &dirty,
                &mut batch,
            );
        }
        assert_eq!(batch.invalidating_events, 0, "{scenario}");
        assert_eq!(dirty.take().project, DirtyProject::Clean, "{scenario}");
        let event = match scenario {
            "external" => continue,
            "mixed_root" => Event::new(EventKind::Create(CreateKind::Folder))
                .add_path(sibling)
                .add_path(root.clone()),
            "pathless" => Event::new(EventKind::Any).set_flag(Flag::Rescan),
            "changed_config" => {
                fs::write(&config_path, config_with_naming("snake_case")).unwrap();
                Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Content)))
                    .add_path(config_path)
            }
            _ => unreachable!(),
        };
        record_message(
            WatchMessage::Event(event),
            &context,
            &prepared,
            &dirty,
            &mut batch,
        );
        assert_eq!(batch.invalidating_events, 1, "{scenario}");
        let capture = take_normalization_capture().unwrap();
        assert!(capture.invalidated, "{scenario}");
        if scenario == "mixed_root" {
            assert_eq!(display_paths(&root, &capture.paths), vec![""]);
            assert!(!capture.needs_rescan);
        } else if scenario == "pathless" {
            assert!(capture.paths.is_empty() && capture.needs_rescan);
        }
        let event = serde_json::to_value(validate_batch(
            2,
            100,
            &context,
            &mut prepared,
            dirty.take(),
            batch,
            true,
        ))
        .unwrap();
        assert_eq!(event["coalesced_events"], 1, "{scenario}");
        assert_eq!(event["runtime_mode"], "warm_full", "{scenario}");
        assert_eq!(event["report_scope"], "requested_path", "{scenario}");
        if scenario == "changed_config" {
            assert_eq!(event["trigger"], "config");
            assert_eq!(event["cache_state"], "reloaded");
            assert_eq!(event["report"]["success"], false);
            assert!(event["report"]["violations"]
                .as_array()
                .unwrap()
                .iter()
                .any(|violation| violation["rule"] == "file_naming"
                    && violation["path"] == "good-name.ts"));
        } else {
            assert_eq!(event["trigger"], "filesystem", "{scenario}");
            assert_eq!(event["fallback_reason"], "full_rescan_event", "{scenario}");
            assert_eq!(event["report"]["success"], true, "{scenario}");
        }
    }
}

#[test]
fn queued_root_event_remains_visible_on_either_side_of_initial_scan() {
    for mutation_before_initial in [false, true] {
        let project = tempfile::tempdir().unwrap();
        let config_home = tempfile::tempdir().unwrap();
        let root = project.path().canonicalize().unwrap();
        let config_path = config_home.path().join("assura.yml");
        fs::write(&config_path, config_with_naming("kebab-case")).unwrap();
        let mut prepared = PreparedStructureCheck::load_for_path(
            Some(root.clone()),
            Some(config_path.clone()),
            false,
        )
        .unwrap();
        let context = external_config_context(root.clone(), config_path, config_home.path());
        let dirty = DirtyState::new();
        dirty.take();
        let write_violation = || fs::write(root.join("BadName.ts"), "export {};\n").unwrap();
        let (sender, receiver) = sync_channel(1);
        let root_event = || {
            WatchMessage::Event(
                Event::new(EventKind::Create(notify::event::CreateKind::Folder))
                    .add_path(root.clone()),
            )
        };
        if mutation_before_initial {
            write_violation();
            sender.send(root_event()).unwrap();
        }
        let initial = prepared.check_path(root.clone()).unwrap();
        assert_eq!(initial.success, !mutation_before_initial);
        if !mutation_before_initial {
            write_violation();
            sender.send(root_event()).unwrap();
        }
        // Model run_watch's queued callback processing after the initial report.
        // An initial report is not an event-queue fence in either ordering.
        let mut batch = WatchBatch::default();
        record_message(
            receiver.recv().unwrap(),
            &context,
            &prepared,
            &dirty,
            &mut batch,
        );
        assert_eq!(batch.invalidating_events, 1);
        let event = serde_json::to_value(validate_batch(
            2,
            100,
            &context,
            &mut prepared,
            dirty.take(),
            batch,
            initial.success,
        ))
        .unwrap();
        assert_eq!(event["trigger"], "filesystem");
        assert_eq!(event["fallback_reason"], "full_rescan_event");
        assert_eq!(event["report"]["success"], false);
        assert!(event["report"]["violations"]
            .as_array()
            .unwrap()
            .iter()
            .any(
                |violation| violation["rule"] == "file_naming" && violation["path"] == "BadName.ts"
            ));
    }
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
