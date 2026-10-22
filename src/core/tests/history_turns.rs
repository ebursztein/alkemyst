mod history_support;
use alkemyst_core::{
    db::Engine,
    history::{History, TimelineQuery, TurnQuery},
    ingest::Ingestor,
};
use history_support::{deck, line, start, state};
use serde_json::json;
use std::io::Write as _;
#[tokio::test]
async fn turn_pages_use_explicit_state_references_and_stable_capture_boundaries() {
    let db = Engine::memory(4096).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    let action = line(
        json!({"clientToMatchServiceMessageType":"ClientToMatchServiceMessageType_ClientToGREMessage","payload":{"type":"ClientMessageType_PerformActionResp","gameStateId":1,"respId":10}}),
    );
    let text = deck()
        + &start("turns", "Ladder")
        + &state("turns", 1, 1, 2, None)
        + &action
        + &state("turns", 2, 2, 1, None)
        + &state("turns", 3, 3, 2, None)
        + &state("turns", 4, 3, 2, Some(20));
    std::fs::write(&path, text).unwrap();
    let ingest = Ingestor::new(db.clone());
    ingest.import_file(&path).await.unwrap();
    let h = History::new(db);
    let game = h.games(Default::default()).await.unwrap().items.remove(0);
    let first = h
        .turns(
            &game.id,
            TurnQuery {
                limit: 2,
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(first.items.len(), 2);
    assert_eq!(first.items[0].number, 1);
    assert_eq!(first.items[0].actions, 1);
    assert_eq!(first.items[0].land_drops_observed, Some(1));
    assert_eq!(first.items[0].local_life_total, Some(20));
    assert_eq!(first.items[1].is_local_turn, Some(false));
    assert_eq!(first.items[1].land_drops_observed, None);
    assert!(first.unassigned_events > 0);
    let events = h
        .timeline(
            &game.id,
            TimelineQuery {
                turn_number: Some(1),
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(events.items.len(), 2);
    assert!(
        events
            .items
            .iter()
            .all(|event| event.turn_number == Some(1))
    );
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    file.write_all(state("turns", 5, 3, 2, Some(20)).as_bytes())
        .unwrap();
    ingest.poll_file(&path).await.unwrap();
    let next = h
        .turns(
            &game.id,
            TurnQuery {
                limit: 2,
                cursor: first.next_cursor,
            },
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(next.through_ordinal, first.through_ordinal);
    assert_eq!(next.items.len(), 1);
    assert_eq!(next.items[0].number, 3);
    assert_eq!(next.items[0].events, 2);
    assert!(next.next_cursor.is_none());
    assert!(
        h.turns("absent", TurnQuery::default())
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn later_backfill_cannot_reassign_events_inside_an_older_snapshot() {
    let db = Engine::memory(4096).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    let action = line(
        json!({"clientToMatchServiceMessageType":"ClientToMatchServiceMessageType_ClientToGREMessage","payload":{"type":"ClientMessageType_PerformActionResp","gameStateId":50,"respId":9}}),
    );
    std::fs::write(
        &path,
        deck() + &start("late", "Ladder") + &state("late", 1, 1, 2, None) + &action,
    )
    .unwrap();
    let ingest = Ingestor::new(db.clone());
    ingest.import_file(&path).await.unwrap();
    let h = History::new(db);
    let game = h.games(Default::default()).await.unwrap().items.remove(0);
    let first = h
        .timeline(
            &game.id,
            TimelineQuery {
                limit: 1,
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .unwrap();
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    file.write_all(state("late", 50, 8, 2, None).as_bytes())
        .unwrap();
    ingest.poll_file(&path).await.unwrap();
    let old = h
        .timeline(
            &game.id,
            TimelineQuery {
                cursor: first.next_cursor,
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        old.items
            .iter()
            .find(|e| e.kind == "action")
            .unwrap()
            .turn_number,
        None
    );
    let fresh = h
        .timeline(&game.id, TimelineQuery::default())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        fresh
            .items
            .iter()
            .find(|e| e.kind == "action")
            .unwrap()
            .turn_number,
        Some(8)
    );
}
#[tokio::test]
async fn a_partial_capture_with_high_turn_numbers_has_valid_turn_cursors() {
    let db = Engine::memory(4096).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    std::fs::write(
        &path,
        start("midgame", "Ladder")
            + &state("midgame", 1000, 100, 2, None)
            + &state("midgame", 1001, 101, 1, Some(20)),
    )
    .unwrap();
    let ingest = Ingestor::new(db.clone());
    ingest.import_file(&path).await.unwrap();
    let h = History::new(db);
    let game = h.games(Default::default()).await.unwrap().items.remove(0);
    assert!(!game.capture_complete);
    let first = h
        .turns(
            &game.id,
            TurnQuery {
                limit: 1,
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(first.items[0].number, 100);
    let next = h
        .turns(
            &game.id,
            TurnQuery {
                limit: 1,
                cursor: first.next_cursor,
            },
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(next.items[0].number, 101);
}
