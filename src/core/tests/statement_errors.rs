use alkemyst_core::db::{Engine, Error, Mutation, Statement, Write};
use serde_json::json;

const VALUES: Statement = Statement {
    name: "test.values",
    sql: "SELECT json_array(?1, ?2, ?3, ?4)",
    reads: &[],
    cacheable: true,
};

#[tokio::test]
async fn scalar_bindings_and_noncacheable_queries_preserve_types() {
    let db = Engine::memory(4096).unwrap();
    let values = vec![json!(null), json!(true), json!(1.25), json!("test")];
    assert_eq!(
        db.read(VALUES, values.clone()).await.unwrap(),
        vec![json!([null, 1, 1.25, "test"])]
    );
    let uncached = Statement {
        cacheable: false,
        ..VALUES
    };
    db.read(uncached, values.clone()).await.unwrap();
    db.read(uncached, values).await.unwrap();
    assert_eq!(db.metrics().await.unwrap().cache_hits, 0);
    assert!(matches!(
        db.read(VALUES, vec![json!(u64::MAX)]).await,
        Err(Error::Binding)
    ));
}

#[tokio::test]
async fn invalid_reads_never_modify_data_or_poison_the_cache() {
    let db = Engine::memory(4096).unwrap();
    for sql in ["not SQL", "SELECT 1", "SELECT 'invalid json'"] {
        assert!(db.read(Statement { sql, ..VALUES }, vec![]).await.is_err());
    }
    assert!(matches!(
        db.read(
            Statement {
                sql: "DELETE FROM decks",
                ..VALUES
            },
            vec![]
        )
        .await,
        Err(Error::NotReadOnly)
    ));
    assert!(
        db.explain(
            Statement {
                sql: "not SQL",
                ..VALUES
            },
            vec![]
        )
        .await
        .is_err()
    );
    assert!(db.read(VALUES, vec![json!(0)]).await.is_err());
    assert!(db.read(VALUES, vec![json!([])]).await.is_err());
}

#[tokio::test]
async fn invalid_write_parameters_roll_back_prior_work() {
    let db = Engine::memory(4096).unwrap();
    let insert = Mutation {
        sql: "INSERT INTO decks VALUES (?1, 'Deck', 'standard', 0)",
        writes: &["decks"],
    };
    assert!(
        db.write(vec![
            Write {
                statement: insert,
                params: vec![json!("a")],
                expected_rows: Some(1)
            },
            Write {
                statement: insert,
                params: vec![json!({})],
                expected_rows: Some(1)
            },
        ])
        .await
        .is_err()
    );
    assert_eq!(
        db.read(
            Statement {
                sql: "SELECT json_array(count(*)) FROM decks",
                reads: &["decks"],
                ..VALUES
            },
            vec![]
        )
        .await
        .unwrap(),
        vec![json!([0])]
    );
    db.write(vec![]).await.unwrap();
    db.close().await.unwrap();
    assert!(matches!(db.metrics().await, Err(Error::Closed)));
    assert!(matches!(db.close().await, Err(Error::Closed)));
}

#[tokio::test]
async fn bounded_cache_evicts_least_recently_used_result() {
    let db = Engine::memory(300).unwrap();
    let query = Statement {
        sql: "SELECT json_array(?1)",
        ..VALUES
    };
    db.read(query, vec![json!("a".repeat(60))]).await.unwrap();
    db.read(query, vec![json!("b".repeat(60))]).await.unwrap();
    db.read(query, vec![json!("b".repeat(60))]).await.unwrap();
    db.read(query, vec![json!("a".repeat(60))]).await.unwrap();
    let stats = db.metrics().await.unwrap();
    assert_eq!(stats.cache_hits, 1);
    assert!(stats.cache_bytes <= 300);
}
