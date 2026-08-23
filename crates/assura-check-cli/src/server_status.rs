//! Generation-ordered publication for the hot-check status artifact.

use crate::server_dirty::DirtyState;
use crate::status_file::{self, CheckStatus};
use notify::Event;
use std::path::Path;

pub(crate) fn record_event_and_publish_dirty(
    dirty: &DirtyState,
    event: &Event,
    config_path: &Path,
    status_file: Option<&Path>,
) -> bool {
    if status_file.is_some_and(|path| event_touches_status_file(event, path)) {
        return false;
    }

    match status_file {
        Some(path) => dirty
            .record_event_with_publication(event, config_path, || write_status(path, 3, true))
            .is_some(),
        None => dirty.record_event(event, config_path),
    }
}

pub(crate) fn publish_dirty_status(dirty: &DirtyState, path: &Path, exit_code: i32) {
    dirty.serialize_publication(|| write_status(path, exit_code, true));
}

pub(crate) fn publish_status_after_generation(
    dirty: &DirtyState,
    path: &Path,
    exit_code: i32,
    generation: u64,
) {
    dirty.with_change_state_since(generation, |changed| {
        write_status(path, exit_code, changed);
    });
}

fn event_touches_status_file(event: &Event, status_file: &Path) -> bool {
    event
        .paths
        .iter()
        .any(|path| status_file::is_status_artifact(path, status_file))
}

fn write_status(path: &Path, exit_code: i32, dirty: bool) {
    if let Err(error) = status_file::write_status(path, CheckStatus { exit_code, dirty }) {
        eprintln!("Warning: publish status file {}: {error}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::event::CreateKind;
    use notify::EventKind;
    use std::sync::{mpsc, Arc, Barrier};
    use std::time::Duration;

    #[test]
    fn validation_after_event_publication_leaves_actual_status_clean() {
        let temp = tempfile::tempdir().unwrap();
        let status_path = temp.path().join("assura.status");
        let config_path = temp.path().join(".assura/config.yml");
        let changed_path = temp.path().join("changed.ts");
        let state = Arc::new(DirtyState::new());
        state.take();
        write_status(&status_path, 0, false);

        let publication_entered = Arc::new(Barrier::new(2));
        let release_publication = Arc::new(Barrier::new(2));
        let recorder_state = Arc::clone(&state);
        let recorder_path = status_path.clone();
        let recorder_entered = Arc::clone(&publication_entered);
        let recorder_release = Arc::clone(&release_publication);
        let recorder = std::thread::spawn(move || {
            recorder_state.record_event_with_publication(
                &Event::new(EventKind::Create(CreateKind::File)).add_path(changed_path),
                &config_path,
                || {
                    write_status(&recorder_path, 3, true);
                    recorder_entered.wait();
                    recorder_release.wait();
                },
            );
        });

        publication_entered.wait();
        assert!(status_file::read_status(&status_path).unwrap().dirty);
        let consumer_state = Arc::clone(&state);
        let consumer_path = status_path.clone();
        let (completed_tx, completed_rx) = mpsc::channel();
        let consumer = std::thread::spawn(move || {
            let taken = consumer_state.take();
            publish_status_after_generation(&consumer_state, &consumer_path, 0, taken.generation);
            completed_tx.send(()).unwrap();
        });

        assert!(completed_rx
            .recv_timeout(Duration::from_millis(50))
            .is_err());
        release_publication.wait();
        completed_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        recorder.join().unwrap();
        consumer.join().unwrap();

        assert_eq!(
            status_file::read_status(&status_path).unwrap(),
            CheckStatus {
                exit_code: 0,
                dirty: false,
            }
        );
    }
}
