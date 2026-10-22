use super::{
    models::{Context, Event},
    privacy,
};
use manasight_parser::{GameEvent, content_hash};
use serde_json::{Value, json};

pub(super) fn event(event: &GameEvent, context: &mut Context, source: &str) -> Option<Event> {
    let p = event.payload();
    let kind = match event {
        GameEvent::GameState(_) => "state",
        GameEvent::ClientAction(_) if p["type"] != "client_ui_message" => "action",
        GameEvent::GameResult(_) => "result",
        GameEvent::MatchState(_) => {
            if p["type"] == "match_completed" {
                "match_end"
            } else {
                "match_start"
            }
        }
        GameEvent::LocalSeat(_) => "seat",
        GameEvent::Truncation(_) => "truncated",
        GameEvent::DetailedLoggingStatus(_) => "logging",
        GameEvent::MatchConnectionState(_)
        | GameEvent::TcpConnectionClose(_)
        | GameEvent::WebSocketClosed(_)
        | GameEvent::ConnectionError(_) => "connection",
        _ => return None,
    };
    let match_id = p["match_id"]
        .as_str()
        .filter(|id| !id.is_empty())
        .or_else(|| p["game_info"]["matchID"].as_str());
    if let Some(id) = match_id {
        let hashed = content_hash(id.as_bytes());
        if context.match_id.as_ref() != Some(&hashed) {
            if !context.pending_deck {
                context.deck = None;
            }
            context.pending_deck = false;
            context.match_id = Some(hashed);
            context.game_number = None;
            context.local_seat = None;
            context.teams.clear();
        }
    }
    if let Some(number) = p["game_info"]["gameNumber"]
        .as_u64()
        .and_then(|n| u32::try_from(n).ok())
        .filter(|n| *n > 0)
    {
        if context.game_number != Some(number) {
            context.pending_deck = false;
        }
        context.game_number = Some(number);
    }
    if kind == "match_start" || kind == "match_end" {
        for player in p["players"].as_array().into_iter().flatten() {
            if let (Some(seat), Some(team)) = (
                positive(&player["system_seat_id"]),
                positive(&player["team_id"]),
            ) {
                context.teams.insert(seat, team);
            }
        }
    }
    let wire = super::wire::state(event);
    if let Some(wire) = &wire {
        if let Some(seat) = wire.local_seat {
            context.local_seat = Some(seat);
        }
        for player in wire.body["players"].as_array().into_iter().flatten() {
            if let (Some(seat), Some(team)) = (
                positive(&player["systemSeatNumber"])
                    .or_else(|| positive(&player["controllerSeatId"])),
                positive(&player["teamId"]),
            ) {
                context.teams.insert(seat, team);
            }
        }
    }
    if kind == "seat" {
        context.local_seat = positive(&p["system_seat_id"]);
    }
    let game_id = context
        .match_id
        .as_ref()
        .zip(context.game_number)
        .map(|(id, number)| format!("{id}:{number}"));
    let mut payload = privacy::gameplay(p);
    if kind == "connection" {
        let interrupted = match event {
            GameEvent::MatchConnectionState(_) => match (p["old"].as_str(), p["new"].as_str()) {
                (Some("MatchCompleted" | "None"), Some("Disconnected")) => Some(false),
                (_, Some("Disconnected")) => Some(true),
                (
                    _,
                    Some(
                        "Playing"
                        | "MatchCompleted"
                        | "ConnectedToMatchDoor"
                        | "ConnectedToMatchDoor_ConnectingToGRE"
                        | "ConnectedToMatchDoor_ConnectedToGRE_Waiting",
                    ),
                ) => Some(false),
                _ => None,
            },
            GameEvent::TcpConnectionClose(_)
            | GameEvent::WebSocketClosed(_)
            | GameEvent::ConnectionError(_) => Some(true),
            _ => None,
        };
        payload["capture_interrupted"] = json!(interrupted);
    }
    if matches!(event, GameEvent::MatchState(_)) {
        // Explicit known queues only. Unknown/custom queue labels are not retained.
        payload["best_of"] = json!(match p["event_id"].as_str() {
            Some("Ladder") => Some(1),
            Some("Traditional_Ladder") => Some(3),
            _ => None,
        });
    }
    if let Some(wire) = &wire {
        if wire.body["players"].is_array() {
            payload["players"] = privacy::gameplay(&wire.body["players"]);
        }
        if kind == "result" {
            payload["prev_game_state_id"] = wire.body["prevGameStateId"].clone();
            payload["turn_info"] = privacy::gameplay(&super::wire::turn(&wire.body));
        }
    }
    if kind == "state" {
        // Timers are transport noise; state IDs and gameplay facts remain stable
        // when Arena resends a state after reconnecting.
        payload.as_object_mut()?.remove("timers");
    }
    if kind == "action" {
        let raw = &p["raw_client_action"]["payload"];
        let raw = if let Some(text) = raw.as_str() {
            serde_json::from_str(text).unwrap_or(Value::Null)
        } else {
            raw.clone()
        };
        payload["action"] = privacy::gameplay(&raw);
    }
    let identity = if kind == "state" && positive(&p["game_state_id"]).is_some() {
        json!([
            context.match_id,
            context.game_number,
            kind,
            p["game_state_id"]
        ])
    } else {
        json!([
            context.match_id,
            context.game_number,
            kind,
            payload,
            if context.match_id.is_none() {
                Some(source)
            } else {
                None
            }
        ])
    };
    let id = content_hash(identity.to_string().as_bytes());
    payload["local_seat"] = json!(context.local_seat);
    let local_team = context
        .local_seat
        .and_then(|seat| context.teams.get(&seat).copied());
    payload["local_team"] = json!(local_team);
    if kind == "result" {
        let outcome = crate::domain::ObservedOutcome::from_result(
            p["result_type"].as_str(),
            p["winning_team_id"].as_i64(),
            local_team,
        );
        payload["outcome"] = json!(outcome);
    }
    Some(Event {
        id,
        match_id: context.match_id.clone(),
        game_id,
        game_number: context.game_number,
        deck_version: context.deck.as_ref().map(|deck| deck.version_id.clone()),
        kind,
        state_id: positive(&p["game_state_id"])
            .or_else(|| wire.as_ref().and_then(|state| state.state_id)),
        payload,
        local_time: event.metadata().local_timestamp().map(|t| t.to_string()),
        instant_utc: event.metadata().instant_utc().map(|t| t.to_rfc3339()),
    })
}
fn positive(value: &Value) -> Option<i64> {
    value.as_i64().filter(|id| *id > 0)
}
