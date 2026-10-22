use alkemyst_core::{
    db::Engine,
    history::{GameQuery, History, StatisticsQuery, TimelineQuery},
    ingest::Ingestor,
};

#[tokio::test]
async fn sparse_real_capture_preserves_known_outcome_and_exposes_gaps() {
    let db = Engine::memory(4 * 1024 * 1024).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    std::fs::write(&path, include_str!("fixtures/arena-game.txt")).unwrap();
    Ingestor::new(db.clone()).import_file(&path).await.unwrap();
    let history = History::new(db.clone());
    let before = db.metrics().await.unwrap().statements;
    let games = history.games(GameQuery::default()).await.unwrap();
    assert_eq!(db.metrics().await.unwrap().statements, before + 1);
    assert_eq!(games.items.len(), 1);
    let game = &games.items[0];
    assert!(!game.capture_complete);
    assert!(!game.missing_data.is_empty());
    assert!(game.outcome.is_some());
    assert!(game.deck_version.is_some());
    assert_eq!(history.game(&game.id).await.unwrap().unwrap().id, game.id);
    let first = history
        .timeline(
            &game.id,
            TimelineQuery {
                limit: 2,
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(first.items.len(), 2);
    assert!(first.next_cursor.is_some());
    let second = history
        .timeline(
            &game.id,
            TimelineQuery {
                limit: 2,
                cursor: first.next_cursor.clone(),
                turn_number: None,
            },
        )
        .await
        .unwrap()
        .unwrap();
    assert!(second.items[0].ordinal > first.items[1].ordinal);
    let stats = history
        .statistics(StatisticsQuery::default())
        .await
        .unwrap();
    assert_eq!(stats.games, 1);
    assert_eq!(stats.known_results, 1);
    assert_eq!(stats.partial_games, 1);
    assert_eq!(stats.losses + stats.wins + stats.draws, 1);
    let warm = db.metrics().await.unwrap().cache_hits;
    history.games(GameQuery::default()).await.unwrap();
    assert_eq!(db.metrics().await.unwrap().cache_hits, warm + 1);
    assert!(history.game("missing").await.unwrap().is_none());
    assert!(
        history
            .timeline("missing", TimelineQuery::default())
            .await
            .unwrap()
            .is_none()
    );
}
