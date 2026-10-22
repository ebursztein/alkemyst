//! Recover gameplay fields omitted by the pinned upstream normalized payload.
use manasight_parser::GameEvent;
use serde_json::{Value, json};

pub(super) struct State {
    pub body: Value,
    pub state_id: Option<i64>,
    pub local_seat: Option<i64>,
}
pub(super) fn state(event: &GameEvent) -> Option<State> {
    if !matches!(event, GameEvent::GameState(_) | GameEvent::GameResult(_)) {
        return None;
    }
    let raw = event.metadata().raw_bytes();
    let start = raw.iter().position(|byte| *byte == b'{')?;
    let outer: Value = serde_json::from_slice(&raw[start..]).ok()?;
    let messages = outer["greToClientEvent"]["greToClientMessages"].as_array()?;
    let payload = event.payload();
    let selected = messages.iter().rev().find(|message| {
        if message["gameStateMessage"].is_null() {
            return false;
        }
        if matches!(event, GameEvent::GameResult(_)) {
            message["gameStateMessage"]["gameInfo"]["stage"] == "GameStage_GameOver"
                && message["gameStateMessage"]["gameInfo"]["gameNumber"]
                    == payload["game_info"]["gameNumber"]
        } else {
            message["gameStateId"] == payload["game_state_id"]
                && message["msgId"].as_i64().unwrap_or(0) == payload["msg_id"].as_i64().unwrap_or(0)
        }
    })?;
    let local_seat = messages
        .iter()
        .filter_map(|message| message["systemSeatIds"].as_array())
        .find_map(|seats| {
            if seats.len() == 1 {
                seats[0].as_i64().filter(|seat| *seat > 0)
            } else {
                None
            }
        });
    Some(State {
        body: selected["gameStateMessage"].clone(),
        state_id: selected["gameStateId"].as_i64().filter(|id| *id > 0),
        local_seat,
    })
}
pub(super) fn turn(body: &Value) -> Value {
    let source = &body["turnInfo"];
    if !source.is_object() {
        return Value::Null;
    }
    json!({"turn_number":source["turnNumber"],"active_player":source["activePlayer"],"decision_player":source["decisionPlayer"],"phase":source["phase"],"step":source["step"]})
}
