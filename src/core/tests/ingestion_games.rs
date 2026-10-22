use alkemyst_core::{
    db::{Engine, Mutation, Statement, Write},
    ingest::Ingestor,
};
use serde_json::{Value, json};
fn line(value: Value) -> String {
    format!("[UnityCrossThreadLogger] {value}\n")
}
fn state(number: u32, id: u32, winner: Option<u32>) -> String {
    let mut info = json!({"matchID":"bo3-fixture","gameNumber":number,"stage":"GameStage_Start"});
    if let Some(winner) = winner {
        info["stage"] = json!("GameStage_GameOver");
        info["results"] = json!([{"scope":"MatchScope_Game","result":"ResultType_WinLoss","winningTeamId":winner}]);
    }
    line(
        json!({"greToClientEvent":{"greToClientMessages":[{"type":"GREMessageType_GameStateMessage","gameStateId":id,"systemSeatIds":[2],"gameStateMessage":{"type":"GameStateType_Full","gameInfo":info}}]}}),
    )
}
fn deck() -> String {
    format!(
        "[UnityCrossThreadLogger]==> EventSetDeckV2 {}\n",
        json!({"request":json!({"Deck":{"MainDeck":[{"cardId":42,"quantity":60}]},"Summary":{"DeckId":"deck-fixture","Attributes":[{"name":"Format","value":"Standard"}]}}).to_string()})
    )
}
async fn rows(db: &Engine, sql: &'static str) -> Vec<Value> {
    db.read(
        Statement {
            name: "test.capture",
            sql,
            reads: &[],
            cacheable: false,
        },
        vec![],
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn bo3_separates_games_and_sideboarding_preserves_the_original_version() {
    let start = line(
        json!({"matchGameRoomStateChangedEvent":{"gameRoomInfo":{"stateType":"MatchGameRoomStateType_Playing","gameRoomConfig":{"matchId":"bo3-fixture","reservedPlayers":[{"systemSeatId":1,"teamId":10},{"systemSeatId":2,"teamId":20}]}}}}),
    );
    let sideboard = line(
        json!({"clientToMatchServiceMessageType":"ClientToMatchServiceMessageType_ClientToGREMessage","payload":{"type":"ClientMessageType_SubmitDeckResp","submitDeckResp":{"deck":{"deckCards":vec![43;60],"sideboardCards":[42]}},"gameStateId":2,"respId":9}}),
    );
    let text = deck()
        + &start
        + &state(1, 1, None)
        + &state(1, 2, Some(20))
        + &sideboard
        + &state(2, 1, None)
        + &state(2, 2, Some(0))
        + &state(3, 1, None)
        + &state(3, 2, Some(10));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    std::fs::write(&path, &text).unwrap();
    let db = Engine::memory(4096).unwrap();
    let ingest = Ingestor::new(db.clone());
    ingest.import_file(&path).await.unwrap();
    let status = ingest.status().await.unwrap();
    assert_eq!(
        (
            status.matches,
            status.games,
            status.deck_versions,
            status.unknown_results
        ),
        (1, 3, 2, 1)
    );
    let results = rows(
        &db,
        "SELECT payload FROM capture_events WHERE kind='result' ORDER BY ordinal",
    )
    .await;
    assert_eq!(
        results
            .iter()
            .map(|row| row["outcome"].clone())
            .collect::<Vec<_>>(),
        vec![json!("won"), Value::Null, json!("lost")]
    );
    let games=rows(&db,"SELECT json_object('number',g.number,'cards',json_group_array(c.reference)) FROM games g JOIN deck_cards c ON c.version_id=g.deck_version AND c.zone='main' GROUP BY g.id ORDER BY g.number").await;
    assert_eq!(games[0]["cards"], json!(["arena:42"]));
    assert_eq!(games[1]["cards"], json!(["arena:43"]));
    assert_eq!(games[2]["cards"], json!(["arena:43"]));
    for sql in [
        "UPDATE games SET deck_version=NULL",
        "DELETE FROM capture_events",
        "DELETE FROM matches",
    ] {
        assert!(
            db.write(vec![Write {
                statement: Mutation { sql, writes: &[] },
                params: vec![],
                expected_rows: None
            }])
            .await
            .is_err()
        );
    }
}

#[tokio::test]
async fn missing_team_or_winner_never_becomes_a_loss_and_private_strings_are_dropped() {
    let text = state(1, 1, None)
        + &state(1, 2, Some(1))
        + &line(
            json!({"clientToMatchServiceMessageType":"ClientToMatchServiceMessageType_ClientToGREMessage","payload":{"type":"ClientMessageType_PerformActionResp","gameStateId":3,"action":{"grpId":42,"userId":"private-user","chat":"private-message","token":"private-token"}}}),
        );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    std::fs::write(&path, &text).unwrap();
    let db = Engine::memory(4096).unwrap();
    let ingest = Ingestor::new(db.clone());
    ingest.import_file(&path).await.unwrap();
    let results = rows(
        &db,
        "SELECT payload FROM capture_events WHERE kind='result'",
    )
    .await;
    assert_eq!(results[0]["outcome"], Value::Null);
    let stored = rows(&db, "SELECT payload FROM capture_events").await;
    let encoded = serde_json::to_string(&stored).unwrap();
    for secret in [
        "private-user",
        "private-message",
        "private-token",
        "raw_client_action",
        "userId",
        "token",
    ] {
        assert!(
            !encoded.contains(secret),
            "unexpected retained field: {secret}"
        );
    }
    assert!(encoded.contains("grpId"));
}

#[tokio::test]
async fn a_new_match_cannot_inherit_a_stale_submission_without_new_evidence() {
    let text =
        deck() + &state(1, 1, None) + &state(1, 1, None).replace("bo3-fixture", "next-fixture");
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    std::fs::write(&path, &text).unwrap();
    let db = Engine::memory(4096).unwrap();
    Ingestor::new(db.clone()).import_file(&path).await.unwrap();
    let attribution = rows(
        &db,
        "SELECT json_object('known',count(deck_version),'games',count(*)) FROM games",
    )
    .await;
    assert_eq!(attribution[0], json!({"known":1,"games":2}));
}

#[tokio::test]
async fn connect_response_recovers_an_explicit_deck_when_the_submission_is_missing() {
    let connect = line(
        json!({"greToClientEvent":{"greToClientMessages":[{"type":"GREMessageType_ConnectResp","systemSeatIds":[2],"connectResp":{"deckMessage":{"deckCards":vec![44;60],"sideboardCards":[45]}}}]}}),
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    std::fs::write(&path, connect + &state(1, 1, None)).unwrap();
    let db = Engine::memory(4096).unwrap();
    Ingestor::new(db.clone()).import_file(&path).await.unwrap();
    let attribution=rows(&db,"SELECT json_object('card',c.reference,'count',c.quantity) FROM games g JOIN deck_cards c ON c.version_id=g.deck_version AND c.zone='main'").await;
    assert_eq!(attribution, vec![json!({"card":"arena:44","count":60})]);
    assert!(
        !serde_json::to_string(&rows(&db, "SELECT payload FROM capture_events").await)
            .unwrap()
            .contains("raw_connect_resp")
    );
}

#[tokio::test]
async fn an_explicit_sideboard_list_is_saved_even_without_the_original_submission() {
    let sideboard = line(
        json!({"clientToMatchServiceMessageType":"ClientToMatchServiceMessageType_ClientToGREMessage","payload":{"type":"ClientMessageType_SubmitDeckResp","submitDeckResp":{"deck":{"deckCards":vec![46;60],"sideboardCards":[47]}}}}),
    );
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    std::fs::write(&path, sideboard + &state(2, 1, None)).unwrap();
    let db = Engine::memory(4096).unwrap();
    Ingestor::new(db.clone()).import_file(&path).await.unwrap();
    let attribution=rows(&db,"SELECT json_object('card',c.reference,'count',c.quantity) FROM games g JOIN deck_cards c ON c.version_id=g.deck_version AND c.zone='main'").await;
    assert_eq!(attribution, vec![json!({"card":"arena:46","count":60})]);
}
