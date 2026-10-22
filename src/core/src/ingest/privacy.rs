use serde_json::{Map, Value};

// Preserve gameplay numbers/enums without retaining account/session records,
// player names, chat, arbitrary text or opaque raw envelopes.
pub(super) fn gameplay(value: &Value) -> Value {
    match value {
        Value::Array(values) => Value::Array(values.iter().map(gameplay).collect()),
        Value::Object(object) => Value::Object(
            object
                .iter()
                .filter(|(key, _)| allowed_key(key))
                .map(|(key, value)| (key.clone(), gameplay(value)))
                .collect::<Map<_, _>>(),
        ),
        Value::String(text) => {
            if allowed_string(text) {
                value.clone()
            } else {
                Value::Null
            }
        }
        _ => value.clone(),
    }
}
fn allowed_key(key: &str) -> bool {
    !key.starts_with("raw_")
        && key.len() <= 64
        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        && ![
            "user_id",
            "player_name",
            "userId",
            "playerName",
            "accountId",
            "sessionId",
            "transactionId",
            "request_id",
            "matchID",
            "matchId",
            "match_id",
            "raw_client_action",
            "raw_match_state",
            "raw_start_hook",
            "name",
            "description",
            "exception",
            "token",
            "accessToken",
            "refreshToken",
        ]
        .contains(&key)
}
fn allowed_string(text: &str) -> bool {
    [
        "keep",
        "mulligan",
        "game_result",
        "game_state_message",
        "queued_game_state_message",
        "mulligan_resp",
        "select_n_resp",
        "submit_deck_resp",
        "client_action",
        "match_started",
        "match_completed",
        "won",
        "lost",
        "draw",
    ]
    .contains(&text)
        || [
            "GameStage_",
            "GameStateType_",
            "GameType_",
            "GameVariant_",
            "MatchState_",
            "MatchScope_",
            "ResultType_",
            "ResultReason_",
            "ZoneType_",
            "Visibility_",
            "ClientMessageType_",
            "AnnotationType_",
            "Phase_",
            "Step_",
            "MulliganType_",
            "MulliganOption_",
            "ActionType_",
            "ObjectType_",
            "GameObjectType_",
            "CardType_",
            "SuperType_",
            "SubType_",
            "Color_",
            "ManaColor_",
            "AbilityType_",
            "CounterType_",
        ]
        .iter()
        .any(|prefix| {
            text.starts_with(prefix)
                && text.len() < 128
                && text.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
}
