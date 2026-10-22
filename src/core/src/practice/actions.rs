use super::*;
use crate::{
    analysis::seed,
    db::Write,
    domain::Origin,
    statements::{decks, practice},
};
use serde_json::json;
impl Practice {
    pub async fn act(
        &self,
        key: &str,
        id: &str,
        expected_revision: u32,
        action: PracticeAction,
        origin: Origin,
    ) -> Result<PracticeView, Error> {
        store::key(key)?;
        store::identifier(id)?;
        let body =
            json!({"id":id,"expected_revision":expected_revision,"action":action}).to_string();
        if let Some(value) = self.receipt(key, "practice.action", &body).await? {
            return Ok(value);
        }
        let mut row = self.stored(id).await?.ok_or(Error::NotFound)?;
        if row.revision != expected_revision {
            return Err(Error::Conflict);
        }
        if row.revision >= 1000 {
            return Err(Error::Invalid(
                "practice session is bounded to one thousand actions",
            ));
        }
        if row.practice_version != PRACTICE_VERSION
            || row.replay_version != landlord::REPLAY_VERSION
        {
            return Err(Error::Incompatible);
        }
        let seed = seed::parse(&row.seed).map_err(Error::Invalid)?;
        row.state.apply(
            &action,
            &cards::physical(&row.snapshot),
            seed,
            row.on_the_play,
        )?;
        let step = uuid::Uuid::new_v4().to_string();
        self.commit(
            vec![
                Write {
                    statement: decks::RECEIPT,
                    params: vec![
                        json!(key),
                        json!("practice.action"),
                        json!(body),
                        json!(step),
                    ],
                    expected_rows: Some(1),
                },
                Write {
                    statement: practice::STEP,
                    params: vec![
                        json!(step),
                        json!(id),
                        json!(row.revision + 1),
                        json!(origin),
                        json!(serde_json::to_string(&action)?),
                        json!(serde_json::to_string(&row.state)?),
                        json!(row.revision),
                    ],
                    expected_rows: Some(1),
                },
            ],
            key,
            "practice.action",
            &body,
        )
        .await
    }
}
