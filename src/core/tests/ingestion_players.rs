use alkemyst_core::{
    db::{Engine, Statement},
    ingest::Ingestor,
};
use serde_json::json;

#[tokio::test]
async fn available_player_state_preserves_lands_life_mulligans_and_team_attribution() {
    let mut text = String::new();
    for (id, over) in [(1, false), (2, true)] {
        let mut info = json!({"matchID":"players-match","gameNumber":1,"stage":"GameStage_Start"});
        if over {
            info["stage"] = json!("GameStage_GameOver");
            info["results"] = json!([{"scope":"MatchScope_Game","result":"ResultType_WinLoss","winningTeamId":20}]);
        }
        let message = json!({"type":"GREMessageType_GameStateMessage","gameStateId":id,"msgId":id,"systemSeatIds":[2],"gameStateMessage":{"type":"GameStateType_Full","gameInfo":info,"players":[{"systemSeatNumber":2,"teamId":20,"lifeTotal":17,"landsPlayedThisTurn":1,"mulliganCount":1,"playerName":"private-name","userId":"private-user"}],"turnInfo":{"turnNumber":1,"activePlayer":2}}});
        text += &format!(
            "[UnityCrossThreadLogger] {}\n",
            json!({"greToClientEvent":{"greToClientMessages":[message]}})
        );
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    std::fs::write(&path, text).unwrap();
    let db = Engine::memory(4096).unwrap();
    Ingestor::new(db.clone()).import_file(&path).await.unwrap();
    let rows=db.read(Statement{name:"test.players",sql:"SELECT json_object('kind',kind,'state_id',state_id,'payload',json(payload)) FROM capture_events WHERE kind IN ('state','result') ORDER BY ordinal",reads:&[],cacheable:false},vec![]).await.unwrap();
    let player = &rows[0]["payload"]["players"][0];
    assert_eq!(player["lifeTotal"], 17);
    assert_eq!(player["landsPlayedThisTurn"], 1);
    assert_eq!(player["mulliganCount"], 1);
    assert_eq!(rows[1]["payload"]["outcome"], "won");
    assert_eq!(rows[1]["state_id"], 2);
    let stored = serde_json::to_string(&rows).unwrap();
    assert!(!stored.contains("private-name"));
    assert!(!stored.contains("private-user"));
}
