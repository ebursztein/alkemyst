use super::*;
use crate::{db::Write, domain::Origin, statements::simulations};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
pub(super) fn row<T: DeserializeOwned>(rows: Vec<Value>) -> Result<Option<T>, Error> {
    rows.into_iter()
        .next()
        .map(serde_json::from_value)
        .transpose()
        .map_err(Error::from)
}
impl Analysis {
    pub async fn simulation(&self, id: &str) -> Result<Option<SimulationJob>, Error> {
        validation::identifier(id)?;
        row(self.db.read(simulations::GET, vec![json!(id)]).await?)
    }
    pub async fn simulation_inputs(&self, id: &str) -> Result<Option<SimulationInputs>, Error> {
        validation::identifier(id)?;
        row(self.db.read(simulations::INPUTS, vec![json!(id)]).await?)
    }
    pub async fn simulations(&self, q: JobQuery) -> Result<JobPage, Error> {
        if !(1..=100).contains(&q.limit) {
            return Err(Error::Invalid("limit must contain 1..=100 jobs"));
        }
        if let Some(id) = &q.version_id {
            validation::identifier(id)?;
        }
        let cursor = q
            .cursor
            .map(|c| {
                if c.len() != 19
                    || !c.starts_with("s1:")
                    || !c[3..].bytes().all(|b| b.is_ascii_hexdigit())
                {
                    return Err(Error::Invalid("invalid simulation cursor"));
                }
                i64::from_str_radix(&c[3..], 16)
                    .ok()
                    .filter(|n| *n > 0)
                    .ok_or(Error::Invalid("invalid simulation cursor"))
            })
            .transpose()?;
        row(self
            .db
            .read(
                simulations::LIST,
                vec![json!(cursor), json!(q.version_id), json!(q.limit)],
            )
            .await?)?
        .ok_or(Error::Stored)
    }
    pub async fn cancel_simulation(
        &self,
        id: &str,
        origin: Origin,
    ) -> Result<SimulationJob, Error> {
        validation::identifier(id)?;
        self.db
            .write(vec![Write {
                statement: simulations::CANCEL,
                params: vec![json!(id), json!(origin)],
                expected_rows: None,
            }])
            .await?;
        self.coordinator.notify.notify_one();
        self.simulation(id).await?.ok_or(Error::NotFound)
    }
}
