use super::worker::{Owned, mutate};
use super::*;
use crate::{
    analysis::{
        compiler::{COMPILER_VERSION, compile},
        seed,
        simulation::{BattlefieldRunner, MODEL_VERSION, POLICY, SimulationReport},
    },
    db::Engine,
    statements::simulations,
};
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
#[derive(Deserialize)]
struct Control {
    cancel: bool,
}
fn runner(job: &Owned) -> Result<BattlefieldRunner, Error> {
    if job.compiler_version != COMPILER_VERSION
        || job.model_version != MODEL_VERSION
        || job.policy_version != POLICY
        || job.replay_version != landlord::REPLAY_VERSION
    {
        return Err(Error::Stored);
    }
    let inputs: SimulationInputs = serde_json::from_value(job.inputs.clone())?;
    let compiled = compile(inputs.snapshot);
    let p = inputs.parameters;
    let mulligan = validation::mulligan(&p, &compiled)?;
    let scenario = compiled
        .scenario(p.turns, p.on_the_play, mulligan, p.goal, true)
        .map_err(|_| Error::Unsupported)?;
    let seed = seed::parse(p.seed.as_deref().ok_or(Error::Stored)?).map_err(Error::Invalid)?;
    Ok(BattlefieldRunner::new(scenario, p.runs, seed)?)
}
async fn progress(
    db: &Engine,
    id: &str,
    owner: &str,
    report: Option<&SimulationReport>,
    status: &str,
    error: Option<&str>,
) -> Result<(), Error> {
    mutate(
        db,
        simulations::PROGRESS,
        vec![
            json!(id),
            json!(owner),
            json!(report.map_or(0, |r| r.completed_runs)),
            json!(report.map(serde_json::to_string).transpose()?),
            json!(status),
            json!(error),
        ],
    )
    .await
}
pub(super) async fn execute(db: &Engine, job: Owned, owner: &str, stop: &CancellationToken) {
    let mut runner = match runner(&job) {
        Ok(r) => r,
        Err(_) => {
            let _ = progress(
                db,
                &job.id,
                owner,
                None,
                "failed",
                Some("incompatible_model"),
            )
            .await;
            return;
        }
    };
    loop {
        let report = runner.report();
        if stop.is_cancelled() {
            let _ = progress(
                db,
                &job.id,
                owner,
                report.as_ref(),
                "interrupted",
                Some("interrupted"),
            )
            .await;
            return;
        }
        let control = store::row::<Control>(
            match db
                .read(simulations::CONTROL, vec![json!(job.id), json!(owner)])
                .await
            {
                Ok(v) => v,
                Err(_) => return,
            },
        );
        match control {
            Ok(Some(c)) if !c.cancel => {}
            Ok(Some(_)) => {
                let _ = progress(db, &job.id, owner, report.as_ref(), "cancelled", None).await;
                return;
            }
            _ => return,
        }
        let mut task = tokio::task::spawn_blocking(move || {
            runner.advance(8)?;
            Ok::<_, landlord::Error>(runner)
        });
        let mut heartbeat = tokio::time::interval(Duration::from_secs(5));
        let result = loop {
            tokio::select! {
                result=&mut task=>break result,
                _=heartbeat.tick()=>{let _=mutate(db,simulations::HEARTBEAT,vec![json!(job.id),json!(owner)]).await;}
            }
        };
        runner = match result {
            Ok(Ok(r)) => r,
            _ => {
                let _ = progress(
                    db,
                    &job.id,
                    owner,
                    report.as_ref(),
                    "failed",
                    Some("worker_failed"),
                )
                .await;
                return;
            }
        };
        let report = runner.report();
        let status = if stop.is_cancelled() {
            "interrupted"
        } else if runner.finished() {
            "completed"
        } else {
            "running"
        };
        if progress(
            db,
            &job.id,
            owner,
            report.as_ref(),
            status,
            (status == "interrupted").then_some("interrupted"),
        )
        .await
        .is_err()
        {
            return;
        }
        if status != "running" {
            return;
        }
    }
}
