#[path = "simulation_jobs/support.rs"]
mod support;
use alkemyst_core::{
    analysis::jobs::{JobQuery, JobStatus},
    domain::Origin,
};
use support::*;

#[tokio::test]
async fn jobs_pin_inputs_and_idempotent_retries_do_not_duplicate() {
    let (db, analysis, version) = setup().await;
    let parameters = parameters(&version);
    let first = analysis
        .create_simulation("job-once", parameters.clone(), Origin::User)
        .await
        .unwrap();
    assert_eq!(first.status, JobStatus::Queued);
    assert_eq!(first.seed, "ffffffffffffffff");
    let retried = analysis
        .create_simulation("job-once", parameters.clone(), Origin::Agent)
        .await
        .unwrap();
    assert_eq!(retried.id, first.id);
    assert_eq!(retried.origin, Origin::User);
    let frozen = analysis
        .simulation_inputs(&first.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(frozen.snapshot.version_id, version);
    assert_eq!(frozen.snapshot.counts.main, 60);
    let page = analysis.simulations(JobQuery::default()).await.unwrap();
    assert_eq!(page.items.len(), 1);
    let mut changed = parameters;
    changed.runs += 1;
    assert!(
        analysis
            .create_simulation("job-once", changed, Origin::User)
            .await
            .is_err()
    );
    let before = db.metrics().await.unwrap().statements;
    assert_eq!(
        analysis.simulation(&first.id).await.unwrap().unwrap(),
        first
    );
    assert!(db.metrics().await.unwrap().statements <= before + 1);
}
#[tokio::test]
async fn cancelling_a_queued_job_is_idempotent_and_keeps_zero_trials_unknown() {
    let (_, analysis, version) = setup().await;
    let job = analysis
        .create_simulation("cancel-queued", parameters(&version), Origin::User)
        .await
        .unwrap();
    let cancelled = analysis
        .cancel_simulation(&job.id, Origin::Agent)
        .await
        .unwrap();
    assert_eq!(cancelled.status, JobStatus::Cancelled);
    assert_eq!(cancelled.completed_runs, 0);
    assert!(cancelled.report.is_none());
    assert_eq!(
        analysis
            .cancel_simulation(&job.id, Origin::User)
            .await
            .unwrap(),
        cancelled
    );
    assert!(
        analysis
            .cancel_simulation("missing", Origin::User)
            .await
            .is_err()
    );
}

#[tokio::test]
async fn worker_completes_seeded_jobs_and_replays_the_frozen_inputs() {
    let (_, analysis, version) = setup().await;
    let p = parameters(&version);
    let first = analysis
        .create_simulation("worker-one", p.clone(), Origin::User)
        .await
        .unwrap();
    let second = analysis
        .create_simulation("worker-two", p, Origin::Agent)
        .await
        .unwrap();
    analysis.start_jobs().await.unwrap();
    let one = finished(&analysis, &first.id).await;
    let two = finished(&analysis, &second.id).await;
    assert_eq!(one.status, JobStatus::Completed);
    assert_eq!(one.completed_runs, 100);
    assert_eq!(one.report, two.report);
    assert_eq!(one.report.as_ref().unwrap().turns[0].joint.samples, 100);
    analysis.stop_jobs().await.unwrap();
}
#[tokio::test]
async fn cancellation_preserves_only_completed_trial_denominators() {
    let (_, analysis, version) = setup().await;
    let mut p = parameters(&version);
    p.runs = 1_000_000;
    let job = analysis
        .create_simulation("worker-cancel", p, Origin::User)
        .await
        .unwrap();
    analysis.start_jobs().await.unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            if analysis
                .simulation(&job.id)
                .await
                .unwrap()
                .unwrap()
                .completed_runs
                > 0
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    analysis
        .cancel_simulation(&job.id, Origin::Agent)
        .await
        .unwrap();
    let done = finished(&analysis, &job.id).await;
    assert_eq!(done.status, JobStatus::Cancelled);
    assert!(done.completed_runs < done.requested_runs);
    assert_eq!(
        done.report.as_ref().unwrap().turns[0].joint.samples,
        done.completed_runs
    );
    analysis.stop_jobs().await.unwrap();
}
async fn finished(
    analysis: &alkemyst_core::analysis::Analysis,
    id: &str,
) -> alkemyst_core::analysis::jobs::SimulationJob {
    tokio::time::timeout(std::time::Duration::from_secs(15), async {
        loop {
            let job = analysis.simulation(id).await.unwrap().unwrap();
            if !matches!(
                job.status,
                JobStatus::Queued | JobStatus::Running | JobStatus::Cancelling
            ) {
                return job;
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap()
}

#[path = "simulation_jobs/bounds.rs"]
mod bounds;
#[path = "simulation_jobs/durability.rs"]
mod durability;
#[path = "simulation_jobs/plans.rs"]
mod plans;
