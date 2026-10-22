use super::{
    CreateDeck, DeckCard, DeckVersion, Decks, Error, ImportDeck, Origin, Result, arena_text,
    validation,
};
use crate::statements::decks;
use serde_json::{Value, json};

impl Decks {
    pub async fn import_arena(
        &self,
        key: &str,
        input: ImportDeck,
        origin: Origin,
    ) -> Result<DeckVersion> {
        validation::request_key(key)?;
        let body = json!({"origin":origin,"input":input}).to_string();
        let receipts = self.db.read(decks::RECEIPT_GET, vec![json!(key)]).await?;
        if let Some(receipt) = receipts.first() {
            if receipt["operation"] != "decks.importArena" || receipt["body"] != body {
                return Err(Error::Conflict);
            }
            return self
                .get(vec![Value::Null, json!(key)])
                .await?
                .ok_or(Error::NotFound);
        }
        let lines = arena_text::parse(&input.text)?;
        let resolutions = self
            .db
            .read(
                decks::RESOLVE_NAMES,
                vec![json!(serde_json::to_string(&lines)?)],
            )
            .await?;
        let missing = lines
            .iter()
            .zip(&resolutions)
            .filter(|(_, r)| r["reference"].is_null())
            .map(|(line, _)| line.name.clone())
            .collect::<Vec<_>>();
        if !missing.is_empty() {
            return Err(Error::UnknownCards(missing));
        }
        let cards = lines
            .into_iter()
            .zip(resolutions)
            .map(|(line, r)| {
                Ok(DeckCard {
                    reference: r["reference"]
                        .as_str()
                        .ok_or(Error::Invalid("missing card reference"))?
                        .into(),
                    quantity: line.quantity,
                    zone: line.zone,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let mut resolved = CreateDeck {
            name: input.name,
            format: input.format,
            cards,
        };
        validation::deck(&resolved.name, &resolved.format, &mut resolved.cards)?;
        self.new_deck(key, resolved, origin, "decks.importArena", body)
            .await
    }

    pub async fn export_arena(&self, id: &str) -> Result<String> {
        let rows = self.db.read(decks::EXPORT, vec![json!(id)]).await?;
        let row = rows.into_iter().next().ok_or(Error::NotFound)?;
        arena_text::export(serde_json::from_value(row["cards"].clone())?)
    }
}
