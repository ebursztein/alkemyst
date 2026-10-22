use super::*;
use alkemyst_core::{
    analysis::Analysis,
    catalog::{Catalog, CatalogSource},
    db::{Engine, Mutation, Write},
};
use serde_json::{Value, json};

#[tokio::test]
async fn queued_jobs_survive_disk_reopen_with_the_same_frozen_inputs() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("simulations.sqlite3");
    let (db, analysis, version) = populate(Engine::open(&path, 8 * 1024 * 1024).unwrap()).await;
    let job = analysis
        .create_simulation("disk-job", parameters(&version), Origin::User)
        .await
        .unwrap();
    let inputs = analysis.simulation_inputs(&job.id).await.unwrap().unwrap();
    db.close().await.unwrap();
    drop(analysis);
    let db = Engine::open(&path, 8 * 1024 * 1024).unwrap();
    let analysis = Analysis::new(db.clone());
    assert_eq!(
        analysis.simulation_inputs(&job.id).await.unwrap().unwrap(),
        inputs
    );
    analysis.start_jobs().await.unwrap();
    let done = finished(&analysis, &job.id).await;
    assert_eq!(done.status, JobStatus::Completed);
    assert_eq!(done.seed, job.seed);
    analysis.stop_jobs().await.unwrap();
    db.close().await.unwrap();
}
#[tokio::test]
async fn catalog_refresh_does_not_rewrite_a_queued_job_or_its_idempotent_receipt() {
    let (db, analysis, version) = setup().await;
    let p = parameters(&version);
    let job = analysis
        .create_simulation("frozen-job", p.clone(), Origin::User)
        .await
        .unwrap();
    let rows: Value =
        serde_json::from_str(include_str!("../fixtures/boros-equipment.json")).unwrap();
    let lines = rows["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            let mut c = e["card"].clone();
            c["id"] = json!(format!("new-{}", c["oracle_id"].as_str().unwrap()));
            c["object"] = json!("card");
            c["lang"] = json!("en");
            c["released_at"] = json!("2026-01-01");
            if c["name"] == "Cloud, Midgar Mercenary" {
                c["oracle_text"] = json!("Changed resource effect");
            }
            c.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    Catalog::new(db)
        .import_jsonl(
            std::io::Cursor::new(lines),
            CatalogSource {
                url: "https://data.scryfall.io/default-cards/fixture.jsonl.gz".into(),
                updated_at: "2026-10-22T00:00:00Z".into(),
            },
        )
        .await
        .unwrap();
    assert!(
        !analysis
            .profile(&version)
            .await
            .unwrap()
            .unwrap()
            .can_simulate
    );
    assert_eq!(
        analysis
            .create_simulation("frozen-job", p.clone(), Origin::Agent)
            .await
            .unwrap()
            .id,
        job.id
    );
    assert!(
        analysis
            .create_simulation("new-invalid-model", p, Origin::User)
            .await
            .is_err()
    );
    analysis.start_jobs().await.unwrap();
    assert_eq!(
        finished(&analysis, &job.id).await.status,
        JobStatus::Completed
    );
    analysis.stop_jobs().await.unwrap();
}
#[tokio::test]
async fn shutdown_interrupts_and_stale_owners_recover_without_reexecuting() {
    let (db, analysis, version) = setup().await;
    let mut p = parameters(&version);
    p.runs = 1_000_000;
    let job = analysis
        .create_simulation("shutdown-job", p, Origin::User)
        .await
        .unwrap();
    analysis.start_jobs().await.unwrap();
    analysis.stop_jobs().await.unwrap();
    assert_eq!(
        analysis.simulation(&job.id).await.unwrap().unwrap().status,
        JobStatus::Interrupted
    );
    let next = Analysis::new(db.clone());
    let stale = next
        .create_simulation("stale-job", parameters(&version), Origin::User)
        .await
        .unwrap();
    db.write(vec![Write{statement:Mutation{sql:"UPDATE simulation_runs SET status='running',owner='dead-owner',updated_at=1 WHERE id=?1",writes:&["simulation_runs"]},params:vec![json!(stale.id)],expected_rows:Some(1)}]).await.unwrap();
    next.start_jobs().await.unwrap();
    let recovered = next.simulation(&stale.id).await.unwrap().unwrap();
    assert_eq!(recovered.status, JobStatus::Interrupted);
    assert!(recovered.report.is_none());
    next.stop_jobs().await.unwrap();
}
