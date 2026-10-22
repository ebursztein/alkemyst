use super::{Decks, Error, Origin, Result};
use crate::statements::decks;
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct VersionSummary {
    pub id: String,
    pub deck_id: String,
    pub ordinal: u32,
    pub parent_version: Option<String>,
    pub name: String,
    pub format: String,
    pub origin: Origin,
    pub created_at: i64,
    pub main_count: u32,
    pub sideboard_count: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct VersionPage {
    pub items: Vec<VersionSummary>,
    pub next_cursor: Option<String>,
}
impl Decks {
    pub async fn versions(
        &self,
        id: &str,
        limit: usize,
        cursor: Option<String>,
    ) -> Result<Option<VersionPage>> {
        if id.is_empty() || id.len() > 256 || !(1..=100).contains(&limit) {
            return Err(Error::Invalid("invalid version history request"));
        }
        let cursor = cursor
            .map(|c| {
                if c.len() != 19
                    || !c.starts_with("v1:")
                    || !c[3..].bytes().all(|b| b.is_ascii_hexdigit())
                {
                    return Err(Error::Invalid("invalid version cursor"));
                }
                i64::from_str_radix(&c[3..], 16)
                    .ok()
                    .filter(|n| *n > 0)
                    .ok_or(Error::Invalid("invalid version cursor"))
            })
            .transpose()?;
        self.db
            .read(
                decks::VERSIONS,
                vec![json!(id), json!(cursor), json!(limit)],
            )
            .await?
            .into_iter()
            .next()
            .map(serde_json::from_value)
            .transpose()
            .map_err(Into::into)
    }
}
