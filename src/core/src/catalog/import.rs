use super::{Catalog, CatalogSource, CatalogStatus, Error, Result};
use crate::{db::Write, statements::cards};
use serde_json::{Value, json};
use std::io::{BufRead, Read};
use tokio::sync::mpsc;

const MAX_RECORD: u64 = 1024 * 1024;

impl Catalog {
    /// Parse on a background thread and send bounded batches to the SQL worker.
    pub async fn import_jsonl<R: BufRead + Send + 'static>(
        &self,
        reader: R,
        source: CatalogSource,
    ) -> Result<CatalogStatus> {
        let _refresh = self.refresh.try_lock().map_err(|_| Error::Busy)?;
        self.import_inner(reader, source).await
    }

    pub(super) async fn import_inner<R: BufRead + Send + 'static>(
        &self,
        reader: R,
        source: CatalogSource,
    ) -> Result<CatalogStatus> {
        if source.url.is_empty() || source.updated_at.is_empty() {
            return Err(Error::Invalid("catalog provenance required"));
        }
        let id = uuid::Uuid::new_v4().to_string();
        self.mutate(cards::CLEAN, vec![json!(id)], None).await?;
        self.mutate(
            cards::BEGIN,
            vec![json!(id), json!(source.url), json!(source.updated_at)],
            Some(1),
        )
        .await?;
        let result = self.consume(reader, &id).await;
        if result.is_err() {
            // A failed/cancelled import never changes the active pointer. Leftover staging
            // rows after process interruption are collected by the next import.
            let _ = self.mutate(cards::DISCARD, vec![json!(id)], None).await;
        }
        result?;
        self.mutate(cards::CLEAN, vec![json!(id)], None).await?;
        self.status().await
    }

    async fn consume<R: BufRead + Send + 'static>(&self, reader: R, id: &str) -> Result<()> {
        let (sender, mut receiver) = mpsc::channel(2);
        let producer = std::thread::Builder::new()
            .name("alkemyst-cards".into())
            .spawn(move || {
                if let Err(error) = parse(reader, &sender) {
                    let _ = sender.blocking_send(Err(error));
                }
            })?;
        while let Some(batch) = receiver.recv().await {
            let batch = batch?;
            self.mutate(
                cards::STAGE,
                vec![json!(id), json!(serde_json::to_string(&batch)?)],
                Some(batch.len()),
            )
            .await?;
        }
        if producer.join().is_err() {
            return Err(Error::ParserStopped);
        }
        self.db
            .write(vec![
                Write {
                    statement: cards::BUILD,
                    params: vec![json!(id)],
                    expected_rows: None,
                },
                Write {
                    statement: cards::RETIRE,
                    params: vec![],
                    expected_rows: None,
                },
                Write {
                    statement: cards::ACTIVATE,
                    params: vec![json!(id)],
                    expected_rows: Some(1),
                },
                Write {
                    statement: cards::PUBLISH,
                    params: vec![json!(id)],
                    expected_rows: Some(1),
                },
            ])
            .await?;
        Ok(())
    }
}

fn parse<R: BufRead>(mut reader: R, sender: &mpsc::Sender<Result<Vec<Value>>>) -> Result<()> {
    let mut batch = Vec::new();
    let mut bytes = 0;
    loop {
        let mut line = String::new();
        let read = reader.by_ref().take(MAX_RECORD + 1).read_line(&mut line)?;
        if read == 0 {
            break;
        }
        if read as u64 > MAX_RECORD {
            return Err(Error::Invalid("card record exceeds 1 MiB"));
        }
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(&line)?;
        if value["object"] != "card"
            || value["id"].as_str().is_none_or(str::is_empty)
            || value["name"].as_str().is_none_or(str::is_empty)
        {
            return Err(Error::Invalid("expected Scryfall card record"));
        }
        if let Some(mana) = value.get("cmc")
            && !mana.is_null()
            && !mana.is_number()
        {
            return Err(Error::Invalid("mana value must be numeric or absent"));
        }
        bytes += read;
        batch.push(value);
        if batch.len() >= 128 || bytes >= 2 * 1024 * 1024 {
            sender
                .blocking_send(Ok(std::mem::take(&mut batch)))
                .map_err(|_| Error::ParserStopped)?;
            bytes = 0;
        }
    }
    if !batch.is_empty() {
        sender
            .blocking_send(Ok(batch))
            .map_err(|_| Error::ParserStopped)?;
    }
    Ok(())
}
