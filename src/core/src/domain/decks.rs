use super::{
    CreateDeck, DeckPage, DeckSummary, DeckVersion, Decks, Error, NewVersion, Origin, Result,
    validation,
};
use crate::{
    db::{Mutation, Write},
    statements::decks,
};
use serde_json::{Value, json};

impl Decks {
    pub async fn create(
        &self,
        key: &str,
        mut input: CreateDeck,
        origin: Origin,
    ) -> Result<DeckVersion> {
        validation::request_key(key)?;
        validation::deck(&input.name, &input.format, &mut input.cards)?;
        let body = json!({"origin":origin,"input":input}).to_string();
        self.new_deck(key, input, origin, "decks.create", body)
            .await
    }

    pub(super) async fn new_deck(
        &self,
        key: &str,
        input: CreateDeck,
        origin: Origin,
        operation: &str,
        body: String,
    ) -> Result<DeckVersion> {
        let deck_id = uuid::Uuid::new_v4().to_string();
        let version_id = uuid::Uuid::new_v4().to_string();
        let created = now()?;
        let prefix = vec![
            write(
                decks::RECEIPT,
                vec![json!(key), json!(operation), json!(body), json!(version_id)],
                Some(1),
            ),
            write(
                decks::CREATE,
                vec![
                    json!(deck_id),
                    json!(input.name),
                    json!(input.format),
                    json!(created),
                    json!(key),
                    json!(version_id),
                ],
                None,
            ),
        ];
        let version = NewVersion {
            deck_id,
            parent_version: None,
            name: input.name,
            format: input.format,
            cards: input.cards,
        };
        self.commit(key, version_id, version, origin, created, prefix)
            .await
    }

    pub async fn create_version(
        &self,
        key: &str,
        mut input: NewVersion,
        origin: Origin,
    ) -> Result<DeckVersion> {
        validation::request_key(key)?;
        validation::deck(&input.name, &input.format, &mut input.cards)?;
        let body = json!({"origin":origin,"input":input}).to_string();
        let id = uuid::Uuid::new_v4().to_string();
        let prefix = vec![write(
            decks::RECEIPT,
            vec![json!(key), json!("decks.version"), json!(body), json!(id)],
            Some(1),
        )];
        self.commit(key, id, input, origin, now()?, prefix).await
    }

    async fn commit(
        &self,
        key: &str,
        id: String,
        input: NewVersion,
        origin: Origin,
        created: i64,
        mut writes: Vec<Write>,
    ) -> Result<DeckVersion> {
        writes.extend([
            write(
                decks::VERSION,
                vec![
                    json!(id),
                    json!(input.deck_id),
                    json!(input.parent_version),
                    json!(input.name),
                    json!(input.format),
                    json!(origin),
                    json!(created),
                    json!(key),
                ],
                None,
            ),
            write(
                decks::CARDS,
                vec![
                    json!(id),
                    json!(serde_json::to_string(&input.cards)?),
                    json!(key),
                ],
                None,
            ),
            write(decks::SEAL, vec![json!(id)], None),
        ]);
        match self.db.write(writes).await {
            Err(crate::db::Error::RowCount { .. }) => return Err(Error::Conflict),
            result => result?,
        }
        self.get(vec![Value::Null, json!(key)])
            .await?
            .ok_or(Error::NotFound)
    }

    pub(super) async fn get(&self, params: Vec<Value>) -> Result<Option<DeckVersion>> {
        self.db
            .read(decks::GET, params)
            .await?
            .into_iter()
            .next()
            .map(serde_json::from_value)
            .transpose()
            .map_err(Into::into)
    }

    pub async fn version(&self, id: &str) -> Result<Option<DeckVersion>> {
        self.get(vec![json!(id), Value::Null]).await
    }

    pub async fn list(&self, limit: usize, cursor: Option<String>) -> Result<DeckPage> {
        if !(1..=100).contains(&limit) || cursor.as_ref().is_some_and(|c| c.len() > 128) {
            return Err(Error::Invalid("invalid deck page request"));
        }
        let mut items: Vec<DeckSummary> = self
            .db
            .read(decks::LIST, vec![json!(cursor), json!(limit + 1)])
            .await?
            .into_iter()
            .map(serde_json::from_value)
            .collect::<std::result::Result<_, _>>()?;
        let more = items.len() > limit;
        items.truncate(limit);
        let next_cursor = more
            .then(|| items.last().map(|item| item.id.clone()))
            .flatten();
        Ok(DeckPage { items, next_cursor })
    }
}

pub(super) fn write(
    statement: Mutation,
    params: Vec<Value>,
    expected_rows: Option<usize>,
) -> Write {
    Write {
        statement,
        params,
        expected_rows,
    }
}

pub(super) fn now() -> Result<i64> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| Error::Invalid("system clock predates Unix epoch"))?;
    i64::try_from(elapsed.as_secs()).map_err(|_| Error::Invalid("system clock out of range"))
}
