use super::*;
use alkemyst_core::db::Statement;
use serde_json::json;

#[tokio::test]
async fn job_pages_seek_creation_cursors_and_use_the_version_index() {
    let (db, _, version) = setup().await;
    let statement = Statement {
        name: "test.simulations.plan",
        sql: include_str!("../../src/statements/simulations/list.sql"),
        reads: &[],
        cacheable: false,
    };
    let plan = db
        .explain(statement, vec![json!(100), json!(version), json!(3)])
        .await
        .unwrap()
        .join("\n");
    assert!(plan.contains("simulation_version_order"), "{plan}");
    assert!(plan.contains("rowid<?"), "{plan}");
    assert!(!plan.contains("SCAN simulation_runs"), "{plan}");
}
