use crate::{
    analysis::{DeckSnapshot, compiler::Profile, simulation::SimulationReport, turns::Goal},
    domain::Origin,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ModelScope {
    PrecombatGoldfish,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct SimulationParameters {
    pub version_id: String,
    #[schema(minimum = 1, maximum = 1000000)]
    pub runs: u32,
    #[schema(minimum = 1, maximum = 30)]
    pub turns: u8,
    pub on_the_play: bool,
    /// Full-width hexadecimal seed. Omit to generate and persist a random seed.
    #[schema(pattern = "^[0-9a-fA-F]{16}$")]
    pub seed: Option<String>,
    pub scope: ModelScope,
    pub goal: Goal,
    pub mulligan: MulliganParameters,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct MulliganParameters {
    pub minimum_keep: u8,
    pub force_mulligans: u8,
    pub min_lands: u8,
    pub max_lands: u8,
    pub required_references: Vec<String>,
}
impl Default for MulliganParameters {
    fn default() -> Self {
        Self {
            minimum_keep: 4,
            force_mulligans: 0,
            min_lands: 2,
            max_lands: 5,
            required_references: vec![],
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Cancelling,
    Completed,
    Cancelled,
    Failed,
    Interrupted,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct SimulationJob {
    pub id: String,
    pub version_id: String,
    pub status: JobStatus,
    pub origin: Origin,
    pub created_at: i64,
    pub updated_at: i64,
    pub requested_runs: u32,
    pub completed_runs: u32,
    pub seed: String,
    pub compiler_version: String,
    pub model_version: String,
    pub policy_version: String,
    pub replay_version: String,
    pub catalog_snapshot: Option<String>,
    pub parameters: SimulationParameters,
    pub report: Option<SimulationReport>,
    pub error_code: Option<String>,
    pub cancellation_origin: Option<Origin>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct SimulationInputs {
    pub snapshot: DeckSnapshot,
    pub profile: Profile,
    pub parameters: SimulationParameters,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct JobPage {
    pub items: Vec<SimulationJob>,
    pub next_cursor: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, IntoParams)]
#[into_params(parameter_in=Query)]
pub struct JobQuery {
    #[serde(default = "limit")]
    #[param(minimum = 1, maximum = 100, default = 40)]
    pub limit: usize,
    pub cursor: Option<String>,
    pub version_id: Option<String>,
}
fn limit() -> usize {
    40
}

impl Default for JobQuery {
    fn default() -> Self {
        Self {
            limit: 40,
            cursor: None,
            version_id: None,
        }
    }
}
