use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Zone {
    Main,
    Sideboard,
    Command,
    Companion,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    User,
    Agent,
    Arena,
    Import,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DeckCard {
    /// oracle:<id>, scryfall:<printing-id>, or arena:<numeric-id>.
    pub reference: String,
    pub quantity: u16,
    pub zone: Zone,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateDeck {
    pub name: String,
    pub format: String,
    pub cards: Vec<DeckCard>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct NewVersion {
    pub deck_id: String,
    pub parent_version: Option<String>,
    pub name: String,
    pub format: String,
    pub cards: Vec<DeckCard>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct DeckVersion {
    pub id: String,
    pub deck_id: String,
    pub ordinal: u32,
    pub parent_version: Option<String>,
    pub name: String,
    pub format: String,
    pub origin: Origin,
    pub created_at: i64,
    pub cards: Vec<DeckCard>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeckSummary {
    pub id: String,
    pub name: String,
    pub format: String,
    pub latest_version: String,
    pub version_count: u32,
    pub main_count: u32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeckPage {
    pub items: Vec<DeckSummary>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct NewNote {
    pub deck_id: String,
    pub version_id: Option<String>,
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct Note {
    pub id: String,
    pub deck_id: String,
    pub version_id: Option<String>,
    pub body: String,
    pub origin: Origin,
    pub created_at: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct NotePage {
    pub items: Vec<Note>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct ImportDeck {
    pub name: String,
    pub format: String,
    pub text: String,
}
