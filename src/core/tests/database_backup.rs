use alkemyst_core::{
    db::Engine,
    domain::{CreateDeck, DeckCard, Decks, NewVersion, Origin, Zone},
    ingest::Ingestor,
};

fn deck() -> CreateDeck {
    CreateDeck {
        name: "Snapshot fixture".into(),
        format: "test".into(),
        cards: vec![DeckCard {
            reference: "oracle:equipment".into(),
            quantity: 4,
            zone: Zone::Main,
        }],
    }
}

#[tokio::test]
async fn standalone_backup_preserves_wal_versions_and_history_without_future_changes() {
    let dir = tempfile::tempdir().unwrap();
    let database = dir.path().join("live.sqlite3");
    let backup = dir.path().join("backup.sqlite3");
    let log = dir.path().join("Player.log");
    std::fs::write(&log, include_str!("fixtures/arena-game.txt")).unwrap();
    let db = Engine::open(&database, 4096).unwrap();
    let decks = Decks::new(db.clone());
    let first = decks.create("first", deck(), Origin::User).await.unwrap();
    let next = decks
        .create_version(
            "second",
            NewVersion {
                deck_id: first.deck_id.clone(),
                parent_version: Some(first.id.clone()),
                name: "Second version".into(),
                format: first.format.clone(),
                cards: first.cards.clone(),
            },
            Origin::Agent,
        )
        .await
        .unwrap();
    let ingest = Ingestor::new(db.clone());
    ingest.import_file(&log).await.unwrap();
    let captured = serde_json::to_value(ingest.status().await.unwrap()).unwrap();
    assert!(
        std::fs::metadata(database.with_extension("sqlite3-wal"))
            .unwrap()
            .len()
            > 0
    );
    db.backup_to(&backup).await.unwrap();
    assert!(!backup.with_extension("sqlite3-wal").exists());
    decks
        .create("after-backup", deck(), Origin::User)
        .await
        .unwrap();
    db.close().await.unwrap();
    let restored = Engine::open(&backup, 4096).unwrap();
    let saved = Decks::new(restored.clone());
    assert_eq!(saved.version(&first.id).await.unwrap(), Some(first));
    assert_eq!(saved.version(&next.id).await.unwrap(), Some(next));
    let count = saved.list(100, None).await.unwrap().items.len();
    let live = Engine::open(&database, 4096).unwrap();
    assert_eq!(
        Decks::new(live.clone())
            .list(100, None)
            .await
            .unwrap()
            .items
            .len(),
        count + 1
    );
    let recovered = Ingestor::new(restored.clone());
    assert_eq!(
        serde_json::to_value(recovered.status().await.unwrap()).unwrap(),
        captured
    );
    assert_eq!(recovered.import_file(&log).await.unwrap().events_seen, 0);
    live.close().await.unwrap();
    restored.close().await.unwrap();
}

#[tokio::test]
async fn backups_never_overwrite_existing_files_or_the_live_database() {
    let dir = tempfile::tempdir().unwrap();
    let live = dir.path().join("live.sqlite3");
    let existing = dir.path().join("existing.sqlite3");
    std::fs::write(&existing, b"keep this file").unwrap();
    let db = Engine::open(&live, 4096).unwrap();
    Decks::new(db.clone())
        .create("deck", deck(), Origin::User)
        .await
        .unwrap();
    assert!(db.backup_to(&existing).await.is_err());
    assert_eq!(std::fs::read(&existing).unwrap(), b"keep this file");
    assert!(db.backup_to(&live).await.is_err());
    assert!(
        db.backup_to(&dir.path().join("missing/backup.sqlite3"))
            .await
            .is_err()
    );
    assert_eq!(
        Decks::new(db.clone())
            .list(10, None)
            .await
            .unwrap()
            .items
            .len(),
        1
    );
    db.close().await.unwrap();
    let absent = dir.path().join("closed.sqlite3");
    assert!(db.backup_to(&absent).await.is_err());
    assert!(!absent.exists());
}
