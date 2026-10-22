use super::{Analysis, Error};
use crate::{domain::Zone, statements::comparison};
use serde::{Deserialize, Serialize};
use serde_json::json;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ComparisonSide {
    pub id: String,
    pub deck_id: String,
    pub ordinal: u32,
    pub name: String,
    pub format: String,
    pub main: u32,
    pub sideboard: u32,
    pub unresolved_main: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct CardDelta {
    /// Oracle identity when resolved; original reference otherwise.
    pub identity: String,
    pub name: Option<String>,
    pub zone: Zone,
    pub base_quantity: u32,
    pub candidate_quantity: u32,
    pub delta: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct CurveDelta {
    /// Printed mana value; null retains unresolved main-deck cards.
    pub mana_value: Option<f64>,
    pub base_quantity: u32,
    pub candidate_quantity: u32,
    pub delta: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct DeckComparison {
    pub base: ComparisonSide,
    pub candidate: ComparisonSide,
    pub catalog_snapshot: Option<String>,
    pub catalog_updated_at: Option<String>,
    /// Changed quantities by identity and zone. Printing-only changes cancel out.
    pub cards: Vec<CardDelta>,
    /// Printed front-face nonland curve, not effective casting or equip costs.
    pub curve: Vec<CurveDelta>,
}
impl Analysis {
    /// One SQL statement resolves both versions against the same catalog snapshot,
    /// aggregates all deltas and orders them. No simulations or win rates are inferred.
    pub async fn compare(
        &self,
        base: &str,
        candidate: &str,
    ) -> Result<Option<DeckComparison>, Error> {
        for id in [base, candidate] {
            if id.is_empty() || id.len() > 256 {
                return Err(Error::Invalid(
                    "version identifier must contain 1..=256 bytes",
                ));
            }
        }
        self.db
            .read(comparison::COMPARE, vec![json!(base), json!(candidate)])
            .await?
            .into_iter()
            .next()
            .map(serde_json::from_value)
            .transpose()
            .map_err(Error::from)
    }
}
