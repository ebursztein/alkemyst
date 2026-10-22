use alkemyst_core::{
    catalog::Catalog,
    chat::{Chat, NewConversation},
    db::Engine,
    domain::Decks,
};
use rusqlite::Connection;
use std::path::Path;

fn legacy(path: &Path, version: usize) {
    let connection = Connection::open(path).unwrap();
    let migrations: &[&[&str]] = &[
        &[include_str!("../migrations/001_initial.sql")],
        &[include_str!("../migrations/002_catalog.sql")],
        &[include_str!("../migrations/003_arena_aliases.sql")],
        &[include_str!("../migrations/004_decks.sql")],
        &[include_str!("../migrations/005_receipt_origins.sql")],
        &[include_str!("../migrations/006_capture.sql")],
        &[
            include_str!("../migrations/007_history_order.sql"),
            include_str!("../migrations/007_history_match_facts.sql"),
            include_str!("../migrations/007_history_facts.sql"),
            include_str!("../migrations/007_history_summaries.sql"),
            include_str!("../migrations/007_history_matches.sql"),
        ],
        &[
            include_str!("../migrations/008_simulations.sql"),
            include_str!("../migrations/008_simulation_details.sql"),
        ],
        &[include_str!("../migrations/009_practice.sql")],
    ];
    for scripts in migrations.iter().take(version) {
        for sql in *scripts {
            connection.execute_batch(sql).unwrap();
        }
    }
}

#[tokio::test]
async fn known_older_backups_migrate_only_the_private_copy() {
    let dir = tempfile::tempdir().unwrap();
    let target = Engine::memory(4096).unwrap();
    for version in 1..=9 {
        let original = dir.path().join(format!("schema-{version}.sqlite3"));
        let staged = dir.path().join(format!("staged-{version}.sqlite3"));
        legacy(&original, version);
        let before = std::fs::read(&original).unwrap();
        target.stage_backup(&original, &staged).await.unwrap();
        assert_eq!(std::fs::read(&original).unwrap(), before);
        target.restore_from(&staged).await.unwrap();
        assert_eq!(
            Catalog::new(target.clone()).status().await.unwrap().cards,
            0
        );
        assert!(
            Decks::new(target.clone())
                .list(10, None)
                .await
                .unwrap()
                .items
                .is_empty()
        );
        let chat = Chat::new(target.clone());
        let conversation = chat
            .create(
                "after-upgrade",
                NewConversation {
                    title: "New local history".into(),
                    version_id: None,
                    share_deck: false,
                    share_notes: false,
                    share_history: false,
                },
            )
            .await
            .unwrap();
        assert_eq!(
            chat.get(&conversation.id).await.unwrap(),
            Some(conversation)
        );
        assert_eq!(std::fs::read(&original).unwrap(), before);
    }
    target.close().await.unwrap();
}

#[tokio::test]
async fn legacy_restore_preserves_versions_cards_notes_and_invalidates_cached_reads() {
    let dir = tempfile::tempdir().unwrap();
    let original = dir.path().join("legacy.sqlite3");
    legacy(&original, 8);
    let connection = Connection::open(&original).unwrap();
    connection.execute_batch(
        "INSERT INTO decks VALUES ('legacy-deck', 'Legacy Boros', 'standard', 1);
         INSERT INTO deck_versions(id,deck_id,ordinal,name,format,origin,created_at,sealed)
            VALUES ('legacy-version','legacy-deck',1,'Legacy Boros','standard','user',1,0);
         INSERT INTO deck_cards VALUES ('legacy-version','oracle:equipment',4,'main');
         UPDATE deck_versions SET sealed=1 WHERE id='legacy-version';
         INSERT INTO notes VALUES ('legacy-note','legacy-deck','legacy-version','Saved locally','user',1);
         INSERT INTO table_revisions VALUES ('deck_versions',7);",
    ).unwrap();
    drop(connection);
    let before = std::fs::read(&original).unwrap();
    let target = Engine::memory(4096).unwrap();
    let decks = Decks::new(target.clone());
    for _ in 0..2 {
        assert!(decks.list(10, None).await.unwrap().items.is_empty());
    }
    target.restore_from(&original).await.unwrap();
    let version = decks.version("legacy-version").await.unwrap().unwrap();
    assert_eq!(version.name, "Legacy Boros");
    assert_eq!(version.cards[0].quantity, 4);
    assert_eq!(
        decks.notes("legacy-deck", 10, None).await.unwrap().items[0].body,
        "Saved locally"
    );
    assert_eq!(
        decks.list(10, None).await.unwrap().items[0].version_count,
        1
    );
    assert_eq!(
        target.metrics().await.unwrap().table_revisions["deck_versions"],
        7
    );
    assert_eq!(std::fs::read(&original).unwrap(), before);
    target.close().await.unwrap();
}

#[tokio::test]
async fn forged_legacy_schemas_are_rejected_before_migration_or_restoration() {
    let dir = tempfile::tempdir().unwrap();
    let original = dir.path().join("forged.sqlite3");
    let staged = dir.path().join("staged.sqlite3");
    legacy(&original, 8);
    let connection = Connection::open(&original).unwrap();
    connection.execute_batch("CREATE TABLE unexpected (secret TEXT); INSERT INTO unexpected VALUES ('not application data');").unwrap();
    drop(connection);
    let before = std::fs::read(&original).unwrap();
    let target = Engine::memory(4096).unwrap();
    assert!(target.stage_backup(&original, &staged).await.is_err());
    assert!(!staged.exists());
    assert!(target.restore_from(&original).await.is_err());
    assert!(
        Decks::new(target.clone())
            .list(10, None)
            .await
            .unwrap()
            .items
            .is_empty()
    );
    assert_eq!(std::fs::read(&original).unwrap(), before);
    target.close().await.unwrap();
}

#[tokio::test]
async fn corrupt_legacy_rows_and_unversioned_databases_are_rejected_without_touching_sources() {
    let dir = tempfile::tempdir().unwrap();
    let target = Engine::memory(4096).unwrap();
    let corruptions = [
        "PRAGMA ignore_check_constraints=ON; INSERT INTO table_revisions VALUES ('bad', -1);",
        "PRAGMA foreign_keys=OFF; INSERT INTO deck_versions(id,deck_id,ordinal,name,format,origin,created_at,sealed)
            VALUES ('orphan','missing',1,'Orphan','standard','user',1,1);",
        "PRAGMA user_version=0;",
    ];
    for (index, corruption) in corruptions.iter().enumerate() {
        let source = dir.path().join(format!("invalid-{index}.sqlite3"));
        let staged = dir.path().join(format!("staged-{index}.sqlite3"));
        legacy(&source, 8);
        let connection = Connection::open(&source).unwrap();
        connection.execute_batch(corruption).unwrap();
        drop(connection);
        let before = std::fs::read(&source).unwrap();
        assert!(target.stage_backup(&source, &staged).await.is_err());
        assert!(!staged.exists());
        assert!(target.restore_from(&source).await.is_err());
        assert_eq!(std::fs::read(&source).unwrap(), before);
    }
    target.close().await.unwrap();
}
