use super::*;
use crate::{
    analysis::{
        compiler::COMPILER_VERSION,
        seed,
        simulation::{BattlefieldRunner, MODEL_VERSION, POLICY},
    },
    db::Write,
    domain::Origin,
    statements::{decks, simulations},
};
use serde::Deserialize;
use serde_json::json;
#[derive(Deserialize)]
struct Receipt {
    operation: String,
    body: String,
    job: Option<SimulationJob>,
}
impl Analysis {
    async fn simulation_receipt(
        &self,
        key: &str,
        body: &str,
    ) -> Result<Option<SimulationJob>, Error> {
        let Some(receipt) =
            store::row::<Receipt>(self.db.read(simulations::RECEIPT, vec![json!(key)]).await?)?
        else {
            return Ok(None);
        };
        if receipt.operation != "simulations.create" || receipt.body != body {
            return Err(Error::Conflict);
        }
        Ok(Some(receipt.job.ok_or(Error::Stored)?))
    }
    pub async fn create_simulation(
        &self,
        key: &str,
        mut p: SimulationParameters,
        origin: Origin,
    ) -> Result<SimulationJob, Error> {
        validation::request(key, &mut p)?;
        let body = serde_json::to_string(&p)?;
        if let Some(job) = self.simulation_receipt(key, &body).await? {
            return Ok(job);
        }
        if self.coordinator.stop.is_cancelled() {
            return Err(Error::Stopped);
        }
        let compiled = self.prepare(&p.version_id).await?.ok_or(Error::NotFound)?;
        let mulligan = validation::mulligan(&p, &compiled)?;
        let scenario = compiled
            .scenario(p.turns, p.on_the_play, mulligan, p.goal, true)
            .map_err(|error| match error {
                crate::analysis::compiler::Error::Configuration(e) => Error::Model(e),
                _ => Error::Unsupported,
            })?;
        let seed = p
            .seed
            .as_deref()
            .map(seed::parse)
            .transpose()
            .map_err(Error::Invalid)?
            .unwrap_or_else(|| uuid::Uuid::new_v4().as_u128() as u64);
        BattlefieldRunner::new(scenario, p.runs, seed)?;
        p.seed = Some(seed::encode(seed));
        let id = uuid::Uuid::new_v4().to_string();
        let writes = vec![
            Write {
                statement: decks::RECEIPT,
                params: vec![
                    json!(key),
                    json!("simulations.create"),
                    json!(body),
                    json!(id),
                ],
                expected_rows: Some(1),
            },
            Write {
                statement: simulations::CREATE,
                params: vec![
                    json!(key),
                    json!(p.version_id),
                    json!(origin),
                    json!(p.runs),
                    json!(p.seed),
                    json!(COMPILER_VERSION),
                    json!(MODEL_VERSION),
                    json!(POLICY),
                    json!(landlord::REPLAY_VERSION),
                    json!(serde_json::to_string(&p)?),
                    json!(serde_json::to_string(compiled.snapshot())?),
                    json!(serde_json::to_string(&compiled.profile)?),
                ],
                expected_rows: Some(1),
            },
        ];
        match self.db.write(writes).await {
            Ok(()) => {}
            Err(crate::db::Error::RowCount {
                expected: 1,
                actual: 0,
            }) => {
                return self
                    .simulation_receipt(key, &body)
                    .await?
                    .ok_or(Error::QueueFull);
            }
            Err(error) => return Err(error.into()),
        }
        self.coordinator.notify.notify_one();
        self.simulation_receipt(key, &body)
            .await?
            .ok_or(Error::Stored)
    }
}
