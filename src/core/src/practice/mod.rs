//! Manual local sandbox. It tracks physical cards and turns, not the full rules of Magic.
mod actions;
mod cards;
mod create;
mod models;
mod moves;
mod state;
mod store;
use crate::db::Engine;
pub use models::*;
pub const PRACTICE_VERSION: &str = "manual-sandbox-v1";
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] crate::db::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Analysis(#[from] crate::analysis::Error),
    #[error("invalid practice action: {0}")]
    Invalid(&'static str),
    #[error("idempotency conflict or stale practice revision")]
    Conflict,
    #[error("practice session or deck version not found")]
    NotFound,
    #[error("stored practice version is incompatible")]
    Incompatible,
}
#[derive(Clone)]
pub struct Practice {
    db: Engine,
}
impl Practice {
    pub fn new(db: Engine) -> Self {
        Self { db }
    }
}
