use super::{
    Decks, Error, NewNote, Note, NotePage, Origin, Result,
    decks::{now, write},
    validation,
};
use crate::statements::{decks, notes};
use serde_json::json;

impl Decks {
    pub async fn create_note(&self, key: &str, input: NewNote, origin: Origin) -> Result<Note> {
        validation::request_key(key)?;
        if input.body.trim().is_empty() || input.body.len() > 20000 {
            return Err(Error::Invalid(
                "note must be nonempty and at most 20000 bytes",
            ));
        }
        let id = uuid::Uuid::new_v4().to_string();
        let body = json!({"origin":origin,"input":input}).to_string();
        let writes = vec![
            write(
                decks::RECEIPT,
                vec![json!(key), json!("notes.create"), json!(body), json!(id)],
                Some(1),
            ),
            write(
                notes::CREATE,
                vec![
                    json!(id),
                    json!(input.deck_id),
                    json!(input.version_id),
                    json!(input.body),
                    json!(origin),
                    json!(now()?),
                    json!(key),
                ],
                None,
            ),
        ];
        match self.db.write(writes).await {
            Err(crate::db::Error::RowCount { .. }) => return Err(Error::Conflict),
            result => result?,
        }
        let result = self.db.read(notes::GET, vec![json!(key)]).await?;
        Ok(serde_json::from_value(
            result.into_iter().next().ok_or(Error::NotFound)?,
        )?)
    }

    pub async fn notes(
        &self,
        deck_id: &str,
        limit: usize,
        cursor: Option<String>,
    ) -> Result<NotePage> {
        if !(1..=100).contains(&limit) || cursor.as_ref().is_some_and(|c| c.len() > 128) {
            return Err(Error::Invalid("invalid note page request"));
        }
        let mut items: Vec<Note> = self
            .db
            .read(
                notes::LIST,
                vec![json!(deck_id), json!(cursor), json!(limit + 1)],
            )
            .await?
            .into_iter()
            .map(serde_json::from_value)
            .collect::<std::result::Result<_, _>>()?;
        let more = items.len() > limit;
        items.truncate(limit);
        let next_cursor = more
            .then(|| items.last().map(|note| note.id.clone()))
            .flatten();
        Ok(NotePage { items, next_cursor })
    }
}
