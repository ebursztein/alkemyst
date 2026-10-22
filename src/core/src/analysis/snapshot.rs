use crate::{db::Engine, domain::Zone, statements::analysis};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use utoipa::ToSchema;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] crate::db::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("invalid analysis request: {0}")]
    Invalid(&'static str),
    #[error("idempotency key was already used for a different request")]
    Conflict,
    #[error("simulation queue is full")]
    QueueFull,
    #[error("requested simulation or version does not exist")]
    NotFound,
    #[error("deck has missing or unsupported model inputs")]
    Unsupported,
    #[error("simulation worker is stopping")]
    Stopped,
    #[error("stored simulation state is inconsistent")]
    Stored,
    #[error(transparent)]
    Model(#[from] landlord::Error),
}
#[derive(Clone)]
pub struct Analysis {
    pub(super) db: Engine,
    pub(super) coordinator: std::sync::Arc<super::jobs::Coordinator>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct DeckSnapshot {
    pub version_id: String,
    pub deck_id: String,
    pub name: String,
    pub format: String,
    pub catalog_snapshot: Option<String>,
    pub catalog_updated_at: Option<String>,
    pub counts: Counts,
    pub curve: Vec<CurveBin>,
    pub cards: Vec<ResolvedCard>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct Counts {
    pub main: u32,
    pub sideboard: u32,
    pub lands: u32,
    pub equipment: u32,
    pub creatures: u32,
    pub legendary: u32,
    pub unresolved_main: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct CurveBin {
    pub mana_value: Option<f64>,
    pub quantity: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, ToSchema)]
pub struct ResolvedCard {
    pub reference: String,
    pub quantity: u16,
    pub zone: Zone,
    pub data: Option<Value>,
}
impl Analysis {
    pub fn new(db: Engine) -> Self {
        Self {
            db,
            coordinator: std::sync::Arc::new(super::jobs::Coordinator::new()),
        }
    }
    /// Exactly one prepared statement performs resolution and relational aggregation.
    pub async fn snapshot(&self, version_id: &str) -> Result<Option<DeckSnapshot>, Error> {
        if version_id.is_empty() || version_id.len() > 256 {
            return Err(Error::Invalid(
                "version identifier must contain 1..=256 bytes",
            ));
        }
        self.db
            .read(analysis::SNAPSHOT, vec![json!(version_id)])
            .await?
            .into_iter()
            .next()
            .map(serde_json::from_value)
            .transpose()
            .map_err(Error::from)
    }
}

impl Analysis {
    pub async fn prepare(
        &self,
        version_id: &str,
    ) -> Result<Option<super::compiler::CompiledDeck>, Error> {
        Ok(self
            .snapshot(version_id)
            .await?
            .map(super::compiler::compile))
    }
    pub async fn profile(
        &self,
        version_id: &str,
    ) -> Result<Option<super::compiler::Profile>, Error> {
        Ok(self
            .prepare(version_id)
            .await?
            .map(|compiled| compiled.profile))
    }
}
