use alkemyst_core::{
    db::{Engine, Mutation, Write},
    ingest::Ingestor,
};
use serde_json::json;
use std::io::Write as _;

fn state() -> String {
    format!(
        "[UnityCrossThreadLogger]2026-10-21 12:00:00\n{}\n",
        json!({"greToClientEvent":{"greToClientMessages":[{"type":"GREMessageType_GameStateMessage","gameStateId":1,"systemSeatIds":[2],"gameStateMessage":{"type":"GameStateType_Full","gameInfo":{"matchID":"partial-match","gameNumber":1},"turnInfo":{"turnNumber":1,"activePlayer":2}}}]}})
    )
}
async fn mutation(db: &Engine, sql: &'static str) {
    db.write(vec![Write {
        statement: Mutation { sql, writes: &[] },
        params: vec![],
        expected_rows: None,
    }])
    .await
    .unwrap();
}

#[tokio::test]
async fn partial_lines_and_multiline_entries_resume_after_restart() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    let text = state();
    let cut = text.len() / 2;
    std::fs::write(&path, &text[..cut]).unwrap();
    let db = Engine::open(&dir.path().join("capture.db"), 4096).unwrap();
    let first = Ingestor::new(db.clone()).poll_file(&path).await.unwrap();
    assert_eq!(first.events_seen, 0);
    assert!(first.awaiting_partial);
    db.close().await.unwrap();
    let db = Engine::open(&dir.path().join("capture.db"), 4096).unwrap();
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    file.write_all(&text.as_bytes()[cut..text.len() - 1])
        .unwrap();
    let ingest = Ingestor::new(db.clone());
    assert_eq!(ingest.poll_file(&path).await.unwrap().events_seen, 0);
    file.write_all(b"\n").unwrap();
    let complete = ingest.poll_file(&path).await.unwrap();
    assert!(complete.events_seen > 0);
    assert!(!complete.awaiting_partial);
    assert_eq!(ingest.status().await.unwrap().games, 1);
    assert_eq!(ingest.status().await.unwrap().unknown_results, 1);
    assert_eq!(ingest.poll_file(&path).await.unwrap().events_seen, 0);
    db.close().await.unwrap();
}

#[tokio::test]
async fn failed_event_commit_rolls_back_games_and_checkpoint_then_retries() {
    let db = Engine::memory(4096).unwrap();
    mutation(&db,"CREATE TRIGGER reject_capture BEFORE INSERT ON capture_events BEGIN SELECT RAISE(ABORT, 'test interruption'); END").await;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    std::fs::write(&path, state()).unwrap();
    let ingest = Ingestor::new(db.clone());
    assert!(ingest.poll_file(&path).await.is_err());
    let status = ingest.status().await.unwrap();
    assert_eq!(
        (status.sources, status.matches, status.games, status.events),
        (0, 0, 0, 0)
    );
    mutation(&db, "DROP TRIGGER reject_capture").await;
    ingest.poll_file(&path).await.unwrap();
    let status = ingest.status().await.unwrap();
    assert_eq!((status.sources, status.matches, status.games), (1, 1, 1));
    assert!(status.events > 0);
    assert_eq!(ingest.poll_file(&path).await.unwrap().events_seen, 0);
}

#[tokio::test]
async fn replacement_and_truncation_reset_source_context_without_duplication() {
    let db = Engine::memory(4096).unwrap();
    let ingest = Ingestor::new(db);
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    std::fs::write(&path, state()).unwrap();
    ingest.poll_file(&path).await.unwrap();
    let events = ingest.status().await.unwrap().events;
    std::fs::rename(&path, dir.path().join("Player-prev.log")).unwrap();
    std::fs::write(&path, state()).unwrap();
    assert!(ingest.poll_file(&path).await.unwrap().rotated);
    assert_eq!(ingest.status().await.unwrap().events, events);
    std::fs::write(&path, "[UnityCrossThreadLogger]").unwrap();
    assert!(ingest.poll_file(&path).await.unwrap().rotated);
    assert_eq!(ingest.status().await.unwrap().games, 1);
}

#[tokio::test]
async fn shrinking_uncommitted_file_does_not_read_past_its_end() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    std::fs::write(&path, "a".repeat(200)).unwrap();
    let ingest = Ingestor::new(Engine::memory(4096).unwrap());
    assert!(ingest.poll_file(&path).await.unwrap().awaiting_partial);
    std::fs::write(&path, "short").unwrap();
    assert!(ingest.poll_file(&path).await.unwrap().rotated);
}

#[tokio::test]
async fn large_history_streams_in_batches_and_oversized_entries_leave_checkpoint_intact() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    let text = ("x".repeat(1024) + "\n").repeat(8500) + &state();
    std::fs::write(&path, &text).unwrap();
    let ingest = Ingestor::new(Engine::memory(4096).unwrap());
    let report = ingest.import_file(&path).await.unwrap();
    assert!(report.caught_up);
    assert_eq!(report.bytes_committed, text.len() as u64);
    let events = ingest.status().await.unwrap().events;
    std::fs::write(&path, vec![b'x'; 8 * 1024 * 1024 + 1]).unwrap();
    assert!(matches!(
        ingest.poll_file(&path).await,
        Err(alkemyst_core::ingest::Error::Invalid(_))
    ));
    assert_eq!(ingest.status().await.unwrap().events, events);
    std::fs::write(&path, state()).unwrap();
    assert!(ingest.poll_file(&path).await.unwrap().rotated);
    assert_eq!(ingest.status().await.unwrap().games, 1);
}
