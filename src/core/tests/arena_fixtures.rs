use manasight_parser::{GameEvent, parse_whole_log};
use serde_json::Value;
use std::collections::BTreeSet;

fn fixture() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/arena-game.txt"
    ))
    .expect("sanitized real Arena capture must exist")
}

#[test]
fn real_capture_contains_deck_changes_game_result_and_turn_actions() {
    let events = parse_whole_log(&fixture());
    let payloads: Vec<_> = events.iter().map(GameEvent::payload).collect();
    let hashes: BTreeSet<_> = payloads
        .iter()
        .filter(|p| p["type"] == "deck_submission")
        .map(|p| p["maindeck_hash"].as_str().unwrap())
        .collect();
    assert_eq!(hashes.len(), 3, "three observed deck configurations");
    assert_eq!(
        payloads
            .iter()
            .filter(|p| p["type"] == "match_started")
            .count(),
        1
    );
    assert_eq!(
        payloads
            .iter()
            .filter(|p| p["type"] == "match_completed")
            .count(),
        1
    );
    assert_eq!(
        payloads
            .iter()
            .filter(|p| p["type"] == "game_result")
            .count(),
        1
    );
    assert!(
        payloads
            .iter()
            .any(|p| p["turn_info"]["turn_number"].as_u64().unwrap_or(0) > 1)
    );
    assert!(
        events
            .iter()
            .any(|e| matches!(e, GameEvent::ClientAction(_)))
    );
    for event in &events {
        if matches!(event, GameEvent::DeckSubmission(_)) {
            let raw = std::str::from_utf8(event.metadata().raw_bytes()).unwrap();
            let start = raw.find('{').unwrap();
            let outer: Value = serde_json::from_str(&raw[start..]).unwrap();
            let request: Value = serde_json::from_str(outer["request"].as_str().unwrap()).unwrap();
            let cards = request["Deck"]["MainDeck"].as_array().unwrap();
            assert_eq!(
                cards
                    .iter()
                    .map(|c| c["quantity"].as_u64().unwrap())
                    .sum::<u64>(),
                60
            );
        }
    }
}

#[test]
fn public_fixture_has_no_free_text_or_real_identifiers() {
    let input = fixture();
    assert!(!input.contains("/Users/"));
    assert!(!input.to_ascii_lowercase().contains("access_token"));
    assert!(!input.to_ascii_lowercase().contains("authorization"));
    assert!(!input.contains('@'));
    for line in input.lines() {
        if let Some(start) = line.find('{') {
            validate_strings(&serde_json::from_str::<Value>(&line[start..]).unwrap());
        }
    }
}

fn validate_strings(value: &Value) {
    match value {
        Value::Object(object) => {
            for (key, value) in object {
                assert!(!key.contains('-'), "unexpected dynamic key");
                if key == "request" {
                    validate_strings(
                        &serde_json::from_str::<Value>(value.as_str().unwrap()).unwrap(),
                    );
                } else {
                    validate_strings(value);
                }
            }
        }
        Value::Array(array) => array.iter().for_each(validate_strings),
        Value::String(text) => {
            let permitted = text.is_empty()
                || text.starts_with("fixture-")
                || ["Ladder", "Standard", "Format"].contains(&text.as_str())
                || text.split_once('_').is_some_and(|(prefix, suffix)| {
                    [
                        "ClientToMatchServiceMessageType",
                        "GREMessageType",
                        "GameStateType",
                        "GameStage",
                        "MatchState",
                        "GameType",
                        "GameVariant",
                        "MatchScope",
                        "ResultType",
                        "ResultReason",
                        "MatchGameRoomStateType",
                        "MatchCompletedReason",
                        "ClientMessageType",
                        "MulliganOption",
                        "Phase",
                        "Step",
                        "ZoneType",
                        "Visibility",
                        "CardType",
                        "SuperType",
                        "SubType",
                        "ObjectType",
                        "AnnotationType",
                        "AbilityType",
                        "ActionType",
                        "ManaColor",
                        "TimerType",
                        "CounterType",
                        "AbilityCategory",
                        "GameObjectType",
                        "Color",
                        "StopType",
                        "SettingKey",
                        "SettingScope",
                        "PromptType",
                        "SelectNOption",
                        "SelectNType",
                        "SelectionType",
                        "TargetType",
                        "TargetSelection",
                        "DecisionType",
                        "PayloadType",
                    ]
                    .contains(&prefix)
                        && suffix
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '_')
                });
            assert!(permitted, "unexpected string in sanitized capture: {text}");
        }
        _ => {}
    }
}
