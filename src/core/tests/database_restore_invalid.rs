use alkemyst_core::db::Engine;

#[tokio::test]
async fn structurally_valid_sqlite_with_foreign_schema_or_invalid_data_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let pristine = dir.path().join("pristine.sqlite3");
    let engine = Engine::memory(4096).unwrap();
    engine.backup_to(&pristine).await.unwrap();
    // Deliberately bypass application writes to model externally damaged backup files.
    // Production SQLite access remains confined to the statement engine.
    for (index, fault, message) in [
        (
            0,
            "CREATE TRIGGER sqliteEvil BEFORE INSERT ON decks
                BEGIN SELECT RAISE(ABORT, 'unexpected trigger'); END;",
            "unsupported database schema",
        ),
        (
            1,
            "PRAGMA foreign_keys=OFF;
                INSERT INTO notes VALUES ('bad-note', 'missing-deck', NULL, 'note', 'user', 0);",
            "foreign key check failed",
        ),
        (
            2,
            "PRAGMA ignore_check_constraints=ON;
                INSERT INTO table_revisions VALUES ('invalid', -1);",
            "integrity check failed",
        ),
        (3, "PRAGMA user_version=0;", "unsupported database schema"),
    ] {
        let input = dir.path().join(format!("fault-{index}.sqlite3"));
        std::fs::copy(&pristine, &input).unwrap();
        let connection = rusqlite::Connection::open(&input).unwrap();
        connection.execute_batch(fault).unwrap();
        if index == 2 {
            connection
                .execute_batch("PRAGMA ignore_check_constraints=OFF;")
                .unwrap();
            let check: String = connection
                .query_row("PRAGMA integrity_check", [], |r| r.get(0))
                .unwrap();
            assert_ne!(check, "ok", "fault did not create a constraint violation");
        }
        connection.close().unwrap();
        let before = std::fs::read(&input).unwrap();
        let result = engine.restore_from(&input).await;
        assert!(result.is_err(), "backup fault {index} was accepted");
        let error = result.unwrap_err();
        assert!(error.to_string().contains(message), "{error}");
        assert_eq!(std::fs::read(&input).unwrap(), before);
        // Rejection leaves the current schema and contents usable for another snapshot.
        let after = dir.path().join(format!("after-{index}.sqlite3"));
        engine.backup_to(&after).await.unwrap();
        let other = Engine::open(&after, 4096).unwrap();
        other.close().await.unwrap();
    }
    engine.close().await.unwrap();
}
