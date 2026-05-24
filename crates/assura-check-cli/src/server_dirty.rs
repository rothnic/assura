//! Dirty-state tracking for the hot validation daemon.

use notify::event::{AccessKind, AccessMode, CreateKind, EventKind, ModifyKind, RemoveKind};
use notify::Event;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const MAX_INCREMENTAL_DIRTY_PATHS: usize = 64;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DirtyProject {
    Clean,
    Full,
    Paths(Vec<PathBuf>),
}

#[derive(Debug)]
pub(crate) struct DirtyTake {
    pub(crate) config_changed: bool,
    pub(crate) project: DirtyProject,
}

#[derive(Debug)]
pub(crate) struct DirtyState {
    inner: Mutex<DirtySnapshot>,
}

pub(crate) fn dirty_project_paths(
    project: DirtyProject,
    explicit_path: PathBuf,
) -> Option<Vec<PathBuf>> {
    match project {
        DirtyProject::Clean => Some(vec![explicit_path]),
        DirtyProject::Paths(mut paths) => {
            if !paths_match_any(&paths, &explicit_path) {
                paths.push(explicit_path);
            }
            Some(paths)
        }
        DirtyProject::Full => None,
    }
}

fn paths_match_any(paths: &[PathBuf], path: &Path) -> bool {
    paths
        .iter()
        .any(|existing| paths_refer_to_same_change(existing, path))
}

fn paths_refer_to_same_change(left: &Path, right: &Path) -> bool {
    left == right || left.ends_with(right) || right.ends_with(left)
}

#[derive(Debug)]
struct DirtySnapshot {
    config_changed: bool,
    project_changed: bool,
    full_project: bool,
    paths: Vec<PathBuf>,
}

impl DirtyState {
    pub(crate) fn new() -> Self {
        Self {
            inner: Mutex::new(DirtySnapshot {
                config_changed: false,
                project_changed: true,
                full_project: true,
                paths: Vec::new(),
            }),
        }
    }

    pub(crate) fn record_event(&self, event: &Event, config_path: &Path) {
        match classify_event(event, config_path) {
            DirtyEvent::Ignore => {}
            DirtyEvent::Config => self.mark_config_changed(),
            DirtyEvent::Full => self.mark_full_project(),
            DirtyEvent::Paths(paths) => self.mark_paths(paths),
        }
    }

    pub(crate) fn mark_clean_after_initial_check(&self) {
        let mut snapshot = self.inner.lock().expect("dirty state poisoned");
        snapshot.project_changed = false;
        snapshot.full_project = false;
        snapshot.paths.clear();
    }

    pub(crate) fn take(&self) -> DirtyTake {
        let mut snapshot = self.inner.lock().expect("dirty state poisoned");
        let config_changed = std::mem::take(&mut snapshot.config_changed);
        let project = if !snapshot.project_changed {
            DirtyProject::Clean
        } else if snapshot.full_project || config_changed || snapshot.paths.is_empty() {
            DirtyProject::Full
        } else {
            DirtyProject::Paths(std::mem::take(&mut snapshot.paths))
        };
        snapshot.project_changed = false;
        snapshot.full_project = false;
        snapshot.paths.clear();
        DirtyTake {
            config_changed,
            project,
        }
    }

    pub(crate) fn config_changed(&self) -> bool {
        self.inner
            .lock()
            .expect("dirty state poisoned")
            .config_changed
    }

    fn mark_config_changed(&self) {
        let mut snapshot = self.inner.lock().expect("dirty state poisoned");
        snapshot.config_changed = true;
        snapshot.project_changed = true;
        snapshot.full_project = true;
        snapshot.paths.clear();
    }

    fn mark_full_project(&self) {
        let mut snapshot = self.inner.lock().expect("dirty state poisoned");
        snapshot.project_changed = true;
        snapshot.full_project = true;
        snapshot.paths.clear();
    }

    fn mark_paths(&self, paths: Vec<PathBuf>) {
        let mut snapshot = self.inner.lock().expect("dirty state poisoned");
        snapshot.project_changed = true;
        if snapshot.full_project {
            return;
        }
        for path in paths {
            if snapshot.paths.iter().any(|existing| existing == &path) {
                continue;
            }
            snapshot.paths.push(path);
            if snapshot.paths.len() > MAX_INCREMENTAL_DIRTY_PATHS {
                snapshot.full_project = true;
                snapshot.paths.clear();
                return;
            }
        }
    }
}

enum DirtyEvent {
    Ignore,
    Config,
    Full,
    Paths(Vec<PathBuf>),
}

fn classify_event(event: &Event, config_path: &Path) -> DirtyEvent {
    if event.need_rescan() {
        return DirtyEvent::Full;
    }
    if event.paths.iter().any(|path| path == config_path) {
        return DirtyEvent::Config;
    }

    match event.kind {
        EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
            existing_file_paths_or_full(event)
        }
        EventKind::Access(_) => DirtyEvent::Ignore,
        EventKind::Create(CreateKind::File) | EventKind::Remove(RemoveKind::File) => {
            event_paths_or_full(event)
        }
        EventKind::Create(CreateKind::Any | CreateKind::Other)
        | EventKind::Modify(ModifyKind::Any | ModifyKind::Other)
        | EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Metadata(_))
        | EventKind::Modify(ModifyKind::Name(_)) => existing_file_paths_or_full(event),
        EventKind::Create(CreateKind::Folder)
        | EventKind::Remove(_)
        | EventKind::Any
        | EventKind::Other => DirtyEvent::Full,
    }
}

fn event_paths_or_full(event: &Event) -> DirtyEvent {
    if event.paths.is_empty() {
        DirtyEvent::Full
    } else {
        DirtyEvent::Paths(event.paths.clone())
    }
}

fn existing_file_paths_or_full(event: &Event) -> DirtyEvent {
    if event.paths.is_empty() {
        return DirtyEvent::Full;
    }

    let mut saw_file = false;
    for path in &event.paths {
        if path.is_dir() {
            return DirtyEvent::Full;
        }
        if path.is_file()
            || path
                .symlink_metadata()
                .is_ok_and(|metadata| metadata.is_symlink())
        {
            saw_file = true;
        }
    }

    if saw_file {
        DirtyEvent::Paths(event.paths.clone())
    } else {
        DirtyEvent::Full
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::DataChange;

    #[test]
    fn access_events_do_not_dirty_project() {
        let temp = tempfile::tempdir().unwrap();
        let state = DirtyState::new();
        state.mark_clean_after_initial_check();
        state.record_event(
            &Event::new(EventKind::Access(AccessKind::Read))
                .add_path(temp.path().join("src").join("file.ts")),
            &temp.path().join(".assura/config.yml"),
        );

        assert_eq!(state.take().project, DirtyProject::Clean);
    }

    #[test]
    fn write_close_access_tracks_incremental_path() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("file.ts");
        std::fs::write(&path, "").unwrap();
        let state = DirtyState::new();
        state.mark_clean_after_initial_check();
        state.record_event(
            &Event::new(EventKind::Access(AccessKind::Close(AccessMode::Write)))
                .add_path(path.clone()),
            &temp.path().join(".assura/config.yml"),
        );

        assert_eq!(state.take().project, DirtyProject::Paths(vec![path]));
    }

    #[test]
    fn file_create_tracks_incremental_path() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("src").join("file.ts");
        let state = DirtyState::new();
        state.mark_clean_after_initial_check();
        state.record_event(
            &Event::new(EventKind::Create(CreateKind::File)).add_path(path.clone()),
            &temp.path().join(".assura/config.yml"),
        );

        assert_eq!(state.take().project, DirtyProject::Paths(vec![path]));
    }

    #[test]
    fn folder_create_requires_full_project_check() {
        let temp = tempfile::tempdir().unwrap();
        let state = DirtyState::new();
        state.mark_clean_after_initial_check();
        state.record_event(
            &Event::new(EventKind::Create(CreateKind::Folder)).add_path(temp.path().join("src")),
            &temp.path().join(".assura/config.yml"),
        );

        assert_eq!(state.take().project, DirtyProject::Full);
    }

    #[test]
    fn config_changes_request_config_reload_and_full_check() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join(".assura/config.yml");
        let state = DirtyState::new();
        state.mark_clean_after_initial_check();
        state.record_event(
            &Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Content)))
                .add_path(config.clone()),
            &config,
        );

        let dirty = state.take();
        assert!(dirty.config_changed);
        assert_eq!(dirty.project, DirtyProject::Full);
    }

    #[test]
    fn config_changed_check_does_not_clear_project_dirty_state() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join(".assura/config.yml");
        let state = DirtyState::new();
        state.mark_clean_after_initial_check();
        state.record_event(
            &Event::new(EventKind::Modify(ModifyKind::Data(DataChange::Content)))
                .add_path(config.clone()),
            &config,
        );

        assert!(state.config_changed());
        assert!(state.config_changed());
        let dirty = state.take();
        assert!(dirty.config_changed);
        assert_eq!(dirty.project, DirtyProject::Full);
    }

    #[test]
    fn dirty_project_paths_uses_explicit_path_when_clean() {
        assert_eq!(
            dirty_project_paths(DirtyProject::Clean, PathBuf::from("src/file.ts")),
            Some(vec![PathBuf::from("src/file.ts")])
        );
    }

    #[test]
    fn dirty_project_paths_preserves_pending_watcher_paths() {
        assert_eq!(
            dirty_project_paths(
                DirtyProject::Paths(vec![PathBuf::from("src/other.ts")]),
                PathBuf::from("src/file.ts"),
            ),
            Some(vec![
                PathBuf::from("src/other.ts"),
                PathBuf::from("src/file.ts")
            ])
        );
    }

    #[test]
    fn dirty_project_paths_deduplicates_explicit_relative_path() {
        assert_eq!(
            dirty_project_paths(
                DirtyProject::Paths(vec![PathBuf::from("/project/src/file.ts")]),
                PathBuf::from("src/file.ts"),
            ),
            Some(vec![PathBuf::from("/project/src/file.ts")])
        );
    }

    #[test]
    fn dirty_project_paths_deduplicates_explicit_absolute_path() {
        assert_eq!(
            dirty_project_paths(
                DirtyProject::Paths(vec![PathBuf::from("src/file.ts")]),
                PathBuf::from("/project/src/file.ts"),
            ),
            Some(vec![PathBuf::from("src/file.ts")])
        );
    }

    #[test]
    fn dirty_project_paths_rejects_full_dirty_state() {
        assert_eq!(
            dirty_project_paths(DirtyProject::Full, PathBuf::from("src/file.ts")),
            None
        );
    }
}
