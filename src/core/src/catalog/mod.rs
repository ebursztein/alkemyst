//! Local card metadata. No network is needed for reads after the first import.
mod arena;
mod download;
mod import;
mod models;
mod search;

use crate::{
    db::{Engine, Write},
    statements::cards,
};
pub use download::{BulkDescriptor, BulkProvider, Scryfall};
pub use models::{ArenaResolution, CardPage, CardSummary, CatalogSource, CatalogStatus, Search};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Database(#[from] crate::db::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Network(#[from] reqwest::Error),
    #[error("invalid catalog request: {0}")]
    Invalid(&'static str),
    #[error("a catalog refresh is already running")]
    Busy,
    #[error("catalog parser stopped unexpectedly")]
    ParserStopped,
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct Catalog {
    db: Engine,
    refresh: Arc<Mutex<()>>,
}

impl Catalog {
    pub fn new(db: Engine) -> Self {
        Self {
            db,
            refresh: Arc::new(Mutex::new(())),
        }
    }

    pub async fn status(&self) -> Result<CatalogStatus> {
        let rows = self.db.read(cards::STATUS, vec![]).await?;
        Ok(serde_json::from_value(
            rows.into_iter()
                .next()
                .ok_or(Error::Invalid("missing catalog state"))?,
        )?)
    }

    /// Full public Scryfall metadata, including unmodeled fields and card faces.
    pub async fn card(&self, oracle_id: &str) -> Result<Option<Value>> {
        Ok(self
            .db
            .read(cards::DETAIL, vec![json!(oracle_id)])
            .await?
            .into_iter()
            .next())
    }

    pub async fn resolve_arena(&self, ids: &[u32]) -> Result<Vec<ArenaResolution>> {
        if ids.len() > 1000 {
            return Err(Error::Invalid("at most 1000 Arena IDs per request"));
        }
        self.db
            .read(cards::RESOLVE, vec![json!(serde_json::to_string(ids)?)])
            .await?
            .into_iter()
            .map(|row| Ok(serde_json::from_value(row)?))
            .collect()
    }

    async fn mutate(
        &self,
        statement: crate::db::Mutation,
        params: Vec<Value>,
        expected_rows: Option<usize>,
    ) -> Result<()> {
        self.db
            .write(vec![Write {
                statement,
                params,
                expected_rows,
            }])
            .await?;
        Ok(())
    }
}
