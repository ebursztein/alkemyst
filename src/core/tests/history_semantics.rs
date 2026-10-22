use alkemyst_core::{
    db::Engine,
    history::{GameQuery, History, MatchMode, MissingData, StatisticsQuery, TimelineQuery},
    ingest::Ingestor,
};
use serde_json::{Value, json};
use std::io::Write as _;
mod history_support;
use history_support::{deck, line, start, state};
#[tokio::test]
async fn samples_filters_and_completeness_do_not_infer_missing_history() {
    let db = Engine::memory(4 * 1024 * 1024).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    let text = deck()
        + &start("complete", "Traditional_Ladder")
        + &state("complete", 1, 1, 2, None)
        + &state("complete", 2, 2, 1, None)
        + &state("complete", 3, 3, 2, None)
        + &state("complete", 4, 3, 2, Some(20))
        + &start("partial", "Ladder")
        + &state("partial", 5, 5, 2, None)
        + &state("partial", 6, 5, 2, Some(0));
    std::fs::write(&path, text).unwrap();
    let ingest = Ingestor::new(db.clone());
    ingest.import_file(&path).await.unwrap();
    let h = History::new(db.clone());
    let page = h.games(GameQuery::default()).await.unwrap();
    let complete = &page.items[1];
    let partial = &page.items[0];
    assert!(complete.capture_complete, "{:?}", complete.missing_data);
    assert_eq!(complete.mode, MatchMode::Bo3);
    assert_eq!(complete.on_play, Some(true));
    assert_eq!(complete.mulligans, Some(1));
    assert_eq!(complete.land_drops_observed, Some(2));
    assert_eq!(complete.land_turn_samples, 2);
    assert!(!partial.capture_complete);
    assert!(partial.missing_data.contains(&MissingData::InitialState));
    assert!(partial.outcome.is_none());
    assert_eq!(partial.mode, MatchMode::Bo1);
    let stats = h.statistics(StatisticsQuery::default()).await.unwrap();
    assert_eq!(
        (
            stats.games,
            stats.known_results,
            stats.wins,
            stats.unknown_results,
            stats.partial_games
        ),
        (2, 1, 1, 1, 1)
    );
    assert_eq!(stats.decisive_win_rate, Some(1.0));
    assert_eq!(stats.decisive_games, 1);
    assert_eq!(
        h.statistics(StatisticsQuery {
            complete_only: true,
            ..Default::default()
        })
        .await
        .unwrap()
        .games,
        1
    );
    assert_eq!(
        h.games(GameQuery {
            mode: Some(MatchMode::Bo3),
            ..Default::default()
        })
        .await
        .unwrap()
        .items
        .len(),
        1
    );
    let filtered = h
        .games(GameQuery {
            deck_id: complete.deck_id.clone(),
            version_id: complete.deck_version.clone(),
            match_id: Some(complete.match_id.clone()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(filtered.items.len(), 1);
    let first = h
        .games(GameQuery {
            limit: 1,
            ..Default::default()
        })
        .await
        .unwrap();
    let next = h
        .games(GameQuery {
            limit: 1,
            cursor: first.next_cursor,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(next.items[0].id, complete.id);
    assert!(next.next_cursor.is_none());
    let original = h
        .timeline(
            &complete.id,
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
    file.write_all(
        (start("complete", "Traditional_Ladder") + &state("complete", 4, 3, 2, Some(10)))
            .as_bytes(),
    )
    .unwrap();
    ingest.poll_file(&path).await.unwrap();
    let changed = h.game(&complete.id).await.unwrap().unwrap();
    assert!(changed.outcome.is_none());
    assert!(
        changed
            .missing_data
            .contains(&MissingData::ConflictingResult)
    );
    let mut cursor = original.next_cursor;
    while let Some(next) = cursor {
        let page = h
            .timeline(
                &complete.id,
                TimelineQuery {
                    limit: 1,
                    cursor: Some(next),
                    turn_number: None,
                },
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(page.through_ordinal, original.through_ordinal);
        assert!(
            page.items
                .iter()
                .all(|event| event.ordinal <= original.through_ordinal)
        );
        cursor = page.next_cursor;
    }
    assert!(
        h.timeline(&complete.id, TimelineQuery::default())
            .await
            .unwrap()
            .unwrap()
            .through_ordinal
            > original.through_ordinal
    );
    assert_eq!(
        h.statistics(StatisticsQuery::default())
            .await
            .unwrap()
            .known_results,
        0
    );
}

#[tokio::test]
async fn match_outcomes_require_match_scoped_evidence_and_conflicts_stay_unknown() {
    let db = Engine::memory(4096).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    let end = |scope: &str, winner: u32| {
        line(
            json!({"matchGameRoomStateChangedEvent":{"gameRoomInfo":{"stateType":"MatchGameRoomStateType_MatchCompleted","gameRoomConfig":{"matchId":"scope"},"finalMatchResult":{"resultList":[{"scope":scope,"result":"ResultType_WinLoss","winningTeamId":winner}]}}}}),
        )
    };
    let mut opened: Value =
        serde_json::from_str(start("scope", "Ladder").split_once(' ').unwrap().1).unwrap();
    opened["timestamp"] = json!(1_700_000_000_000i64);
    let text = deck()
        + &line(opened)
        + &state("scope", 1, 1, 2, None)
        + &state("scope", 2, 1, 2, Some(20))
        + &end("MatchScope_Game", 20);
    std::fs::write(&path, text).unwrap();
    let ingest = Ingestor::new(db.clone());
    ingest.import_file(&path).await.unwrap();
    let h = History::new(db);
    let page = h.matches(GameQuery::default()).await.unwrap();
    let m = &page.items[0];
    assert!(m.completed);
    assert!(m.outcome.is_none());
    assert!(m.games[0].outcome.is_some());
    assert_eq!(
        m.games[0].match_instant_utc.as_deref(),
        Some("2023-11-14T22:13:20+00:00")
    );
    assert!(m.games[0].local_time.is_none());
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    file.write_all(end("MatchScope_Match", 20).as_bytes())
        .unwrap();
    ingest.poll_file(&path).await.unwrap();
    assert_eq!(
        h.match_summary(&m.id).await.unwrap().unwrap().outcome,
        Some(alkemyst_core::domain::ObservedOutcome::Won)
    );
    file.write_all(end("MatchScope_Match", 10).as_bytes())
        .unwrap();
    ingest.poll_file(&path).await.unwrap();
    assert!(
        h.match_summary(&m.id)
            .await
            .unwrap()
            .unwrap()
            .outcome
            .is_none()
    );
}
