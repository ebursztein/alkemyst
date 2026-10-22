use super::*;
use crate::statements::chat;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
pub(super) fn row<T: DeserializeOwned>(rows: Vec<Value>) -> Result<Option<T>> {
    rows.into_iter()
        .next()
        .map(serde_json::from_value)
        .transpose()
        .map_err(Into::into)
}
impl Chat {
    pub async fn get(&self, id: &str) -> Result<Option<Conversation>> {
        validation::text(id, 256)?;
        row(self.db.read(chat::GET, vec![json!(id)]).await?)
    }
    pub async fn list(&self, query: ChatQuery) -> Result<ConversationPage> {
        let cursor = validation::page(&query, "c1:")?;
        row(self
            .db
            .read(chat::LIST, vec![json!(cursor), json!(query.limit)])
            .await?)?
        .ok_or(Error::NotFound)
    }
    pub async fn messages(&self, id: &str, query: ChatQuery) -> Result<Option<MessagePage>> {
        validation::text(id, 256)?;
        let cursor = validation::page(&query, "m1:")?;
        row(self
            .db
            .read(
                chat::MESSAGES,
                vec![json!(id), json!(cursor), json!(query.limit)],
            )
            .await?)
    }
}
