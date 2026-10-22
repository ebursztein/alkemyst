use crate::domain::ObservedOutcome;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MatchMode {
    Bo1,
    Bo3,
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MissingData {
    InitialState,
    TerminalResult,
    StateGap,
    UnverifiableChain,
    Truncated,
    Connection,
    DeckVersion,
    LocalSeat,
    ConflictingResult,
    Outcome,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GameSummary {
    pub id: String,
    pub match_id: String,
    pub number: u32,
    pub deck_id: Option<String>,
    pub deck_version: Option<String>,
    pub deck_name: Option<String>,
    pub format: Option<String>,
    pub first_ordinal: i64,
    pub events: u64,
    pub outcome: Option<ObservedOutcome>,
    pub mode: MatchMode,
    pub capture_complete: bool,
    pub missing_data: Vec<MissingData>,
    pub local_seat: Option<i64>,
    pub on_play: Option<bool>,
    pub turns_observed: u64,
    pub last_turn: Option<u32>,
    pub mulligans: Option<u32>,
    pub mulligan_actions_observed: u64,
    /// Recorded land counters on observed local turns; not an inferred full-game total.
    pub land_drops_observed: Option<u64>,
    pub land_turn_samples: u64,
    /// Zone-less Arena log clock; not silently interpreted as UTC.
    pub local_time: Option<String>,
    /// Explicit match clock evidence; not a reconstructed game-start timestamp.
    pub match_instant_utc: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct GamePage {
    pub items: Vec<GameSummary>,
    pub next_cursor: Option<String>,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TimelineEvent {
    pub ordinal: i64,
    pub id: String,
    pub kind: String,
    pub state_id: Option<i64>,
    pub turn_number: Option<u32>,
    pub phase: Option<String>,
    pub step: Option<String>,
    pub local_seat: Option<i64>,
    pub local_time: Option<String>,
    pub instant_utc: Option<String>,
    pub data: BTreeMap<String, serde_json::Value>,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TimelinePage {
    pub game_id: String,
    pub items: Vec<TimelineEvent>,
    pub next_cursor: Option<String>,
    pub through_ordinal: i64,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SampleBreakdown {
    pub mode: MatchMode,
    pub games: u64,
    pub known_results: u64,
    pub wins: u64,
    pub losses: u64,
    pub draws: u64,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Statistics {
    pub games: u64,
    pub matches: u64,
    pub known_results: u64,
    pub wins: u64,
    pub losses: u64,
    pub draws: u64,
    pub unknown_results: u64,
    pub partial_games: u64,
    /// Wins / (wins + losses), excluding draws and unknown results.
    pub decisive_win_rate: Option<f64>,
    pub decisive_games: u64,
    pub on_play_samples: u64,
    pub on_draw_samples: u64,
    pub unknown_play_draw: u64,
    pub observed_mulligans_average: Option<f64>,
    pub mulligan_samples: u64,
    pub recorded_land_drops: u64,
    pub land_turn_samples: u64,
    pub by_mode: Vec<SampleBreakdown>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MatchSummary {
    pub id: String,
    pub first_ordinal: i64,
    pub mode: MatchMode,
    pub outcome: Option<ObservedOutcome>,
    pub completed: bool,
    pub game_count: u64,
    pub games: Vec<GameSummary>,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MatchPage {
    pub items: Vec<MatchSummary>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TurnSummary {
    pub number: u32,
    pub events: u64,
    pub states: u64,
    pub actions: u64,
    pub first_ordinal: i64,
    pub last_ordinal: i64,
    pub active_player: Option<i64>,
    pub is_local_turn: Option<bool>,
    pub phase_samples: u64,
    pub phases: Vec<String>,
    pub land_drops_observed: Option<u64>,
    pub land_counter_samples: u64,
    pub local_life_total: Option<i64>,
    pub mulligans: Option<u32>,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TurnPage {
    pub game_id: String,
    pub through_ordinal: i64,
    pub unassigned_events: u64,
    pub items: Vec<TurnSummary>,
    pub next_cursor: Option<String>,
}
