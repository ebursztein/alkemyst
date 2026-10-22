use super::{Ingestor, Result};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;
use utoipa::ToSchema;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TrackingState {
    Waiting,
    Tracking,
    Degraded,
    Stopped,
}
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct TrackingHealth {
    pub state: TrackingState,
    /// Stable error classification, never a local path or raw log content.
    pub last_error: Option<String>,
}
pub struct TrackingTask {
    stop: CancellationToken,
    health: watch::Receiver<TrackingHealth>,
    task: Option<tokio::task::JoinHandle<()>>,
}
impl Ingestor {
    pub fn watch(&self, previous: PathBuf, current: PathBuf) -> TrackingTask {
        let ingestor = self.clone();
        let stop = CancellationToken::new();
        let cancelled = stop.clone();
        let (publish, health) = watch::channel(TrackingHealth {
            state: TrackingState::Waiting,
            last_error: None,
        });
        let task = tokio::spawn(async move {
            let mut tick = tokio::time::interval(Duration::from_millis(500));
            'tracking: loop {
                tokio::select! {_=cancelled.cancelled()=>break,_=tick.tick()=>{}}
                let mut available = false;
                let mut last_error = None;
                for (path, retained) in [(&previous, true), (&current, false)] {
                    let poll = async {
                        if retained {
                            ingestor.import_file(path).await
                        } else {
                            ingestor.poll_file(path).await
                        }
                    };
                    let result = tokio::select! {_=cancelled.cancelled()=>break 'tracking,result=poll=>result};
                    match result {
                        Ok(_) => available = true,
                        Err(super::Error::Io(error))
                            if error.kind() == std::io::ErrorKind::NotFound => {}
                        Err(error) => {
                            last_error = Some(
                                match error {
                                    super::Error::Database(_) => "storage_unavailable",
                                    super::Error::Io(_) => "log_unavailable",
                                    super::Error::Worker => "worker_stopped",
                                    _ => "invalid_capture",
                                }
                                .into(),
                            )
                        }
                    }
                }
                let state = if last_error.is_some() {
                    TrackingState::Degraded
                } else if available {
                    TrackingState::Tracking
                } else {
                    TrackingState::Waiting
                };
                publish.send_replace(TrackingHealth { state, last_error });
            }
            publish.send_modify(|health| health.state = TrackingState::Stopped);
        });
        TrackingTask {
            stop,
            health,
            task: Some(task),
        }
    }
    pub fn discover_and_watch(
        &self,
    ) -> std::result::Result<TrackingTask, manasight_parser::log::discovery::DiscoveryError> {
        let paths = manasight_parser::log::discovery::resolve_log_paths()?;
        Ok(self.watch(
            paths.player_prev_log().to_path_buf(),
            paths.player_log().to_path_buf(),
        ))
    }
}
impl TrackingTask {
    pub fn subscribe(&self) -> watch::Receiver<TrackingHealth> {
        self.health.clone()
    }
    pub async fn stop(mut self) -> Result<()> {
        self.stop.cancel();
        if let Some(task) = self.task.take() {
            task.await.map_err(|_| super::Error::Worker)?;
        }
        Ok(())
    }
    pub fn is_finished(&self) -> bool {
        self.task.as_ref().is_none_or(|task| task.is_finished())
    }
}
impl Drop for TrackingTask {
    fn drop(&mut self) {
        self.stop.cancel();
    }
}
