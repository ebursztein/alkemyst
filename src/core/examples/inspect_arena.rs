//! Read-only local validation. Prints counts, never identities or raw events.
use manasight_parser::{GameEvent, parse_whole_log};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for path in std::env::args().skip(1) {
        let input = std::fs::read_to_string(&path)?;
        let events = parse_whole_log(&input);
        let mut counts = BTreeMap::<String, usize>::new();
        let mut matches = BTreeSet::new();
        let mut games = BTreeSet::new();
        let mut decks = BTreeSet::new();
        let mut lists = BTreeSet::new();
        let mut card_counts = BTreeSet::new();
        let mut distinct_cards = BTreeSet::new();
        for event in &events {
            let payload = event.payload();
            let kind = match event {
                GameEvent::LocalSeat(_) => "local_seat",
                _ => payload["type"].as_str().unwrap_or("other"),
            };
            *counts.entry(kind.to_owned()).or_default() += 1;
            if matches!(event, GameEvent::DeckSubmission(_)) {
                if let Some(id) = payload["deck_id"].as_str() {
                    decks.insert(id);
                }
                if let Some(hash) = payload["maindeck_hash"].as_str() {
                    lists.insert(hash);
                }
                // Upstream exposes deck metadata but retains card lists only in raw bytes.
                let raw = std::str::from_utf8(event.metadata().raw_bytes())?;
                if let Some(start) = raw.find('{') {
                    let outer: Value = serde_json::from_str(&raw[start..])?;
                    if let Some(request) = outer["request"].as_str() {
                        let request: Value = serde_json::from_str(request)?;
                        if let Some(cards) = request["Deck"]["MainDeck"].as_array() {
                            card_counts.insert(
                                cards
                                    .iter()
                                    .filter_map(|c| c["quantity"].as_u64())
                                    .sum::<u64>(),
                            );
                            distinct_cards.insert(cards.len());
                        }
                    }
                }
            }
            let info = &payload["game_info"];
            if let Some(id) = info["matchID"].as_str() {
                matches.insert(id);
                if let Some(number) = info["gameNumber"].as_u64() {
                    games.insert((id, number));
                }
            }
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "file": std::path::Path::new(&path).file_name().and_then(|name| name.to_str()),
                "bytes": input.len(), "events": counts, "matches_with_game_info": matches.len(),
                "numbered_games": games.len(), "submitted_decks": decks.len(),
                "main_deck_versions": lists.len(), "main_deck_card_counts": card_counts,
                "main_deck_distinct_card_counts": distinct_cards,
            }))?
        );
    }
    Ok(())
}
