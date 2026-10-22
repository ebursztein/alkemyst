mod models;
mod queries;
use crate::{db::Engine, domain::Error, statements::history};
pub use models::*;
pub use queries::*;
use serde_json::json;
type Result<T> = std::result::Result<T, Error>;
#[derive(Clone)]
pub struct History {
    db: Engine,
}
impl History {
    pub fn new(db: Engine) -> Self {
        Self { db }
    }
    pub async fn games(&self, query: GameQuery) -> Result<GamePage> {
        limit(query.limit)?;
        filters(&[&query.deck_id, &query.version_id, &query.match_id])?;
        let cursor = decode(query.cursor.as_deref(), 1)?.first().copied();
        Ok(serde_json::from_value(
            self.db
                .read(
                    history::GAMES,
                    vec![
                        json!(cursor),
                        json!(query.deck_id),
                        json!(query.version_id),
                        json!(query.match_id),
                        json!(query.mode),
                        json!(query.limit),
                    ],
                )
                .await?
                .remove(0),
        )?)
    }
    pub async fn matches(&self, query: GameQuery) -> Result<MatchPage> {
        limit(query.limit)?;
        filters(&[&query.deck_id, &query.version_id, &query.match_id])?;
        let cursor = decode(query.cursor.as_deref(), 1)?.first().copied();
        Ok(serde_json::from_value(
            self.db
                .read(
                    history::MATCHES,
                    vec![
                        json!(cursor),
                        json!(query.deck_id),
                        json!(query.version_id),
                        json!(query.match_id),
                        json!(query.mode),
                        json!(query.limit),
                    ],
                )
                .await?
                .remove(0),
        )?)
    }
    pub async fn match_summary(&self, id: &str) -> Result<Option<MatchSummary>> {
        id_length(id)?;
        self.db
            .read(history::MATCH, vec![json!(id)])
            .await?
            .into_iter()
            .next()
            .map(serde_json::from_value)
            .transpose()
            .map_err(Into::into)
    }
    pub async fn game(&self, id: &str) -> Result<Option<GameSummary>> {
        id_length(id)?;
        self.db
            .read(history::GAME, vec![json!(id)])
            .await?
            .into_iter()
            .next()
            .map(serde_json::from_value)
            .transpose()
            .map_err(Into::into)
    }
    pub async fn timeline(&self, id: &str, query: TimelineQuery) -> Result<Option<TimelinePage>> {
        id_length(id)?;
        limit(query.limit)?;
        if query.turn_number == Some(0) {
            return Err(Error::Invalid("Turn number must be positive"));
        }
        let cursor = decode(query.cursor.as_deref(), 2)?;
        let value = self
            .db
            .read(
                history::TIMELINE,
                vec![
                    json!(id),
                    json!(cursor.get(1)),
                    json!(cursor.first()),
                    json!(query.limit),
                    json!(query.turn_number),
                ],
            )
            .await?
            .remove(0);
        if value["found"] != true {
            return Ok(None);
        }
        Ok(Some(serde_json::from_value(value)?))
    }
    pub async fn turns(&self, id: &str, query: TurnQuery) -> Result<Option<TurnPage>> {
        id_length(id)?;
        limit(query.limit)?;
        let cursor = decode_cursor(query.cursor.as_deref(), 2, "t1:", false)?;
        let value = self
            .db
            .read(
                history::TURNS,
                vec![
                    json!(id),
                    json!(cursor.get(1)),
                    json!(cursor.first()),
                    json!(query.limit),
                ],
            )
            .await?
            .remove(0);
        if value["found"] != true {
            return Ok(None);
        }
        Ok(Some(serde_json::from_value(value)?))
    }
    pub async fn statistics(&self, query: StatisticsQuery) -> Result<Statistics> {
        filters(&[&query.deck_id, &query.version_id])?;
        Ok(serde_json::from_value(
            self.db
                .read(
                    history::STATISTICS,
                    vec![
                        json!(query.deck_id),
                        json!(query.version_id),
                        json!(query.mode),
                        json!(query.complete_only),
                    ],
                )
                .await?
                .remove(0),
        )?)
    }
}
fn limit(limit: usize) -> Result<()> {
    if !(1..=100).contains(&limit) {
        return Err(Error::Invalid("Page limit must be 1..100"));
    }
    Ok(())
}
fn id_length(id: &str) -> Result<()> {
    if id.is_empty() || id.len() > 256 {
        return Err(Error::Invalid("Invalid history identifier"));
    }
    Ok(())
}
fn filters(values: &[&Option<String>]) -> Result<()> {
    for value in values.iter().filter_map(|v| v.as_deref()) {
        id_length(value)?;
    }
    Ok(())
}
fn decode(cursor: Option<&str>, parts: usize) -> Result<Vec<i64>> {
    decode_cursor(cursor, parts, "h1:", parts == 2)
}
fn decode_cursor(
    cursor: Option<&str>,
    parts: usize,
    prefix: &str,
    ordered: bool,
) -> Result<Vec<i64>> {
    let Some(cursor) = cursor else {
        return Ok(vec![]);
    };
    let invalid = || Error::Invalid("Invalid history cursor");
    let values = cursor
        .strip_prefix(prefix)
        .ok_or_else(invalid)?
        .split(':')
        .map(|part| {
            if part.len() != 16 {
                return Err(invalid());
            }
            i64::from_str_radix(part, 16)
                .map_err(|_| invalid())
                .and_then(|value| if value > 0 { Ok(value) } else { Err(invalid()) })
        })
        .collect::<Result<Vec<_>>>()?;
    if values.len() != parts {
        return Err(invalid());
    }
    if ordered && values[1] > values[0] {
        return Err(invalid());
    }
    Ok(values)
}
