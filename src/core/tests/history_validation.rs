use alkemyst_core::{
    db::Engine,
    history::{GameQuery, History, StatisticsQuery, TimelineQuery, TurnQuery},
};
#[tokio::test]
async fn empty_samples_remain_unknown_and_malformed_filters_never_reach_sql() {
    let db = Engine::memory(4096).unwrap();
    let h = History::new(db.clone());
    let stats = h.statistics(StatisticsQuery::default()).await.unwrap();
    assert_eq!(stats.games, 0);
    assert!(stats.decisive_win_rate.is_none());
    assert!(stats.observed_mulligans_average.is_none());
    assert_eq!(stats.mulligan_samples, 0);
    assert!(
        h.games(GameQuery::default())
            .await
            .unwrap()
            .items
            .is_empty()
    );
    assert!(
        h.matches(GameQuery::default())
            .await
            .unwrap()
            .items
            .is_empty()
    );
    let before = db.metrics().await.unwrap().read_calls;
    assert!(
        h.turns(
            "game",
            TurnQuery {
                limit: 0,
                ..Default::default()
            }
        )
        .await
        .is_err()
    );
    assert!(
        h.turns(
            "game",
            TurnQuery {
                cursor: Some("invalid".into()),
                ..Default::default()
            }
        )
        .await
        .is_err()
    );
    assert!(
        h.timeline(
            "game",
            TimelineQuery {
                turn_number: Some(0),
                ..Default::default()
            }
        )
        .await
        .is_err()
    );
    for limit in [0, 101] {
        assert!(
            h.games(GameQuery {
                limit,
                ..Default::default()
            })
            .await
            .is_err()
        );
        assert!(
            h.matches(GameQuery {
                limit,
                ..Default::default()
            })
            .await
            .is_err()
        );
        assert!(
            h.timeline(
                "game",
                TimelineQuery {
                    limit,
                    cursor: None,
                    turn_number: None,
                }
            )
            .await
            .is_err()
        );
    }
    for cursor in [
        "",
        "wrong:123",
        "h1:0000000000000000",
        "h1:zzzzzzzzzzzzzzzz",
        "h1:8000000000000000",
        "h1:0000000000000001:0000000000000002",
        "h1:0000000000000001:0000000000000001:0000000000000001",
    ] {
        assert!(
            h.games(GameQuery {
                cursor: Some(cursor.into()),
                ..Default::default()
            })
            .await
            .is_err()
        );
        assert!(
            h.timeline(
                "game",
                TimelineQuery {
                    cursor: Some(cursor.into()),
                    ..Default::default()
                }
            )
            .await
            .is_err()
        );
    }
    for id in ["".to_owned(), "a".repeat(257)] {
        assert!(h.game(&id).await.is_err());
        assert!(h.match_summary(&id).await.is_err());
        assert!(h.timeline(&id, TimelineQuery::default()).await.is_err());
        assert!(
            h.statistics(StatisticsQuery {
                deck_id: Some(id),
                ..Default::default()
            })
            .await
            .is_err()
        );
    }
    assert_eq!(db.metrics().await.unwrap().read_calls, before);
}
