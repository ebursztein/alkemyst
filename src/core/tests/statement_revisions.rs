use alkemyst_core::db::{Engine, Error, Mutation, Statement, Write};
use serde_json::json;

const COUNT: Statement = Statement {
    name: "test.count",
    sql: "SELECT json_array(count(*)) FROM decks",
    reads: &["decks"],
    cacheable: true,
};
const INSERT: Mutation = Mutation {
    sql: "INSERT INTO decks VALUES (?1, 'Deck', 'standard', 0)",
    writes: &["decks"],
};
fn insert(id: &str) -> Write {
    Write {
        statement: INSERT,
        params: vec![json!(id)],
        expected_rows: Some(1),
    }
}

#[tokio::test]
async fn revisions_advance_once_per_commit_and_survive_reopening() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("revision.db");
    let db = Engine::open(&path, 4096).unwrap();
    db.write(vec![insert("a"), insert("b")]).await.unwrap();
    assert_eq!(
        db.metrics().await.unwrap().table_revisions.get("decks"),
        Some(&1)
    );
    assert!(db.write(vec![insert("c"), insert("a")]).await.is_err());
    assert_eq!(
        db.metrics().await.unwrap().table_revisions.get("decks"),
        Some(&1)
    );
    db.close().await.unwrap();
    let reopened = Engine::open(&path, 4096).unwrap();
    assert_eq!(
        reopened
            .metrics()
            .await
            .unwrap()
            .table_revisions
            .get("decks"),
        Some(&1)
    );
    reopened.write(vec![insert("c")]).await.unwrap();
    assert_eq!(
        reopened
            .metrics()
            .await
            .unwrap()
            .table_revisions
            .get("decks"),
        Some(&2)
    );
}

#[tokio::test]
async fn unrelated_or_noop_writes_do_not_invalidate_cached_reads() {
    let db = Engine::memory(4096).unwrap();
    db.read(COUNT, vec![]).await.unwrap();
    db.write(vec![Write {
        statement: Mutation {
            sql: "UPDATE decks SET name = 'Missing' WHERE id = 'absent'",
            writes: &["decks"],
        },
        params: vec![],
        expected_rows: Some(0),
    }])
    .await
    .unwrap();
    db.read(COUNT, vec![]).await.unwrap();
    // A scalar calculation has no table dependencies and survives a deck write.
    let independent = Statement {
        sql: "SELECT json_array(42)",
        reads: &[],
        ..COUNT
    };
    db.read(independent, vec![]).await.unwrap();
    db.write(vec![insert("a")]).await.unwrap();
    db.read(independent, vec![]).await.unwrap();
    assert_eq!(db.read(COUNT, vec![]).await.unwrap(), vec![json!([1])]);
    assert_eq!(db.metrics().await.unwrap().cache_hits, 2);
}

#[tokio::test]
async fn newer_database_schema_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("future.db");
    let db = Engine::open(&path, 0).unwrap();
    db.write(vec![Write {
        statement: Mutation {
            sql: "PRAGMA user_version=999",
            writes: &[],
        },
        params: vec![],
        expected_rows: None,
    }])
    .await
    .unwrap();
    db.close().await.unwrap();
    assert!(matches!(Engine::open(&path, 0), Err(Error::FutureSchema)));
}
