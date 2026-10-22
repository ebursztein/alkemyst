use super::*;
use crate::db::Engine;
use std::sync::Arc;
use tokio::{
    sync::{Mutex, Notify, oneshot},
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
pub(in crate::analysis) struct Coordinator {
    pub stop: CancellationToken,
    pub notify: Arc<Notify>,
    task: Mutex<Option<JoinHandle<()>>>,
}
impl Coordinator {
    pub fn new() -> Self {
        Self {
            stop: CancellationToken::new(),
            notify: Arc::new(Notify::new()),
            task: Mutex::new(None),
        }
    }
    async fn start(&self, db: Engine) -> Result<(), Error> {
        if self.stop.is_cancelled() {
            return Err(Error::Stopped);
        }
        let mut task = self.task.lock().await;
        if task.is_none() {
            let (ready, started) = oneshot::channel();
            *task = Some(tokio::spawn(super::worker::run(
                db,
                self.stop.clone(),
                self.notify.clone(),
                ready,
            )));
            drop(task);
            let _ = started.await;
        }
        Ok(())
    }
}
impl Drop for Coordinator {
    fn drop(&mut self) {
        self.stop.cancel();
    }
}
impl Analysis {
    pub async fn start_jobs(&self) -> Result<(), Error> {
        self.coordinator.start(self.db.clone()).await
    }
    pub fn signal_jobs_stop(&self) {
        self.coordinator.stop.cancel();
    }
    pub async fn stop_jobs(&self) -> Result<(), Error> {
        self.signal_jobs_stop();
        let task = self.coordinator.task.lock().await.take();
        if let Some(task) = task {
            task.await.map_err(|_| Error::Stored)?;
        }
        Ok(())
    }
    pub async fn submit_simulation(
        &self,
        key: &str,
        p: SimulationParameters,
        origin: crate::domain::Origin,
    ) -> Result<SimulationJob, Error> {
        let job = self.create_simulation(key, p, origin).await?;
        self.start_jobs().await?;
        Ok(job)
    }
}
