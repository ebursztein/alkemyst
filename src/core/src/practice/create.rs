use super::*;
use crate::{
    analysis::seed,
    db::Write,
    domain::Origin,
    statements::{decks, practice},
};
use serde::Deserialize;
use serde_json::json;
#[derive(Deserialize)]
struct Receipt {
    operation: String,
    body: String,
    value: Option<Stored>,
}
impl Practice {
    pub(super) async fn receipt(
        &self,
        key: &str,
        operation: &str,
        body: &str,
    ) -> Result<Option<PracticeView>, Error> {
        let Some(receipt) =
            store::row::<Receipt>(self.db.read(practice::RECEIPT, vec![json!(key)]).await?)?
        else {
            return Ok(None);
        };
        if receipt.operation != operation || receipt.body != body {
            return Err(Error::Conflict);
        }
        Ok(Some(cards::view(
            receipt.value.ok_or(Error::Incompatible)?,
        )?))
    }
    pub async fn create(
        &self,
        key: &str,
        mut input: NewPractice,
        origin: Origin,
    ) -> Result<PracticeView, Error> {
        store::key(key)?;
        store::identifier(&input.version_id)?;
        if let Some(seed) = &mut input.seed {
            *seed = seed::encode(seed::parse(seed).map_err(Error::Invalid)?);
        }
        let body = serde_json::to_string(&input)?;
        if let Some(value) = self.receipt(key, "practice.create", &body).await? {
            return Ok(value);
        }
        let snapshot = crate::analysis::read_snapshot(&self.db, &input.version_id)
            .await?
            .ok_or(Error::NotFound)?;
        let total = usize::try_from(snapshot.counts.main)
            .map_err(|_| Error::Invalid("invalid deck size"))?;
        if total == 0 || total > 1000 {
            return Err(Error::Invalid("practice needs 1..=1000 main-deck cards"));
        }
        let seed = input
            .seed
            .as_deref()
            .map(seed::parse)
            .transpose()
            .map_err(Error::Invalid)?
            .unwrap_or_else(|| uuid::Uuid::new_v4().as_u128() as u64);
        let id = uuid::Uuid::new_v4().to_string();
        let step = uuid::Uuid::new_v4().to_string();
        let state = state::State::deal(total, seed, 0);
        let writes = vec![
            Write {
                statement: decks::RECEIPT,
                params: vec![
                    json!(key),
                    json!("practice.create"),
                    json!(body),
                    json!(step),
                ],
                expected_rows: Some(1),
            },
            Write {
                statement: practice::CREATE,
                params: vec![
                    json!(id),
                    json!(snapshot.version_id),
                    json!(origin),
                    json!(seed::encode(seed)),
                    json!(PRACTICE_VERSION),
                    json!(landlord::REPLAY_VERSION),
                    json!(input.on_the_play),
                    json!(serde_json::to_string(&snapshot)?),
                    json!(key),
                    json!(step),
                ],
                expected_rows: Some(1),
            },
            Write {
                statement: practice::STEP,
                params: vec![
                    json!(step),
                    json!(id),
                    json!(0),
                    json!(origin),
                    json!(null),
                    json!(serde_json::to_string(&state)?),
                    json!(-1),
                ],
                expected_rows: Some(1),
            },
        ];
        self.commit(writes, key, "practice.create", &body).await
    }
    pub(super) async fn commit(
        &self,
        writes: Vec<Write>,
        key: &str,
        operation: &str,
        body: &str,
    ) -> Result<PracticeView, Error> {
        match self.db.write(writes).await {
            Ok(()) => {}
            Err(crate::db::Error::RowCount {
                expected: 1,
                actual: 0,
            }) => {
                return self
                    .receipt(key, operation, body)
                    .await?
                    .ok_or(Error::Conflict);
            }
            Err(error) => return Err(error.into()),
        }
        self.receipt(key, operation, body)
            .await?
            .ok_or(Error::Incompatible)
    }
}
