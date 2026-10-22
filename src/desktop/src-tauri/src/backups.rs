use crate::runtime::Error;
use alkemyst_core::db::Engine;
use serde::Serialize;
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;

#[derive(Clone, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum RestoreStatus {
    None,
    Restored { safety_backup: PathBuf },
    Failed { safety_backup: Option<PathBuf> },
}

pub(crate) struct Backups {
    db: Engine,
    directory: PathBuf,
    prepared: Mutex<Option<tempfile::TempDir>>,
    pub status: RestoreStatus,
}

impl Backups {
    pub fn new(db: Engine, database: PathBuf) -> Self {
        let directory = database
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."))
            .to_owned();
        Self {
            db,
            directory,
            prepared: Mutex::new(None),
            status: RestoreStatus::None,
        }
    }

    fn pending(&self) -> PathBuf {
        self.directory.join("pending-restore.sqlite3")
    }

    pub async fn prepare(&self, source: &Path) -> Result<(), Error> {
        let mut prepared = self.prepared.lock().await;
        let candidate = tempfile::Builder::new()
            .prefix("restore-selection-")
            .tempdir_in(&self.directory)?;
        self.db
            .stage_backup(source, &candidate.path().join("alkemyst.sqlite3"))
            .await?;
        // Replacing an unconfirmed selection drops its private temporary directory.
        *prepared = Some(candidate);
        Ok(())
    }

    pub async fn cancel(&self) {
        self.prepared.lock().await.take();
    }

    pub async fn confirm(&self) -> Result<(), Error> {
        let mut prepared = self.prepared.lock().await;
        let candidate = prepared.as_ref().ok_or(Error::NoRestore)?;
        // Same-volume hard linking atomically publishes without replacing an existing
        // request. Unconfirmed selections are never discovered or applied at startup.
        std::fs::hard_link(candidate.path().join("alkemyst.sqlite3"), self.pending())?;
        prepared.take();
        Ok(())
    }

    /// Called before Application construction: there are no running services or jobs.
    pub async fn recover(&mut self) -> Result<(), Error> {
        let pending = self.pending();
        if !pending.try_exists()? {
            return Ok(());
        }
        let safety = self.safety_backup().await.ok();
        let restored = if safety.is_some() {
            self.db.restore_from(&pending).await.is_ok()
        } else {
            false
        };
        if restored {
            // Consume the request before starting capture. If cleanup fails, do not
            // open services and risk reapplying it over newly captured history later.
            std::fs::remove_file(pending)?;
            self.status = RestoreStatus::Restored {
                safety_backup: safety.expect("restore requires a safety backup"),
            };
        } else {
            let failed = tempfile::Builder::new()
                .prefix("failed-restore-")
                .tempdir_in(&self.directory)?;
            std::fs::rename(pending, failed.path().join("alkemyst.sqlite3"))?;
            let _ = failed.keep();
            self.status = RestoreStatus::Failed {
                safety_backup: safety,
            };
        }
        Ok(())
    }

    async fn safety_backup(&self) -> Result<PathBuf, Error> {
        let directory = self.directory.join("backups");
        std::fs::create_dir_all(&directory)?;
        let recovery = tempfile::Builder::new()
            .prefix("before-restore-")
            .tempdir_in(directory)?;
        let destination = recovery.path().join("alkemyst.sqlite3");
        self.db.backup_to(&destination).await?;
        let _ = recovery.keep();
        Ok(destination)
    }
}
