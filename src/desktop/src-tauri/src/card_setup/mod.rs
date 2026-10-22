mod discovery;
mod locations;
mod vdf;
use alkemyst_core::catalog::{Catalog, Error};
use serde::Serialize;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::sync::{Mutex, Notify, watch};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Debug)]
pub enum CardSources {
    Discover,
    Directories(Vec<PathBuf>),
    File(PathBuf),
    Disabled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CardCacheState {
    Disabled,
    Searching,
    Ready,
    Missing,
    Degraded,
    Stopped,
}

#[derive(Clone, Debug, Serialize)]
pub struct CardCacheHealth {
    pub state: CardCacheState,
    /// Stable classifications only; no paths or game data cross the native bridge.
    pub last_error: Option<&'static str>,
    pub checks: u64,
}

pub(crate) struct CardTask {
    stop: CancellationToken,
    retry: Arc<Notify>,
    sources: Arc<Mutex<CardSources>>,
    health: watch::Receiver<CardCacheHealth>,
    task: Option<tokio::task::JoinHandle<()>>,
}

impl CardTask {
    pub fn start(catalog: Catalog, sources: CardSources) -> Self {
        let stop = CancellationToken::new();
        let cancelled = stop.clone();
        let retry = Arc::new(Notify::new());
        let wake = retry.clone();
        let sources = Arc::new(Mutex::new(sources));
        let configuration = sources.clone();
        let (publish, health) = watch::channel(CardCacheHealth {
            state: CardCacheState::Searching,
            last_error: None,
            checks: 0,
        });
        let task = tokio::spawn(async move {
            let mut previous = None;
            loop {
                let sources = configuration.lock().await.clone();
                publish.send_modify(|health| health.state = CardCacheState::Searching);
                let result = tokio::select! {
                    biased;
                    _ = cancelled.cancelled() => break,
                    result = scan(&catalog, sources, &mut previous) => result,
                };
                let (state, error) = result;
                publish.send_modify(|health| {
                    health.state = state;
                    health.last_error = error;
                    health.checks += 1;
                });
                let wait = if error == Some("metadata_busy") {
                    1
                } else {
                    30
                };
                tokio::select! {
                    biased;
                    _ = cancelled.cancelled() => break,
                    _ = wake.notified() => {},
                    _ = tokio::time::sleep(Duration::from_secs(wait)) => {},
                }
            }
            publish.send_modify(|health| health.state = CardCacheState::Stopped);
        });
        Self {
            stop,
            retry,
            sources,
            health,
            task: Some(task),
        }
    }
    pub fn status(&self) -> CardCacheHealth {
        let mut health = self.health.borrow().clone();
        if self.task.as_ref().is_some_and(|task| task.is_finished()) && !self.stop.is_cancelled() {
            health.state = CardCacheState::Degraded;
            health.last_error = Some("worker_stopped");
        }
        health
    }
    pub fn retry(&self) {
        self.retry.notify_one();
    }
    pub async fn choose(&self, file: PathBuf) {
        *self.sources.lock().await = CardSources::File(file);
        self.retry();
    }
    pub async fn stop(mut self) -> Result<(), tokio::task::JoinError> {
        self.stop.cancel();
        if let Some(task) = self.task.take() {
            task.await?;
        }
        Ok(())
    }
}
impl Drop for CardTask {
    fn drop(&mut self) {
        self.stop.cancel();
    }
}

type ScanResult = (CardCacheState, Option<&'static str>);
async fn scan(
    catalog: &Catalog,
    sources: CardSources,
    previous: &mut Option<discovery::Asset>,
) -> ScanResult {
    if matches!(sources, CardSources::Disabled) {
        return (CardCacheState::Disabled, None);
    }
    let candidates = match tokio::task::spawn_blocking(move || discovery::find(sources)).await {
        Ok(Ok(candidates)) => candidates,
        _ => return (CardCacheState::Degraded, Some("cache_unavailable")),
    };
    if candidates.is_empty() {
        return (CardCacheState::Missing, None);
    }
    let mut failed = false;
    for candidate in candidates {
        if previous.as_ref() == Some(&candidate) {
            return if failed {
                (CardCacheState::Degraded, Some("cache_unavailable"))
            } else {
                (CardCacheState::Ready, None)
            };
        }
        match catalog.import_arena_database(&candidate.path).await {
            Ok(count) if count > 0 => {
                *previous = Some(candidate);
                return if failed {
                    (CardCacheState::Degraded, Some("cache_unavailable"))
                } else {
                    (CardCacheState::Ready, None)
                };
            }
            Err(Error::Busy) => return (CardCacheState::Searching, Some("metadata_busy")),
            _ => failed = true,
        }
    }
    (CardCacheState::Degraded, Some("cache_unavailable"))
}
