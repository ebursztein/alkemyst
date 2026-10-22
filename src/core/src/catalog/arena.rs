use super::{Catalog, Error, Result};
use crate::{db::Write, statements::cards};
use serde_json::json;
use std::path::Path;

impl Catalog {
    /// Native-only ingestion of installed public card identity data. The source
    /// opens read-only on the SQL worker and its path is never persisted.
    pub async fn import_arena_database(&self, path: &Path) -> Result<usize> {
        let _refresh = self.refresh.try_lock().map_err(|_| Error::Busy)?;
        let aliases = self.db.read_source(path, cards::ARENA_SOURCE).await?;
        let writes = aliases
            .chunks(1024)
            .map(|chunk| {
                Ok(Write {
                    statement: cards::ALIASES,
                    params: vec![json!(serde_json::to_string(chunk)?)],
                    expected_rows: Some(chunk.len()),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        self.db.write(writes).await?;
        Ok(aliases.len())
    }
}
