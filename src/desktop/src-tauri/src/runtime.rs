use alkemyst_core::{
    Application,
    api::{AuthConfig, RunningServer},
    db::Engine,
    ingest::{TrackingHealth, TrackingState, TrackingTask},
};
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
use tokio::sync::Mutex;
#[derive(Clone)]
pub enum Tracking {
    Discover,
    Files { previous: PathBuf, current: PathBuf },
    Disabled,
}
pub struct RuntimeOptions {
    pub database: PathBuf,
    pub tracking: Tracking,
    pub port: u16,
    pub development: bool,
}
#[derive(Clone, Serialize)]
pub struct LocalConnection {
    pub base_url: String,
    pub token: String,
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Storage(#[from] alkemyst_core::db::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Tracking(#[from] alkemyst_core::ingest::Error),
    #[error("local services are stopping")]
    Stopped,
    #[error("no backup has been selected for restoration")]
    NoRestore,
}
pub struct NativeTrackingStatus {
    pub paused: bool,
    pub health: TrackingHealth,
}
struct Services {
    server: Option<RunningServer>,
    tracking: Option<TrackingTask>,
    paused: bool,
    stopped: bool,
}
pub struct NativeRuntime {
    app: Application,
    db: Engine,
    connection: LocalConnection,
    source: Tracking,
    stopping: AtomicBool,
    services: Mutex<Services>,
    backups: crate::backups::Backups,
}
fn start_tracking(app: &Application, source: &Tracking) -> Option<TrackingTask> {
    match source {
        Tracking::Discover => app.ingest.discover_and_watch().ok(),
        Tracking::Files { previous, current } => {
            Some(app.ingest.watch(previous.clone(), current.clone()))
        }
        Tracking::Disabled => None,
    }
}
impl NativeRuntime {
    pub async fn start(options: RuntimeOptions) -> Result<Self, Error> {
        if let Some(parent) = options
            .database
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
        {
            std::fs::create_dir_all(parent)?;
        }
        let db = Engine::open(&options.database, 8 * 1024 * 1024)?;
        let mut backups = crate::backups::Backups::new(db.clone(), options.database);
        backups.recover().await?;
        let app = Application::new(db.clone());
        let generated = AuthConfig::generate();
        let auth = AuthConfig::new(generated.credential().to_owned(), options.development)?;
        let server = match RunningServer::start(
            app.clone(),
            auth.clone(),
            ([127, 0, 0, 1], options.port).into(),
        )
        .await
        {
            Ok(server) => server,
            Err(error) if error.kind() == std::io::ErrorKind::AddrInUse && options.port != 0 => {
                RunningServer::start(app.clone(), auth.clone(), ([127, 0, 0, 1], 0).into()).await?
            }
            Err(error) => return Err(error.into()),
        };
        let connection = LocalConnection {
            base_url: format!("http://{}", server.address()),
            token: auth.credential().to_owned(),
        };
        let tracking = start_tracking(&app, &options.tracking);
        Ok(Self {
            app,
            db,
            connection,
            source: options.tracking,
            backups,
            stopping: AtomicBool::new(false),
            services: Mutex::new(Services {
                server: Some(server),
                tracking,
                paused: false,
                stopped: false,
            }),
        })
    }
    pub fn bootstrap(&self) -> Result<LocalConnection, Error> {
        if self.stopping.load(Ordering::Acquire) {
            return Err(Error::Stopped);
        }
        Ok(self.connection.clone())
    }

    pub fn restore_status(&self) -> crate::RestoreStatus {
        self.backups.status.clone()
    }
    pub async fn backup_to(&self, path: &std::path::Path) -> Result<(), Error> {
        self.ensure_running()?;
        self.db.backup_to(path).await?;
        Ok(())
    }
    pub async fn prepare_restore(&self, path: &std::path::Path) -> Result<(), Error> {
        self.ensure_running()?;
        self.backups.prepare(path).await
    }
    pub async fn cancel_restore(&self) {
        self.backups.cancel().await;
    }
    pub async fn confirm_restore(&self) -> Result<(), Error> {
        self.ensure_running()?;
        self.backups.confirm().await
    }
    fn ensure_running(&self) -> Result<(), Error> {
        if self.stopping.load(Ordering::Acquire) {
            return Err(Error::Stopped);
        }
        Ok(())
    }
    pub async fn tracking_paused(&self) -> bool {
        self.services.lock().await.paused
    }
    pub async fn tracking_status(&self) -> NativeTrackingStatus {
        let services = self.services.lock().await;
        let health = if let Some(task) = &services.tracking {
            if task.is_finished() {
                TrackingHealth {
                    state: TrackingState::Degraded,
                    last_error: Some("worker_stopped".into()),
                }
            } else {
                task.subscribe().borrow().clone()
            }
        } else if services.paused || services.stopped || matches!(self.source, Tracking::Disabled) {
            TrackingHealth {
                state: TrackingState::Stopped,
                last_error: None,
            }
        } else {
            TrackingHealth {
                state: TrackingState::Degraded,
                last_error: Some("discovery_unavailable".into()),
            }
        };
        NativeTrackingStatus {
            paused: services.paused,
            health,
        }
    }
    pub async fn set_tracking_paused(&self, paused: bool) -> Result<(), Error> {
        let mut services = self.services.lock().await;
        if services.stopped || self.stopping.load(Ordering::Acquire) {
            return Err(Error::Stopped);
        }
        if services.paused == paused {
            return Ok(());
        }
        if let Some(tracking) = services.tracking.take() {
            tracking.stop().await?;
        }
        if !paused {
            services.tracking = start_tracking(&self.app, &self.source);
        }
        services.paused = paused;
        Ok(())
    }
    pub async fn stop(&self) -> Result<(), Error> {
        self.stopping.store(true, Ordering::Release);
        let mut services = self.services.lock().await;
        if services.stopped {
            return Ok(());
        }
        let mut failure = None;
        if let Some(tracking) = services.tracking.take()
            && let Err(error) = tracking.stop().await
        {
            failure = Some(Error::Tracking(error));
        }
        if let Some(server) = services.server.take()
            && let Err(error) = server.stop().await
        {
            failure.get_or_insert(Error::Io(error));
        }
        if let Err(error) = self.db.close().await {
            failure.get_or_insert(Error::Storage(error));
        }
        services.stopped = true;
        failure.map_or(Ok(()), Err)
    }
}
