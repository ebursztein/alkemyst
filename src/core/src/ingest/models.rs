use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;

#[derive(Clone, Default, Serialize, Deserialize)]
pub(super) struct Context {
    pub match_id: Option<String>,
    pub game_number: Option<u32>,
    pub deck: Option<DeckLink>,
    #[serde(default)]
    pub pending_deck: bool,
    pub local_seat: Option<i64>,
    pub teams: BTreeMap<i64, i64>,
}
#[derive(Clone, Serialize, Deserialize)]
pub(super) struct DeckLink {
    pub alias: String,
    pub deck_id: String,
    pub version_id: String,
    pub format: String,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub(super) struct Checkpoint {
    pub sequence: i64,
    pub offset: u64,
    pub file_id: String,
    pub prefix_len: usize,
    pub prefix_hash: String,
    pub anchor_hash: String,
    pub context: Context,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PollReport {
    pub bytes_committed: u64,
    pub events_seen: usize,
    pub rotated: bool,
    pub caught_up: bool,
    pub awaiting_partial: bool,
}
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CaptureStatus {
    pub sources: u64,
    pub matches: u64,
    pub games: u64,
    pub events: u64,
    pub deck_versions: u64,
    /// Games without one unambiguous observed local outcome, including unfinished games.
    pub unknown_results: u64,
}
#[derive(Serialize)]
pub(super) struct Event {
    pub id: String,
    pub match_id: Option<String>,
    pub game_id: Option<String>,
    pub game_number: Option<u32>,
    pub deck_version: Option<String>,
    pub kind: &'static str,
    pub state_id: Option<i64>,
    pub payload: serde_json::Value,
    pub local_time: Option<String>,
    pub instant_utc: Option<String>,
}
