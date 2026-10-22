use super::{Ingestor, Result, models::DeckLink};
use crate::{
    db::Write,
    domain::{CreateDeck, DeckCard, NewVersion, Origin, Zone},
    statements::capture,
};
use manasight_parser::{GameEvent, content_hash};
use serde_json::{Value, json};
use std::collections::BTreeMap;

impl Ingestor {
    pub(super) async fn submitted_deck(
        &self,
        event: &GameEvent,
        previous: Option<&DeckLink>,
    ) -> Result<Option<DeckLink>> {
        let payload = event.payload();
        let is_sideboard = payload["type"] == "submit_deck_resp";
        let is_connect = payload["type"] == "connect_resp";
        if !matches!(event, GameEvent::DeckSubmission(_)) && !is_sideboard && !is_connect {
            return Ok(None);
        }
        let mut cards = vec![];
        if is_sideboard || is_connect {
            for (field, zone) in [
                ("deck_cards", Zone::Main),
                ("sideboard_cards", Zone::Sideboard),
            ] {
                for id in payload[field]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_u64)
                {
                    cards.push(DeckCard {
                        reference: format!("arena:{id}"),
                        quantity: 1,
                        zone,
                    });
                }
            }
        } else {
            let raw = std::str::from_utf8(event.metadata().raw_bytes())
                .map_err(|_| super::Error::Invalid("Invalid deck submission"))?;
            let Some(start) = raw.find('{') else {
                return Ok(None);
            };
            let outer: Value = serde_json::from_str(&raw[start..])?;
            let request = match &outer["request"] {
                Value::String(text) => serde_json::from_str(text)?,
                value => value.clone(),
            };
            for (field, zone) in [
                ("MainDeck", Zone::Main),
                ("Sideboard", Zone::Sideboard),
                ("CommandZone", Zone::Command),
                ("Companions", Zone::Companion),
            ] {
                for card in request["Deck"][field].as_array().into_iter().flatten() {
                    if let (Some(id), Some(quantity)) =
                        (card["cardId"].as_u64(), card["quantity"].as_u64())
                    {
                        cards.push(DeckCard {
                            reference: format!("arena:{id}"),
                            quantity: u16::try_from(quantity)
                                .map_err(|_| super::Error::Invalid("Invalid card quantity"))?,
                            zone,
                        });
                    }
                }
            }
        }
        if cards.is_empty() {
            return Ok(None);
        }
        let mut counts = BTreeMap::<(Zone, String), u16>::new();
        for card in cards {
            let count = counts.entry((card.zone, card.reference)).or_default();
            *count = count
                .checked_add(card.quantity)
                .ok_or(super::Error::Invalid("Invalid card quantity"))?;
        }
        let cards: Vec<_> = counts
            .into_iter()
            .map(|((zone, reference), quantity)| DeckCard {
                zone,
                reference,
                quantity,
            })
            .collect();
        let format = payload["deck_format"]
            .as_str()
            .filter(|format| !format.is_empty())
            .map(str::to_lowercase)
            .unwrap_or_else(|| previous.map_or("unknown".into(), |deck| deck.format.clone()));
        let fingerprint = content_hash(
            serde_json::to_string(&json!({"cards":cards,"format":format}))?.as_bytes(),
        );
        let alias = if is_sideboard || is_connect {
            previous
                .map(|deck| deck.alias.clone())
                .unwrap_or_else(|| content_hash(format!("unidentified:{fingerprint}").as_bytes()))
        } else {
            payload["deck_id"]
                .as_str()
                .filter(|id| !id.is_empty())
                .map(|id| content_hash(id.as_bytes()))
                .unwrap_or_else(|| content_hash(format!("unidentified:{fingerprint}").as_bytes()))
        };
        let key = format!(
            "arena:{}",
            content_hash(format!("{alias}:{fingerprint}").as_bytes())
        );
        let found = self
            .db
            .read(capture::DECK, vec![json!(alias), json!(key)])
            .await?
            .remove(0);
        if let (Some(deck_id), Some(version_id)) =
            (found["deck_id"].as_str(), found["version_id"].as_str())
        {
            return Ok(Some(DeckLink {
                alias,
                deck_id: deck_id.into(),
                version_id: version_id.into(),
                format,
            }));
        }
        let name = payload["name"]
            .as_str()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or("Captured Arena deck")
            .to_owned();
        let version = if let Some(deck_id) = found["deck_id"].as_str() {
            self.decks
                .create_version(
                    &key,
                    NewVersion {
                        deck_id: deck_id.into(),
                        parent_version: None,
                        name,
                        format: format.clone(),
                        cards,
                    },
                    Origin::Arena,
                )
                .await?
        } else {
            self.decks
                .create(
                    &key,
                    CreateDeck {
                        name,
                        format: format.clone(),
                        cards,
                    },
                    Origin::Arena,
                )
                .await?
        };
        self.db
            .write(vec![Write {
                statement: capture::ALIAS,
                params: vec![json!(alias), json!(version.deck_id)],
                expected_rows: None,
            }])
            .await?;
        Ok(Some(DeckLink {
            alias,
            deck_id: version.deck_id,
            version_id: version.id,
            format,
        }))
    }
}
