use alkemyst_core::{
    db::{Engine, Mutation, Write},
    history::{GameQuery, History, MatchMode, MissingData},
    ingest::Ingestor,
};
use serde_json::json;
#[tokio::test]
async fn truncation_and_connection_facts_remain_visible_as_missing_data() {
    let db = Engine::memory(4096).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    std::fs::write(&path, include_str!("fixtures/arena-game.txt")).unwrap();
    Ingestor::new(db.clone()).import_file(&path).await.unwrap();
    let h = History::new(db.clone());
    let id = h
        .games(GameQuery::default())
        .await
        .unwrap()
        .items
        .remove(0)
        .id;
    for (key, kind) in [
        ("gap-marker", "truncated"),
        ("disconnect-marker", "connection"),
    ] {
        db.write(vec![Write{statement:Mutation{sql:"INSERT INTO capture_events(id,match_id,game_id,kind,payload) SELECT ?1,match_id,id,?2,'{}' FROM games WHERE id=?3",writes:&["capture_events"]},params:vec![json!(key),json!(kind),json!(id)],expected_rows:Some(1)}]).await.unwrap();
    }
    let summary = h.game(&id).await.unwrap().unwrap();
    assert!(!summary.capture_complete);
    assert!(summary.missing_data.contains(&MissingData::Truncated));
    assert!(!summary.missing_data.contains(&MissingData::Connection));
}
#[tokio::test]
async fn unknown_seats_and_unverifiable_chains_are_not_promoted_to_complete_captures() {
    let text = format!(
        "[UnityCrossThreadLogger] {}\n",
        json!({"greToClientEvent":{"greToClientMessages":[{"type":"GREMessageType_GameStateMessage","gameStateId":2,"systemSeatIds":[1,2],"gameStateMessage":{"type":"GameStateType_Diff","gameInfo":{"matchID":"unverified","gameNumber":1}}}]}})
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    std::fs::write(&path, text).unwrap();
    let db = Engine::memory(4096).unwrap();
    Ingestor::new(db.clone()).import_file(&path).await.unwrap();
    let game = History::new(db)
        .games(GameQuery::default())
        .await
        .unwrap()
        .items
        .remove(0);
    assert_eq!(game.mode, MatchMode::Unknown);
    assert!(!game.capture_complete);
    assert!(game.on_play.is_none());
    assert!(game.mulligans.is_none());
    assert!(game.land_drops_observed.is_none());
    for reason in [
        MissingData::InitialState,
        MissingData::TerminalResult,
        MissingData::DeckVersion,
        MissingData::LocalSeat,
        MissingData::UnverifiableChain,
    ] {
        assert!(game.missing_data.contains(&reason), "{reason:?}");
    }
}
