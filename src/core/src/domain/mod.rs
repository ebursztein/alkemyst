//! Transport-independent application operations. Deck edits always append versions.
mod arena_text;
mod decks;
mod models;
mod notes;
mod outcome;
pub use outcome::ObservedOutcome;
mod text_operations;
mod validation;

use crate::db::Engine;
pub use models::{
    CreateDeck, DeckCard, DeckPage, DeckSummary, DeckVersion, ImportDeck, NewNote, NewVersion,
    Note, NotePage, Origin, Zone,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] crate::db::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("invalid request: {0}")]
    Invalid(&'static str),
    #[error("idempotency key already used for a different request")]
    Conflict,
    #[error("unresolved cards: {0:?}")]
    UnknownCards(Vec<String>),
    #[error("requested object does not exist")]
    NotFound,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct Decks {
    db: Engine,
}

impl Decks {
    pub fn new(db: Engine) -> Self {
        Self { db }
    }
}
mod versions;
pub use versions::{VersionPage, VersionSummary};
