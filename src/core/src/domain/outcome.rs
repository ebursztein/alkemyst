use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ObservedOutcome {
    Won,
    Lost,
    Draw,
}

impl ObservedOutcome {
    /// Missing winners or team mappings are not evidence of a loss.
    pub fn from_result(kind: Option<&str>, winner: Option<i64>, team: Option<i64>) -> Option<Self> {
        match (kind, winner.filter(|id| *id > 0), team.filter(|id| *id > 0)) {
            (Some("ResultType_Draw"), _, _) => Some(Self::Draw),
            (Some("ResultType_WinLoss"), Some(winner), Some(team)) => Some(if winner == team {
                Self::Won
            } else {
                Self::Lost
            }),
            _ => None,
        }
    }
}
