use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct CatalogSource {
    pub url: String,
    pub updated_at: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CatalogStatus {
    pub snapshot_id: Option<String>,
    pub source: Option<CatalogSource>,
    pub cards: u64,
    pub printings: u64,
    pub arena_ids: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct CardSummary {
    pub oracle_id: String,
    pub id: String,
    pub name: String,
    pub mana_value: Option<f64>,
    pub type_line: String,
    pub oracle_text: String,
    pub mana_cost: Option<String>,
    pub color_identity: Vec<String>,
    pub image_uri: Option<String>,
    pub arena_available: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CardPage {
    pub items: Vec<CardSummary>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ArenaResolution {
    pub arena_id: u32,
    /// The original printing metadata; null explicitly means unresolved.
    pub card: Option<Value>,
    pub resolved_by: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct Search {
    /// Plain text tokens across names, rules text and types. Not raw FTS syntax.
    #[serde(default)]
    pub text: String,
    pub type_line: Option<String>,
    pub colors: Option<String>,
    pub format: Option<String>,
    pub mana_min: Option<f64>,
    pub mana_max: Option<f64>,
    #[serde(default)]
    pub arena_only: bool,
    pub legendary: Option<bool>,
    #[serde(default = "default_limit")]
    pub limit: usize,
    pub cursor: Option<String>,
}

fn default_limit() -> usize {
    40
}

impl Default for Search {
    fn default() -> Self {
        Self {
            text: String::new(),
            type_line: None,
            colors: None,
            format: None,
            mana_min: None,
            mana_max: None,
            arena_only: false,
            legendary: None,
            limit: default_limit(),
            cursor: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(super) struct Cursor {
    pub name: String,
    pub oracle_id: String,
}
