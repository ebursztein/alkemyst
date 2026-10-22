use serde_json::{Value, json};
pub fn line(value: Value) -> String {
    format!("[UnityCrossThreadLogger] {value}\n")
}
pub fn start(id: &str, queue: &str) -> String {
    line(
        json!({"matchGameRoomStateChangedEvent":{"gameRoomInfo":{"stateType":"MatchGameRoomStateType_Playing","gameRoomConfig":{"matchId":id,"eventId":queue,"reservedPlayers":[{"systemSeatId":1,"teamId":10},{"systemSeatId":2,"teamId":20}]}}}}),
    )
}
pub fn state(match_id: &str, id: u32, turn: u32, active: u32, winner: Option<u32>) -> String {
    let mut info = json!({"matchID":match_id,"gameNumber":1,"stage":"GameStage_Start"});
    if let Some(winner) = winner {
        info["stage"] = json!("GameStage_GameOver");
        info["results"] = json!([{"scope":"MatchScope_Game","result":"ResultType_WinLoss","winningTeamId":winner}]);
    }
    line(
        json!({"greToClientEvent":{"greToClientMessages":[{"type":"GREMessageType_GameStateMessage","gameStateId":id,"msgId":id,"systemSeatIds":[2],"gameStateMessage":{"type":if id==1{"GameStateType_Full"}else{"GameStateType_Diff"},"prevGameStateId":if id>1{Some(id-1)}else{None},"gameInfo":info,"turnInfo":{"turnNumber":turn,"activePlayer":active},"players":[{"systemSeatNumber":2,"teamId":20,"lifeTotal":20,"landsPlayedThisTurn":1,"mulliganCount":1}]}}]}}),
    )
}
pub fn deck() -> String {
    format!(
        "[UnityCrossThreadLogger]==> EventSetDeckV2 {}\n",
        json!({"request":json!({"Deck":{"MainDeck":[{"cardId":42,"quantity":60}]},"Summary":{"DeckId":"fixture-history","Attributes":[{"name":"Format","value":"Standard"}]}}).to_string()})
    )
}
