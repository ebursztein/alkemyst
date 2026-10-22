use alkemyst_core::{
    db::Engine,
    domain::{CreateDeck, DeckCard, Decks, Origin, Zone},
    ingest::Ingestor,
};

fn deck(name: &str) -> CreateDeck {
    CreateDeck {
        name: name.into(),
        format: "test".into(),
        cards: vec![DeckCard {
            reference: "oracle:equipment".into(),
            quantity: 4,
            zone: Zone::Main,
        }],
    }
}

#[tokio::test]
async fn restore_replaces_data_and_cached_reads_and_preserves_capture_checkpoints() {
    let dir = tempfile::tempdir().unwrap();
    let backup = dir.path().join("backup.sqlite3");
    let source = Engine::memory(4096).unwrap();
    let original = Decks::new(source.clone())
        .create("saved", deck("Saved deck"), Origin::Agent)
        .await
        .unwrap();
    let log = dir.path().join("Player.log");
    std::fs::write(&log, include_str!("fixtures/arena-game.txt")).unwrap();
    Ingestor::new(source.clone())
        .import_file(&log)
        .await
        .unwrap();
    let capture =
        serde_json::to_value(Ingestor::new(source.clone()).status().await.unwrap()).unwrap();
    source.backup_to(&backup).await.unwrap();
    let before = std::fs::read(&backup).unwrap();
    let target = Engine::memory(4096).unwrap();
    let decks = Decks::new(target.clone());
    let obsolete = decks
        .create("old", deck("Discarded deck"), Origin::User)
        .await
        .unwrap();
    for _ in 0..2 {
        assert_eq!(decks.list(100, None).await.unwrap().items.len(), 1);
    }
    assert!(target.metrics().await.unwrap().cache_hits > 0);
    target.restore_from(&backup).await.unwrap();
    assert_eq!(decks.version(&original.id).await.unwrap(), Some(original));
    assert!(decks.version(&obsolete.id).await.unwrap().is_none());
    assert!(
        decks
            .list(100, None)
            .await
            .unwrap()
            .items
            .iter()
            .all(|d| d.name != "Discarded deck")
    );
    assert_eq!(
        serde_json::to_value(Ingestor::new(target.clone()).status().await.unwrap()).unwrap(),
        capture
    );
    assert_eq!(
        Ingestor::new(target.clone())
            .import_file(&log)
            .await
            .unwrap()
            .events_seen,
        0
    );
    assert_eq!(std::fs::read(&backup).unwrap(), before);
    target.close().await.unwrap();
    source.close().await.unwrap();
}

#[tokio::test]
async fn validated_staging_keeps_the_source_unchanged_and_never_clobbers_destinations() {
    let dir = tempfile::tempdir().unwrap();
    let backup = dir.path().join("source.sqlite3");
    let staged = dir.path().join("staged.sqlite3");
    let db = Engine::memory(4096).unwrap();
    let version = Decks::new(db.clone())
        .create("saved", deck("Saved"), Origin::User)
        .await
        .unwrap();
    db.backup_to(&backup).await.unwrap();
    let bytes = std::fs::read(&backup).unwrap();
    db.stage_backup(&backup, &staged).await.unwrap();
    assert_eq!(std::fs::read(&backup).unwrap(), bytes);
    let ready = Engine::open(&staged, 4096).unwrap();
    assert_eq!(
        Decks::new(ready.clone())
            .version(&version.id)
            .await
            .unwrap(),
        Some(version)
    );
    ready.close().await.unwrap();
    let unchanged = std::fs::read(&staged).unwrap();
    assert!(db.stage_backup(&backup, &staged).await.is_err());
    assert_eq!(std::fs::read(&staged).unwrap(), unchanged);
    db.close().await.unwrap();
}

#[tokio::test]
async fn invalid_or_unsupported_backups_never_replace_current_data_or_create_missing_sources() {
    let dir = tempfile::tempdir().unwrap();
    let good = dir.path().join("good.sqlite3");
    let invalid = dir.path().join("invalid.sqlite3");
    let future = dir.path().join("future.sqlite3");
    let missing = dir.path().join("missing.sqlite3");
    let staged = dir.path().join("staged.sqlite3");
    let db = Engine::memory(4096).unwrap();
    let version = Decks::new(db.clone())
        .create("kept", deck("Kept"), Origin::User)
        .await
        .unwrap();
    db.backup_to(&good).await.unwrap();
    let mut bytes = std::fs::read(&good).unwrap();
    bytes[60..64].copy_from_slice(&(i32::MAX as u32).to_be_bytes());
    std::fs::write(&future, bytes).unwrap();
    std::fs::write(&invalid, b"not a SQLite backup").unwrap();
    for input in [&invalid, &future, &missing] {
        assert!(db.restore_from(input).await.is_err());
        assert!(db.stage_backup(input, &staged).await.is_err());
        assert!(!staged.exists());
        assert_eq!(
            Decks::new(db.clone())
                .version(&version.id)
                .await
                .unwrap()
                .as_ref(),
            Some(&version)
        );
    }
    assert!(!missing.exists());
    db.close().await.unwrap();
}
