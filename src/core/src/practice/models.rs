use crate::{analysis::DeckSnapshot, domain::Origin};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct NewPractice {
    pub version_id: String,
    pub on_the_play: bool,
    #[schema(pattern = "^[0-9a-fA-F]{16}$")]
    pub seed: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PracticePhase {
    Opening,
    Main,
    Finished,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PracticeZone {
    Hand,
    Battlefield,
    Graveyard,
    Exile,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum PracticeAction {
    Mulligan,
    Keep {
        bottom: Vec<u16>,
    },
    Draw,
    NextTurn,
    PlayLand {
        card: u16,
        face: u8,
    },
    /// Manual play: mana payments and card abilities are not enforced.
    Play {
        card: u16,
        face: u8,
        destination: PracticeZone,
    },
    /// Explicit sandbox override for visible cards only; never reveals the library.
    Move {
        card: u16,
        destination: PracticeZone,
    },
    Tap {
        card: u16,
        tapped: bool,
    },
    Finish,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PracticeFace {
    pub name: Option<String>,
    pub mana_cost: Option<String>,
    pub type_line: Option<String>,
    pub oracle_text: Option<String>,
    pub image_uri: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PracticeCard {
    /// Stable physical-copy identity within this session; duplicates have distinct ids.
    pub id: u16,
    pub reference: String,
    pub name: Option<String>,
    pub mana_cost: Option<String>,
    pub type_line: Option<String>,
    pub oracle_text: Option<String>,
    pub image_uri: Option<String>,
    pub faces: Vec<PracticeFace>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PracticePermanent {
    pub card: PracticeCard,
    pub face: u8,
    pub tapped: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PracticeView {
    pub id: String,
    pub version_id: String,
    pub revision: u32,
    pub origin: Origin,
    pub last_action_origin: Origin,
    pub created_at: i64,
    pub updated_at: i64,
    pub seed: String,
    pub replay_version: String,
    pub practice_version: String,
    pub rules_mode: String,
    pub on_the_play: bool,
    pub phase: PracticePhase,
    pub mulligans: u8,
    pub bottom_count: u8,
    pub turn: u8,
    pub library_count: u16,
    pub land_played: bool,
    pub decked: bool,
    pub hand: Vec<PracticeCard>,
    pub battlefield: Vec<PracticePermanent>,
    pub graveyard: Vec<PracticeCard>,
    pub exile: Vec<PracticeCard>,
    pub warnings: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PracticeSummary {
    pub id: String,
    pub version_id: String,
    pub name: String,
    pub revision: u32,
    pub created_at: i64,
    pub phase: PracticePhase,
    pub turn: u8,
    pub mulligans: u8,
    pub decked: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PracticePage {
    pub items: Vec<PracticeSummary>,
    pub next_cursor: Option<String>,
}
#[derive(Debug, Clone, Deserialize, IntoParams)]
#[into_params(parameter_in=Query)]
pub struct PracticeQuery {
    #[serde(default = "limit")]
    #[param(minimum = 1, maximum = 100, default = 40)]
    pub limit: usize,
    pub cursor: Option<String>,
    pub version_id: Option<String>,
}
fn limit() -> usize {
    40
}
impl Default for PracticeQuery {
    fn default() -> Self {
        Self {
            limit: 40,
            cursor: None,
            version_id: None,
        }
    }
}
#[derive(Deserialize)]
pub(super) struct Stored {
    pub id: String,
    pub version_id: String,
    pub origin: Origin,
    pub last_action_origin: Origin,
    pub created_at: i64,
    pub updated_at: i64,
    pub seed: String,
    pub replay_version: String,
    pub practice_version: String,
    pub on_the_play: bool,
    pub snapshot: DeckSnapshot,
    pub revision: u32,
    pub state: super::state::State,
}
