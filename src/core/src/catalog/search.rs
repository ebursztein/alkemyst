use super::{CardPage, CardSummary, Catalog, Error, Result, Search, models::Cursor};
use crate::statements::cards;
use serde_json::json;

impl Catalog {
    pub async fn search(&self, request: Search) -> Result<CardPage> {
        if !(1..=100).contains(&request.limit) || request.text.len() > 256 {
            return Err(Error::Invalid(
                "search limit must be 1..100; text at most 256 bytes",
            ));
        }
        for value in [request.mana_min, request.mana_max].into_iter().flatten() {
            if !value.is_finite() || value < 0.0 {
                return Err(Error::Invalid("invalid mana bound"));
            }
        }
        if let (Some(min), Some(max)) = (request.mana_min, request.mana_max)
            && min > max
        {
            return Err(Error::Invalid("inverted mana bounds"));
        }
        if let Some(format) = &request.format
            && (format.is_empty()
                || format.len() > 32
                || !format.chars().all(|c| c.is_ascii_lowercase()))
        {
            return Err(Error::Invalid("invalid format name"));
        }
        if request
            .type_line
            .as_ref()
            .is_some_and(|text| text.len() > 128)
        {
            return Err(Error::Invalid("type filter too long"));
        }
        let colors = request
            .colors
            .as_ref()
            .map(|colors| {
                colors.chars().try_fold(0_i64, |mask, color| {
                    "WUBRG"
                        .find(color.to_ascii_uppercase())
                        .map(|bit| mask | (1 << bit))
                        .ok_or(Error::Invalid("colors must contain only WUBRG"))
                })
            })
            .transpose()?;
        let cursor: Option<Cursor> = request
            .cursor
            .as_ref()
            .map(|encoded| {
                if encoded.len() > 2048 {
                    return Err(Error::Invalid("cursor too long"));
                }
                serde_json::from_str(encoded).map_err(|_| Error::Invalid("invalid card cursor"))
            })
            .transpose()?;
        let words = request
            .text
            .split(|c: char| !c.is_alphanumeric())
            .filter(|word| !word.is_empty())
            .map(|word| format!("\"{word}\"*"))
            .collect::<Vec<_>>();
        let text = (!words.is_empty()).then(|| words.join(" AND "));
        let params = vec![
            json!(text),
            json!(request.type_line),
            json!(colors),
            json!(request.format),
            json!(request.mana_min),
            json!(request.mana_max),
            json!(request.arena_only),
            json!(request.legendary),
            json!(cursor.as_ref().map(|c| &c.name)),
            json!(cursor.as_ref().map(|c| &c.oracle_id)),
            json!(request.limit + 1),
        ];
        let mut items: Vec<CardSummary> = self
            .db
            .read(cards::SEARCH, params)
            .await?
            .into_iter()
            .map(serde_json::from_value)
            .collect::<std::result::Result<_, _>>()?;
        let has_more = items.len() > request.limit;
        items.truncate(request.limit);
        let next_cursor = if has_more {
            items
                .last()
                .map(|card| {
                    serde_json::to_string(&Cursor {
                        name: card.name.clone(),
                        oracle_id: card.oracle_id.clone(),
                    })
                })
                .transpose()?
        } else {
            None
        };
        Ok(CardPage { items, next_cursor })
    }
}
