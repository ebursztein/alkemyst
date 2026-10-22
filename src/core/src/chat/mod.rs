//! Local conversation storage. Creating or appending history never contacts a provider.
mod models;
mod reads;
mod validation;
mod writes;
use crate::db::Engine;
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] crate::db::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("invalid request: {0}")]
    Invalid(&'static str),
    #[error("idempotency key already used for different conversation content")]
    Conflict,
    #[error("conversation revision changed")]
    StaleRevision,
    #[error("conversation not found")]
    NotFound,
}
pub use models::*;
type Result<T> = std::result::Result<T, Error>;
#[derive(Clone)]
pub struct Chat {
    db: Engine,
}
impl Chat {
    pub fn new(db: Engine) -> Self {
        Self { db }
    }
}
