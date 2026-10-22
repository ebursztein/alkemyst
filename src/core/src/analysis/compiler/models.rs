use crate::{
    analysis::{Counts, CurveBin},
    domain::Zone,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum IssueCode {
    MissingCard,
    InvalidMetadata,
    UnsupportedCost,
    UnknownRules,
    RulesChanged,
    UnsupportedLayout,
    CombatExcluded,
    RecursionExcluded,
    ActivatedAbilityExcluded,
    IgnoredCard,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct Issue {
    pub code: IssueCode,
    pub blocking: bool,
    pub detail: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CardProfile {
    pub reference: String,
    pub quantity: u16,
    pub zone: Zone,
    pub oracle_id: Option<String>,
    pub name: Option<String>,
    pub cast_cost: Option<String>,
    pub adventure_cost: Option<String>,
    pub equip_cost: Option<String>,
    pub issues: Vec<Issue>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct Profile {
    pub version_id: String,
    pub deck_id: String,
    pub name: String,
    pub format: String,
    pub catalog_snapshot: Option<String>,
    pub catalog_updated_at: Option<String>,
    pub compiler_version: String,
    pub model_version: String,
    pub policy_version: String,
    pub sources: Vec<SourceCount>,
    pub counts: Counts,
    pub curve: Vec<CurveBin>,
    pub cards: Vec<CardProfile>,
    pub can_simulate: bool,
    pub assumptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct SourceCount {
    pub color: String,
    /// Color always available once untapped, with no spending restriction.
    pub unrestricted_lands: u32,
    /// Potential land color depends on board state or spending restriction.
    pub conditional_lands: u32,
    /// Includes summoning-sick or restricted creature/artifact mana abilities.
    pub nonland_sources: u32,
}
