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
