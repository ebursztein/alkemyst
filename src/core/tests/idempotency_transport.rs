use alkemyst_core::{
    db::Engine,
    domain::{CreateDeck, Decks, Origin},
};

#[tokio::test]
async fn transport_origin_does_not_change_logical_request_identity() {
    let decks = Decks::new(Engine::memory(4096).unwrap());
    let input = CreateDeck {
        name: "Boros".into(),
        format: "standard".into(),
        cards: vec![],
    };
    let user = decks
        .create("shared-key", input.clone(), Origin::User)
        .await
        .unwrap();
    let agent = decks
        .create("shared-key", input, Origin::Agent)
        .await
        .unwrap();
    assert_eq!(user, agent);
    assert_eq!(agent.origin, Origin::User);
}

#[tokio::test]
async fn existing_receipts_migrate_without_losing_retry_identity() {
    use alkemyst_core::db::{Mutation, Write};
    use serde_json::json;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy.sqlite3");
    let db = Engine::open(&path, 4096).unwrap();
    let decks = Decks::new(db.clone());
    let input = CreateDeck {
        name: "Boros".into(),
        format: "standard".into(),
        cards: vec![],
    };
    let original = decks
        .create("legacy-key", input.clone(), Origin::User)
        .await
        .unwrap();
    db.write(vec![
        Write {
            statement: Mutation {
                sql: "UPDATE mutation_receipts SET body=json_set(body, '$.origin', 'user') WHERE key=?1",
                writes: &["mutation_receipts"],
            },
            params: vec![json!("legacy-key")],
            expected_rows: Some(1),
        },
        Write {
            statement: Mutation { sql: "PRAGMA user_version=4", writes: &[] },
            params: vec![], expected_rows: None,
        },
    ]).await.unwrap();
    db.close().await.unwrap();
    let reopened = Engine::open(&path, 4096).unwrap();
    let retry = Decks::new(reopened.clone())
        .create("legacy-key", input, Origin::Agent)
        .await
        .unwrap();
    assert_eq!(original, retry);
    reopened.close().await.unwrap();
}
