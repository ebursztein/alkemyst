use alkemyst_core::{
    db::Engine,
    history::{GameQuery, History},
    ingest::Ingestor,
};
use serde_json::json;
use std::io::Write as _;
#[tokio::test]
async fn matches_are_grouped_without_turning_game_results_into_match_results() {
    let db = Engine::memory(4 * 1024 * 1024).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    std::fs::write(&path, include_str!("fixtures/arena-game.txt")).unwrap();
    let ingest = Ingestor::new(db.clone());
    ingest.import_file(&path).await.unwrap();
    let h = History::new(db);
    let page = h.matches(GameQuery::default()).await.unwrap();
    assert_eq!(page.items.len(), 1);
    let m = &page.items[0];
    assert_eq!(m.game_count, 1);
    assert_eq!(m.games.len(), 1);
    assert!(m.completed);
    assert!(m.games[0].outcome.is_some());
    let loaded = h.match_summary(&m.id).await.unwrap().unwrap();
    assert_eq!(loaded.id, m.id);
    assert!(h.match_summary("missing").await.unwrap().is_none());
    let filtered = h
        .matches(GameQuery {
            deck_id: m.games[0].deck_id.clone(),
            version_id: m.games[0].deck_version.clone(),
            match_id: Some(m.id.clone()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(filtered.items.len(), 1);
    let start = format!(
        "[UnityCrossThreadLogger] {}\n",
        json!({"matchGameRoomStateChangedEvent":{"gameRoomInfo":{"stateType":"MatchGameRoomStateType_Playing","gameRoomConfig":{"matchId":"empty-match","eventId":"Traditional_Ladder"}}}})
    );
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    file.write_all(start.as_bytes()).unwrap();
    ingest.poll_file(&path).await.unwrap();
    let first = h
        .matches(GameQuery {
            limit: 1,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(first.items[0].game_count, 0);
    assert!(!first.items[0].completed);
    assert!(first.items[0].outcome.is_none());
    let second = h
        .matches(GameQuery {
            limit: 1,
            cursor: first.next_cursor,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(second.items[0].id, m.id);
    assert!(second.next_cursor.is_none());
}
