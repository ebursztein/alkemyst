use super::*;
use crate::{analysis::DeckSnapshot, statements::practice};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
pub(super) fn row<T: DeserializeOwned>(rows: Vec<Value>) -> Result<Option<T>, Error> {
    rows.into_iter()
        .next()
        .map(serde_json::from_value)
        .transpose()
        .map_err(Error::from)
}
pub(super) fn identifier(value: &str) -> Result<(), Error> {
    if value.is_empty() || value.len() > 256 {
        return Err(Error::Invalid("identifier must contain 1..=256 bytes"));
    }
    Ok(())
}
pub(super) fn key(value: &str) -> Result<(), Error> {
    if value.is_empty() || value.len() > 128 || value.chars().any(char::is_control) {
        return Err(Error::Invalid("invalid idempotency key"));
    }
    Ok(())
}
impl Practice {
    pub(super) async fn stored(&self, id: &str) -> Result<Option<Stored>, Error> {
        identifier(id)?;
        row(self.db.read(practice::GET, vec![json!(id)]).await?)
    }
    pub async fn get(&self, id: &str) -> Result<Option<PracticeView>, Error> {
        self.stored(id).await?.map(cards::view).transpose()
    }
    pub async fn inputs(&self, id: &str) -> Result<Option<DeckSnapshot>, Error> {
        identifier(id)?;
        row(self.db.read(practice::INPUTS, vec![json!(id)]).await?)
    }
    pub async fn list(&self, q: PracticeQuery) -> Result<PracticePage, Error> {
        if !(1..=100).contains(&q.limit) {
            return Err(Error::Invalid("limit must contain 1..=100 sessions"));
        }
        if let Some(id) = &q.version_id {
            identifier(id)?;
        }
        let cursor = q
            .cursor
            .map(|c| {
                if c.len() != 19
                    || !c.starts_with("p1:")
                    || !c[3..].bytes().all(|b| b.is_ascii_hexdigit())
                {
                    return Err(Error::Invalid("invalid practice cursor"));
                }
                i64::from_str_radix(&c[3..], 16)
                    .ok()
                    .filter(|n| *n > 0)
                    .ok_or(Error::Invalid("invalid practice cursor"))
            })
            .transpose()?;
        row(self
            .db
            .read(
                practice::LIST,
                vec![json!(cursor), json!(q.version_id), json!(q.limit)],
            )
            .await?)?
        .ok_or(Error::Incompatible)
    }
}
