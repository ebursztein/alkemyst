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
    use serde_json::json;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("legacy.sqlite3");
    let input = CreateDeck {
        name: "Boros".into(),
        format: "standard".into(),
        cards: vec![],
    };
    {
        // Build a real v4 fixture so newer migrations cannot leak into its schema.
        let legacy = rusqlite::Connection::open(&path).unwrap();
        for sql in [
            include_str!("../migrations/001_initial.sql"),
            include_str!("../migrations/002_catalog.sql"),
            include_str!("../migrations/003_arena_aliases.sql"),
            include_str!("../migrations/004_decks.sql"),
        ] {
            legacy.execute_batch(sql).unwrap();
        }
        legacy
            .execute(
                "INSERT INTO decks VALUES ('legacy-deck','Boros','standard',123)",
                [],
            )
            .unwrap();
        legacy.execute("INSERT INTO deck_versions(id,deck_id,ordinal,name,format,origin,created_at) VALUES ('legacy-version','legacy-deck',1,'Boros','standard','user',123)",[]).unwrap();
        legacy
            .execute(
                "UPDATE deck_versions SET sealed=1 WHERE id='legacy-version'",
                [],
            )
            .unwrap();
        legacy.execute("INSERT INTO mutation_receipts VALUES ('legacy-key','decks.create',?1,'legacy-version')",[json!({"input":input,"origin":"user"}).to_string()]).unwrap();
    }
    let reopened = Engine::open(&path, 4096).unwrap();
    let retry = Decks::new(reopened.clone())
        .create("legacy-key", input, Origin::Agent)
        .await
        .unwrap();
    assert_eq!(retry.id, "legacy-version");
    assert_eq!(retry.deck_id, "legacy-deck");
    assert_eq!(retry.origin, Origin::User);
    assert_eq!(retry.created_at, 123);
    reopened.close().await.unwrap();
}
