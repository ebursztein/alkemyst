mod practice_support;
use alkemyst_core::{
    catalog::{Catalog, CatalogSource},
    db::{Engine, Mutation, Write},
    domain::Origin,
    practice::{NewPractice, Practice, PracticeAction, PracticeQuery},
};
use serde_json::{Value, json};

#[tokio::test]
async fn practice_survives_reopen_and_catalog_changes_with_immutable_inputs_and_steps() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("practice.sqlite3");
    let db = Engine::open(&path, 4 * 1024 * 1024).unwrap();
    let (p, version) = practice_support::setup(db.clone()).await;
    let initial = p
        .create(
            "durable",
            NewPractice {
                version_id: version,
                on_the_play: true,
                seed: None,
            },
            Origin::Agent,
        )
        .await
        .unwrap();
    let frozen = p.inputs(&initial.id).await.unwrap().unwrap();
    let kept = p
        .act(
            "durable-keep",
            &initial.id,
            0,
            PracticeAction::Keep { bottom: vec![] },
            Origin::User,
        )
        .await
        .unwrap();
    let rows = include_str!("fixtures/cards.jsonl")
        .lines()
        .map(|line| {
            let mut row: Value = serde_json::from_str(line).unwrap();
            row["name"] = json!("Refreshed public metadata");
            row.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    Catalog::new(db.clone())
        .import_jsonl(
            std::io::Cursor::new(rows),
            CatalogSource {
                url: "https://data.scryfall.io/default-cards/fixture.jsonl.gz".into(),
                updated_at: "2026-10-22T00:00:00Z".into(),
            },
        )
        .await
        .unwrap();
    assert_eq!(p.inputs(&initial.id).await.unwrap().unwrap(), frozen);
    assert_eq!(p.get(&initial.id).await.unwrap().unwrap(), kept);
    for sql in [
        "UPDATE practice_sessions SET seed='0000000000000000' WHERE id=?1",
        "DELETE FROM practice_steps WHERE session_id=?1",
    ] {
        assert!(
            db.write(vec![Write {
                statement: Mutation {
                    sql,
                    writes: &["practice_sessions", "practice_steps"]
                },
                params: vec![json!(initial.id)],
                expected_rows: None
            }])
            .await
            .is_err()
        );
    }
    db.close().await.unwrap();
    drop(p);
    let db = Engine::open(&path, 4 * 1024 * 1024).unwrap();
    let reopened = Practice::new(db);
    assert_eq!(reopened.get(&initial.id).await.unwrap().unwrap(), kept);
    assert_eq!(reopened.inputs(&initial.id).await.unwrap().unwrap(), frozen);
    let next = reopened
        .act(
            "after-reopen",
            &initial.id,
            kept.revision,
            PracticeAction::NextTurn,
            Origin::User,
        )
        .await
        .unwrap();
    assert_eq!(next.turn, 2);
    assert_eq!(next.origin, Origin::Agent);
}

#[tokio::test]
async fn concurrent_actions_use_revisions_and_retries_return_the_original_step() {
    let (p, version) = practice_support::setup(Engine::memory(4 * 1024 * 1024).unwrap()).await;
    let input = NewPractice {
        version_id: version.clone(),
        on_the_play: true,
        seed: Some("FFFFFFFFFFFFFFFF".into()),
    };
    let initial = p.create("race", input.clone(), Origin::User).await.unwrap();
    assert_eq!(initial.seed, "ffffffffffffffff");
    let (a, b) = tokio::join!(
        p.act(
            "race-a",
            &initial.id,
            0,
            PracticeAction::Mulligan,
            Origin::User
        ),
        p.act(
            "race-b",
            &initial.id,
            0,
            PracticeAction::Mulligan,
            Origin::Agent
        )
    );
    assert_ne!(a.is_ok(), b.is_ok());
    let (key, winner) = if let Ok(a) = a {
        ("race-a", a)
    } else {
        ("race-b", b.unwrap())
    };
    assert_eq!(winner.revision, 1);
    let kept = p
        .act(
            "race-keep",
            &initial.id,
            1,
            PracticeAction::Keep {
                bottom: vec![winner.hand[0].id],
            },
            Origin::User,
        )
        .await
        .unwrap();
    assert_eq!(
        p.act(key, &initial.id, 0, PracticeAction::Mulligan, Origin::User)
            .await
            .unwrap(),
        winner
    );
    assert_eq!(p.get(&initial.id).await.unwrap().unwrap(), kept);
    assert!(
        p.act(
            key,
            &initial.id,
            kept.revision,
            PracticeAction::Draw,
            Origin::User
        )
        .await
        .is_err()
    );
    assert!(
        p.create(
            "race",
            NewPractice {
                on_the_play: false,
                ..input.clone()
            },
            Origin::User
        )
        .await
        .is_err()
    );
    assert!(
        p.create("race-a", input.clone(), Origin::User)
            .await
            .is_err()
    );
    for i in 0..4 {
        p.create(&format!("page-{i}"), input.clone(), Origin::User)
            .await
            .unwrap();
    }
    let mut cursor = None;
    let mut ids = std::collections::BTreeSet::new();
    loop {
        let page = p
            .list(PracticeQuery {
                limit: 2,
                cursor,
                version_id: Some(version.clone()),
            })
            .await
            .unwrap();
        for session in page.items {
            assert!(ids.insert(session.id));
        }
        cursor = page.next_cursor;
        if cursor.is_none() {
            break;
        }
    }
    assert_eq!(ids.len(), 5);
}
