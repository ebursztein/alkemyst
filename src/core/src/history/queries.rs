use super::MatchMode;
use serde::Deserialize;
use utoipa::IntoParams;
#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in=Query)]
pub struct GameQuery {
    #[serde(default = "page_limit")]
    #[param(minimum = 1, maximum = 100, default = 40)]
    pub limit: usize,
    pub cursor: Option<String>,
    pub deck_id: Option<String>,
    pub version_id: Option<String>,
    pub match_id: Option<String>,
    pub mode: Option<MatchMode>,
}
impl Default for GameQuery {
    fn default() -> Self {
        Self {
            limit: 40,
            cursor: None,
            deck_id: None,
            version_id: None,
            match_id: None,
            mode: None,
        }
    }
}
#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in=Query)]
pub struct TimelineQuery {
    #[param(minimum = 1)]
    pub turn_number: Option<u32>,
    #[serde(default = "page_limit")]
    #[param(minimum = 1, maximum = 100, default = 40)]
    pub limit: usize,
    pub cursor: Option<String>,
}
impl Default for TimelineQuery {
    fn default() -> Self {
        Self {
            turn_number: None,
            limit: 40,
            cursor: None,
        }
    }
}
#[derive(Clone, Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in=Query)]
pub struct StatisticsQuery {
    pub deck_id: Option<String>,
    pub version_id: Option<String>,
    pub mode: Option<MatchMode>,
    #[serde(default)]
    pub complete_only: bool,
}
fn page_limit() -> usize {
    40
}

#[derive(Clone, Debug, Deserialize, IntoParams)]
#[into_params(parameter_in=Query)]
pub struct TurnQuery {
    #[serde(default = "page_limit")]
    #[param(minimum = 1, maximum = 100, default = 40)]
    pub limit: usize,
    pub cursor: Option<String>,
}
impl Default for TurnQuery {
    fn default() -> Self {
        Self {
            limit: 40,
            cursor: None,
        }
    }
}
