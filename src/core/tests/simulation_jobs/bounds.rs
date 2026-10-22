use super::*;
use alkemyst_core::db::{Mutation, Write};
use serde_json::json;
#[tokio::test]
async fn queue_capacity_rolls_back_receipts_and_list_cursors_remain_bounded() {
    let (_, analysis, version) = setup().await;
    let p = parameters(&version);
    let mut first = None;
    for i in 0..16 {
        let job = analysis
            .create_simulation(&format!("queue-{i}"), p.clone(), Origin::User)
            .await
            .unwrap();
        first.get_or_insert(job.id);
    }
    assert!(matches!(
        analysis
            .create_simulation("queue-full", p.clone(), Origin::User)
            .await,
        Err(alkemyst_core::analysis::Error::QueueFull)
    ));
    analysis
        .cancel_simulation(first.as_ref().unwrap(), Origin::User)
        .await
        .unwrap();
    analysis
        .create_simulation("queue-full", p, Origin::User)
        .await
        .unwrap();
    let mut cursor = None;
    let mut ids = std::collections::BTreeSet::new();
    loop {
        let page = analysis
            .simulations(JobQuery {
                limit: 3,
                cursor,
                version_id: Some(version.clone()),
            })
            .await
            .unwrap();
        for job in page.items {
            assert!(ids.insert(job.id));
        }
        cursor = page.next_cursor;
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!(ids.len(), 17);
    for cursor in ["bad", "s1:0000000000000000", "s1:+000000000000001"] {
        assert!(
            analysis
                .simulations(JobQuery {
                    cursor: Some(cursor.into()),
                    ..Default::default()
                })
                .await
                .is_err()
        );
    }
}
#[tokio::test]
async fn invalid_parameters_and_input_mutations_fail_without_corrupting_the_queue() {
    let (db, analysis, version) = setup().await;
    let mut p = parameters(&version);
    p.seed = None;
    let one = analysis
        .create_simulation("generated-seed", p.clone(), Origin::User)
        .await
        .unwrap();
    assert_eq!(one.seed.len(), 16);
    assert_eq!(
        analysis
            .create_simulation("generated-seed", p.clone(), Origin::User)
            .await
            .unwrap()
            .seed,
        one.seed
    );
    for seed in ["1", "zzzzzzzzzzzzzzzz"] {
        let mut bad = p.clone();
        bad.seed = Some(seed.into());
        assert!(
            analysis
                .create_simulation("bad", bad, Origin::User)
                .await
                .is_err()
        );
    }
    let mut bad = p.clone();
    bad.mulligan.required_references = vec!["oracle:not-in-main".into()];
    assert!(
        analysis
            .create_simulation("bad-required", bad, Origin::User)
            .await
            .is_err()
    );
    assert!(
        db.write(vec![Write {
            statement: Mutation {
                sql: "UPDATE simulation_runs SET seed='0000000000000000' WHERE id=?1",
                writes: &["simulation_runs"]
            },
            params: vec![json!(one.id)],
            expected_rows: Some(1)
        }])
        .await
        .is_err()
    );
    assert_eq!(
        analysis.simulation(&one.id).await.unwrap().unwrap().seed,
        one.seed
    );
}
