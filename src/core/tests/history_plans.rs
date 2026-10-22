use alkemyst_core::{
    db::{Engine, Mutation, Statement, Write},
    history::{GameQuery, History, StatisticsQuery, TimelineQuery, TurnQuery},
};
use serde_json::json;

#[tokio::test]
async fn larger_history_uses_indexed_plans_and_one_statement_per_public_read() {
    let db = Engine::memory(4 * 1024 * 1024).unwrap();
    let count = 1000usize;
    let matches=(0..count).map(|i|json!({"match_id":format!("match-{i}"),"game_id":format!("game-{i}"),"game_number":1,"deck_version":null})).collect::<Vec<_>>();
    let events=(0..count).flat_map(|i|(1..=12).map(move|state|json!({"id":format!("event-{i}-{state}"),"match_id":format!("match-{i}"),"game_id":format!("game-{i}"),"kind":if state==12{"result"}else{"state"},"state_id":state,"payload":{"type":"game_state_message","game_state_type":if state==1{"GameStateType_Full"}else{"GameStateType_Diff"},"prev_game_state_id":if state>1{Some(state-1)}else{None},"outcome":if state==12{[Some("won"),Some("lost"),Some("draw"),None][i%4]}else{None},"local_seat":1,"turn_info":{"turn_number":state,"active_player":if state%2==1{1}else{2}},"players":[{"systemSeatNumber":1,"mulliganCount":0,"landsPlayedThisTurn":1}]}}))).collect::<Vec<_>>();
    db.write(vec![
  Write{statement:Mutation{sql:"INSERT INTO matches SELECT json_extract(value,'$.match_id') FROM json_each(?1)",writes:&["matches"]},params:vec![json!(serde_json::to_string(&matches).unwrap())],expected_rows:Some(count)},
  Write{statement:Mutation{sql:include_str!("../src/statements/capture/games.sql"),writes:&["games"]},params:vec![json!(serde_json::to_string(&matches).unwrap())],expected_rows:Some(count)},
  Write{statement:Mutation{sql:include_str!("../src/statements/capture/events.sql"),writes:&["capture_events","game_capture_order","match_capture_order"]},params:vec![json!(serde_json::to_string(&events).unwrap())],expected_rows:Some(events.len())},
 ]).await.unwrap();
    let games = Statement {
        name: "test.plan.games",
        sql: include_str!("../src/statements/history/games.sql"),
        reads: &[],
        cacheable: false,
    };
    let plan = db
        .explain(
            games,
            vec![
                json!(null),
                json!(null),
                json!(null),
                json!(null),
                json!(null),
                json!(10),
            ],
        )
        .await
        .unwrap()
        .join("\n");
    assert!(plan.contains("capture_events_kind"), "{plan}");
    assert!(
        plan.lines()
            .any(|line| line.contains("SEARCH p USING COVERING INDEX")
                && line.contains("game_id=?")
                && line.contains("state_id=?")),
        "{plan}"
    );
    assert!(plan.contains("game_capture_order"), "{plan}");
    let timeline = Statement {
        sql: include_str!("../src/statements/history/timeline.sql"),
        ..games
    };
    let plan = db
        .explain(
            timeline,
            vec![
                json!("game-0"),
                json!(null),
                json!(null),
                json!(10),
                json!(null),
            ],
        )
        .await
        .unwrap()
        .join("\n");
    assert!(plan.contains("capture_events_game"), "{plan}");
    let status = Statement {
        sql: include_str!("../src/statements/capture/status.sql"),
        ..games
    };
    let plan = db.explain(status, vec![]).await.unwrap().join("\n");
    assert!(plan.contains("capture_events_kind"), "{plan}");
    let h = History::new(db.clone());
    let before = db.metrics().await.unwrap().statements;
    assert_eq!(
        h.games(GameQuery {
            limit: 10,
            ..Default::default()
        })
        .await
        .unwrap()
        .items
        .len(),
        10
    );
    assert_eq!(
        h.matches(GameQuery {
            limit: 2,
            ..Default::default()
        })
        .await
        .unwrap()
        .items
        .len(),
        2
    );
    assert_eq!(
        h.timeline(
            "game-0",
            TimelineQuery {
                limit: 2,
                ..Default::default()
            }
        )
        .await
        .unwrap()
        .unwrap()
        .items
        .len(),
        2
    );
    let stats = h.statistics(StatisticsQuery::default()).await.unwrap();
    assert_eq!(stats.games, count as u64);
    assert_eq!(stats.known_results, 750);
    assert_eq!((stats.wins, stats.losses, stats.draws), (250, 250, 250));
    assert_eq!(stats.unknown_results, 250);
    assert_eq!(stats.decisive_win_rate, Some(0.5));
    assert_eq!(stats.land_turn_samples, 6000);
    assert_eq!(stats.recorded_land_drops, 6000);
    assert_eq!(
        h.turns(
            "game-0",
            TurnQuery {
                limit: 2,
                ..Default::default()
            }
        )
        .await
        .unwrap()
        .unwrap()
        .items
        .len(),
        2
    );
    assert_eq!(db.metrics().await.unwrap().statements, before + 5);
}
