mod practice_support;
use alkemyst_core::{
    db::{Engine, Statement},
    domain::Origin,
    practice::{NewPractice, PracticeQuery},
};
use serde_json::json;
#[tokio::test]
async fn practice_pages_and_latest_steps_use_indexed_creation_and_revision_lookups() {
    let db = Engine::memory(4 * 1024 * 1024).unwrap();
    let (p, version) = practice_support::setup(db.clone()).await;
    let mut ids = vec![];
    for i in 0..4 {
        ids.push(
            p.create(
                &format!("plan-{i}"),
                NewPractice {
                    version_id: version.clone(),
                    on_the_play: true,
                    seed: None,
                },
                Origin::User,
            )
            .await
            .unwrap()
            .id,
        );
    }
    let first = p
        .list(PracticeQuery {
            limit: 2,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(
        first.items.iter().map(|s| s.id.clone()).collect::<Vec<_>>(),
        vec![ids[3].clone(), ids[2].clone()]
    );
    let second = p
        .list(PracticeQuery {
            limit: 2,
            cursor: first.next_cursor,
            version_id: Some(version.clone()),
        })
        .await
        .unwrap();
    assert_eq!(
        second
            .items
            .iter()
            .map(|s| s.id.clone())
            .collect::<Vec<_>>(),
        vec![ids[1].clone(), ids[0].clone()]
    );
    assert!(second.next_cursor.is_none());
    let plan = db
        .explain(
            Statement {
                name: "test.practice.plan",
                sql: include_str!("../src/statements/practice/list.sql"),
                reads: &[],
                cacheable: false,
            },
            vec![json!(100), json!(version), json!(2)],
        )
        .await
        .unwrap()
        .join("\n");
    assert!(plan.contains("practice_version_order"), "{plan}");
    assert!(plan.contains("rowid<?"), "{plan}");
    assert!(!plan.contains("SCAN practice_steps"), "{plan}");
}
