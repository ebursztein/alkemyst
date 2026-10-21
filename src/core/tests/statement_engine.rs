use alkemyst_core::db::{Engine, Mutation, Statement, Write};
use serde_json::json;

const DECKS: Statement = Statement {
    name: "test.decks",
    sql: "SELECT json_object('id', id, 'name', name) FROM decks ORDER BY id",
    reads: &["decks"],
    cacheable: true,
};
const INSERT: Mutation = Mutation {
    sql: "INSERT INTO decks(id, name, format, created_at) VALUES (?1, ?2, 'standard', 0)",
    writes: &["decks"],
};
fn deck(id: &str) -> Write {
    Write {
        statement: INSERT,
        params: vec![json!(id), json!("Boros")],
        expected_rows: Some(1),
    }
}

#[tokio::test]
async fn cache_reuses_results_and_committed_writes_invalidate() {
    let db = Engine::memory(4096).unwrap();
    assert!(db.read(DECKS, vec![]).await.unwrap().is_empty());
    assert!(db.read(DECKS, vec![]).await.unwrap().is_empty());
    let stats = db.metrics().await.unwrap();
    assert_eq!(
        (stats.read_calls, stats.statements, stats.cache_hits),
        (2, 1, 1)
    );
    db.write(vec![deck("a")]).await.unwrap();
    let rows = db.read(DECKS, vec![]).await.unwrap();
    assert_eq!(rows, vec![json!({"id":"a", "name":"Boros"})]);
    assert_eq!(db.read(DECKS, vec![]).await.unwrap(), rows);
    let stats = db.metrics().await.unwrap();
    assert_eq!((stats.read_calls, stats.cache_hits), (4, 2));
}

#[tokio::test]
async fn failed_transactions_roll_back_and_preserve_cached_results() {
    let db = Engine::memory(4096).unwrap();
    db.read(DECKS, vec![]).await.unwrap();
    assert!(db.write(vec![deck("same"), deck("same")]).await.is_err());
    assert!(db.read(DECKS, vec![]).await.unwrap().is_empty());
    let stats = db.metrics().await.unwrap();
    assert_eq!(stats.cache_hits, 1);
    assert_eq!(stats.failed_transactions, 1);
}

#[tokio::test]
async fn row_assertions_roll_back_all_writes() {
    let db = Engine::memory(4096).unwrap();
    let mut write = deck("a");
    write.expected_rows = Some(2);
    assert!(db.write(vec![write]).await.is_err());
    assert!(db.read(DECKS, vec![]).await.unwrap().is_empty());
}

#[tokio::test]
async fn parameters_and_cache_budget_are_respected() {
    let db = Engine::memory(1).unwrap();
    db.write(vec![deck("a")]).await.unwrap();
    for _ in 0..2 {
        assert_eq!(db.read(DECKS, vec![]).await.unwrap().len(), 1);
    }
    assert_eq!(db.metrics().await.unwrap().cache_hits, 0);
    let filtered = Statement {
        sql: "SELECT json_object('id', id) FROM decks WHERE id = ?1",
        ..DECKS
    };
    assert!(
        db.read(filtered, vec![json!("missing")])
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        db.read(filtered, vec![json!("a")]).await.unwrap(),
        vec![json!({"id":"a"})]
    );
    assert!(
        db.read(filtered, vec![json!({"invalid":"binding"})])
            .await
            .is_err()
    );
}

#[tokio::test]
async fn different_callers_share_cache_and_writes_are_serialized() {
    let db = Engine::memory(4096).unwrap();
    let other = db.clone();
    let (first, second) = tokio::join!(db.write(vec![deck("a")]), other.write(vec![deck("b")]));
    first.unwrap();
    second.unwrap();
    let expected = db.read(DECKS, vec![]).await.unwrap();
    assert_eq!(other.read(DECKS, vec![]).await.unwrap(), expected);
    assert_eq!(expected.len(), 2);
    assert_eq!(db.metrics().await.unwrap().cache_hits, 1);
}

#[tokio::test]
async fn persistent_database_reopens_with_migrations_and_uses_index() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.sqlite3");
    let db = Engine::open(&path, 4096).unwrap();
    db.write(vec![deck("a")]).await.unwrap();
    let plan = db
        .explain(
            Statement {
                sql: "SELECT json_object('id', id) FROM decks WHERE id = ?1",
                ..DECKS
            },
            vec![json!("a")],
        )
        .await
        .unwrap();
    assert!(plan.iter().any(|step| step.contains("INDEX")));
    drop(db);
    let reopened = Engine::open(&path, 4096).unwrap();
    assert_eq!(reopened.read(DECKS, vec![]).await.unwrap().len(), 1);
    assert!(Engine::open(&dir.path().join("absent/fail.db"), 4096).is_err());
}
