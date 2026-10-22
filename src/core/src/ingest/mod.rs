mod decks;
mod models;
mod normalize;
mod privacy;
mod source;
mod watch;
mod wire;
use crate::{
    db::{Engine, Write},
    domain::Decks,
    statements::capture,
};
use models::Checkpoint;
pub use models::{CaptureStatus, PollReport};
use serde_json::json;
use std::{path::Path, sync::Arc};
pub use watch::{TrackingHealth, TrackingState, TrackingTask};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("capture storage unavailable: {0}")]
    Database(#[from] crate::db::Error),
    #[error("capture deck operation failed: {0}")]
    Domain(#[from] crate::domain::Error),
    #[error("capture input unavailable: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid capture data: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Invalid(&'static str),
    #[error("capture worker stopped")]
    Worker,
}
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Clone)]
pub struct Ingestor {
    db: Engine,
    decks: Decks,
    lock: Arc<tokio::sync::Mutex<()>>,
}
impl Ingestor {
    pub fn new(db: Engine) -> Self {
        Self {
            decks: Decks::new(db.clone()),
            db,
            lock: Arc::default(),
        }
    }
    pub async fn status(&self) -> Result<CaptureStatus> {
        Ok(serde_json::from_value(
            self.db.read(capture::STATUS, vec![]).await?.remove(0),
        )?)
    }
    pub async fn poll_file(&self, path: &Path) -> Result<PollReport> {
        self.scan(path, false).await
    }
    pub async fn import_file(&self, path: &Path) -> Result<PollReport> {
        let mut report = self.scan(path, true).await?;
        while !report.caught_up {
            let next = self.scan(path, true).await?;
            report.bytes_committed += next.bytes_committed;
            report.events_seen += next.events_seen;
            report.rotated |= next.rotated;
            report.caught_up = next.caught_up;
            report.awaiting_partial = next.awaiting_partial;
        }
        Ok(report)
    }
    async fn scan(&self, path: &Path, retained: bool) -> Result<PollReport> {
        let _guard = self.lock.lock().await;
        let path = path.to_path_buf();
        let key = manasight_parser::content_hash(path.as_os_str().as_encoded_bytes());
        let old: Checkpoint = self
            .db
            .read(capture::CHECKPOINT, vec![json!(key)])
            .await?
            .into_iter()
            .next()
            .map(serde_json::from_value)
            .transpose()?
            .unwrap_or_default();
        let old_sequence = old.sequence;
        let mut batch = tokio::task::spawn_blocking(move || source::read(&path, old, retained))
            .await
            .map_err(|_| Error::Worker)??;
        let mut events = vec![];
        for event in &batch.events {
            if let Some(deck) = self
                .submitted_deck(event, batch.checkpoint.context.deck.as_ref())
                .await?
            {
                batch.checkpoint.context.deck = Some(deck);
                batch.checkpoint.context.pending_deck = true;
            }
            if let Some(event) = normalize::event(event, &mut batch.checkpoint.context, &key) {
                events.push(event);
            }
        }
        let report = PollReport {
            bytes_committed: batch.checkpoint.offset.saturating_sub(batch.start_offset),
            events_seen: events.len(),
            rotated: batch.rotated,
            caught_up: batch.caught_up,
            awaiting_partial: batch.awaiting_partial,
        };
        if old_sequence > 0 && report.bytes_committed == 0 && !batch.rotated {
            return Ok(report);
        }
        let encoded = serde_json::to_string(&events)?;
        let mut writes = [capture::MATCHES, capture::GAMES, capture::EVENTS]
            .into_iter()
            .map(|statement| Write {
                statement,
                params: vec![json!(encoded)],
                expected_rows: None,
            })
            .collect::<Vec<_>>();
        writes.push(Write {
            statement: capture::SAVE_CHECKPOINT,
            params: vec![
                json!(key),
                json!(serde_json::to_string(&batch.checkpoint)?),
                json!(old_sequence),
            ],
            expected_rows: Some(1),
        });
        self.db.write(writes).await?;
        Ok(report)
    }
}
