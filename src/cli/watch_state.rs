//! Shared filesystem invalidation state for watch and daemon runtimes.

pub use assura_watch_state::*;
use notify::event::RemoveKind;
use notify::{Event, EventKind};
use std::path::{Path, PathBuf};

pub(super) fn display_paths(root: &Path, paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| {
            path.strip_prefix(root)
                .unwrap_or(path)
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect()
}

pub(super) fn normalize_config_event(
    event: &mut Event,
    config_path: &Path,
    content_changed: impl FnOnce() -> bool,
) -> bool {
    let directly_touches_config = event.paths.iter().any(|path| path == config_path);
    let may_be_config_replacement = !directly_touches_config
        && config_path.parent().is_some_and(|parent| {
            event.paths.iter().any(|path| {
                path == parent || path.parent().is_some_and(|candidate| candidate == parent)
            })
        });
    if !directly_touches_config && !may_be_config_replacement {
        return true;
    }

    let changed = content_changed();
    if directly_touches_config && !changed {
        event.paths.retain(|path| path != config_path);
        return !event.paths.is_empty();
    }
    if may_be_config_replacement && changed {
        event.paths.clear();
        event.paths.push(config_path.to_path_buf());
    }
    true
}

pub(super) fn normalize_known_file_removal(
    event: &mut Event,
    watch_scope: &Path,
    watch_scope_is_file: bool,
) {
    if watch_scope_is_file
        && matches!(
            event.kind,
            EventKind::Remove(RemoveKind::Any | RemoveKind::Other)
        )
        && !event.paths.is_empty()
        && event.paths.iter().all(|path| path == watch_scope)
    {
        event.kind = EventKind::Remove(RemoveKind::File);
    }
}
