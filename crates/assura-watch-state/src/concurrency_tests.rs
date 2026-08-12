use super::{DirtyProject, DirtyState};
use notify::event::CreateKind;
use notify::{Event, EventKind};
use std::sync::{mpsc, Arc, Barrier};
use std::time::Duration;

#[test]
fn status_publication_cannot_race_a_new_event_into_a_stale_clean_result() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(DirtyState::new());
    let taken = state.take();
    let publication_entered = Arc::new(Barrier::new(2));
    let release_publication = Arc::new(Barrier::new(2));

    let publishing_state = Arc::clone(&state);
    let publishing_entered = Arc::clone(&publication_entered);
    let publishing_release = Arc::clone(&release_publication);
    let publisher = std::thread::spawn(move || {
        publishing_state.with_change_state_since(taken.generation, |changed| {
            assert!(!changed);
            publishing_entered.wait();
            publishing_release.wait();
        });
    });

    publication_entered.wait();
    let recording_state = Arc::clone(&state);
    let changed_path = temp.path().join("late.ts");
    let config_path = temp.path().join(".assura/config.yml");
    let (recorded_tx, recorded_rx) = mpsc::channel();
    let recorder = std::thread::spawn(move || {
        recording_state.record_event(
            &Event::new(EventKind::Create(CreateKind::File)).add_path(changed_path),
            &config_path,
        );
        recorded_tx.send(()).unwrap();
    });

    assert!(recorded_rx.recv_timeout(Duration::from_millis(50)).is_err());
    release_publication.wait();
    recorded_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    publisher.join().unwrap();
    recorder.join().unwrap();

    assert!(state.changed_since(taken.generation));
    assert!(matches!(state.take().project, DirtyProject::Paths(_)));
}

#[test]
fn event_publication_cannot_race_validation_into_stale_dirty_state() {
    let temp = tempfile::tempdir().unwrap();
    let state = Arc::new(DirtyState::new());
    state.take();
    let publication_entered = Arc::new(Barrier::new(2));
    let release_publication = Arc::new(Barrier::new(2));
    let changed_path = temp.path().join("late.ts");
    let config_path = temp.path().join(".assura/config.yml");

    let recording_state = Arc::clone(&state);
    let recording_entered = Arc::clone(&publication_entered);
    let recording_release = Arc::clone(&release_publication);
    let recorder = std::thread::spawn(move || {
        recording_state.record_event_with_publication(
            &Event::new(EventKind::Create(CreateKind::File)).add_path(changed_path),
            &config_path,
            || {
                recording_entered.wait();
                recording_release.wait();
            },
        );
    });

    publication_entered.wait();
    let consuming_state = Arc::clone(&state);
    let (taken_tx, taken_rx) = mpsc::channel();
    let consumer = std::thread::spawn(move || {
        taken_tx.send(consuming_state.take()).unwrap();
    });

    assert!(taken_rx.recv_timeout(Duration::from_millis(50)).is_err());
    release_publication.wait();
    let taken = taken_rx.recv_timeout(Duration::from_secs(1)).unwrap();
    recorder.join().unwrap();
    consumer.join().unwrap();

    assert!(matches!(taken.project, DirtyProject::Paths(_)));
    assert!(!state.changed_since(taken.generation));
}
