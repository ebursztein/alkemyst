use alkemyst_core::{db::Engine, ingest::Ingestor};

#[tokio::test]
async fn retained_real_capture_survives_reimport_and_restart_without_duplicates() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("Player-prev.log");
    std::fs::write(&path, include_str!("fixtures/arena-game.txt")).unwrap();
    let db = Engine::open(&directory.path().join("capture.sqlite3"), 4 * 1024 * 1024).unwrap();
    let ingest = Ingestor::new(db.clone());
    let report = ingest.import_file(&path).await.unwrap();
    assert!(report.events_seen > 0);
    let first = ingest.status().await.unwrap();
    assert_eq!(first.matches, 1);
    assert_eq!(first.games, 1);
    assert_eq!(first.deck_versions, 3);
    assert_eq!(ingest.import_file(&path).await.unwrap().events_seen, 0);
    let copied = directory.path().join("Player.log");
    std::fs::copy(&path, &copied).unwrap();
    ingest.import_file(&copied).await.unwrap();
    let second = ingest.status().await.unwrap();
    assert_eq!(first.events, second.events);
    assert_eq!(second.games, 1);
    db.close().await.unwrap();
    let reopened = Engine::open(&directory.path().join("capture.sqlite3"), 4096).unwrap();
    let restarted = Ingestor::new(reopened.clone());
    assert_eq!(restarted.import_file(&copied).await.unwrap().events_seen, 0);
    assert_eq!(restarted.status().await.unwrap().events, first.events);
    reopened.close().await.unwrap();
}
