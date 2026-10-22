use alkemyst_core::{
    db::Engine,
    ingest::{Ingestor, TrackingState},
};
use std::time::Duration;

#[tokio::test]
async fn automatic_tracking_waits_for_logs_reports_errors_and_can_stop() {
    let dir = tempfile::tempdir().unwrap();
    let previous = dir.path().join("Player-prev.log");
    let current = dir.path().join("Player.log");
    let ingest = Ingestor::new(Engine::memory(4096).unwrap());
    let task = ingest.watch(previous, current.clone());
    let mut health = task.subscribe();
    assert!(!task.is_finished());
    tokio::time::timeout(Duration::from_secs(3), health.changed())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(health.borrow().state, TrackingState::Waiting);
    std::fs::write(&current, [0xff, b'\n']).unwrap();
    tokio::time::timeout(Duration::from_secs(3), async {
        while health.borrow().state != TrackingState::Degraded {
            health.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    assert_eq!(
        health.borrow().last_error.as_deref(),
        Some("invalid_capture")
    );
    std::fs::write(&current, include_str!("fixtures/arena-game.txt")).unwrap();
    tokio::time::timeout(Duration::from_secs(3), async {
        while health.borrow().state != TrackingState::Tracking {
            health.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
    assert_eq!(ingest.status().await.unwrap().games, 1);
    task.stop().await.unwrap();
    assert_eq!(health.borrow().state, TrackingState::Stopped);
}

#[tokio::test]
async fn dropping_tracking_task_stops_the_background_loop() {
    let dir = tempfile::tempdir().unwrap();
    let previous = dir.path().join("previous.log");
    let current = dir.path().join("current.log");
    let ingest = Ingestor::new(Engine::memory(4096).unwrap());
    let task = ingest.watch(previous, current);
    let mut health = task.subscribe();
    drop(task);
    tokio::time::timeout(Duration::from_secs(3), async {
        while health.borrow().state != TrackingState::Stopped {
            health.changed().await.unwrap();
        }
    })
    .await
    .unwrap();
}
