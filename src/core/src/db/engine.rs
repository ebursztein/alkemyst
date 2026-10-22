use super::{Error, Metrics, Result, Statement, Write, worker::Worker};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::sync::{mpsc, oneshot};

type Reply<T> = oneshot::Sender<Result<T>>;

pub(super) enum Command {
    Read(Statement, Vec<Value>, Reply<Vec<Value>>),
    Write(Vec<Write>, Reply<()>),
    Explain(Statement, Vec<Value>, Reply<Vec<String>>),
    Metrics(Reply<Metrics>),
    Close(Reply<()>),
}

#[derive(Clone)]
pub struct Engine {
    sender: mpsc::Sender<Command>,
}

impl Engine {
    pub fn memory(cache_bytes: usize) -> Result<Self> {
        Self::start(None, cache_bytes)
    }

    pub fn open(path: &Path, cache_bytes: usize) -> Result<Self> {
        Self::start(Some(path.to_path_buf()), cache_bytes)
    }

    fn start(path: Option<PathBuf>, cache_bytes: usize) -> Result<Self> {
        let (sender, mut receiver) = mpsc::channel(64);
        let (ready, init) = std::sync::mpsc::sync_channel(1);
        std::thread::Builder::new()
            .name("alkemyst-sql".into())
            .spawn(move || {
                let mut worker = match Worker::open(path.as_deref(), cache_bytes) {
                    Ok(worker) => {
                        if ready.send(Ok(())).is_err() {
                            return;
                        }
                        worker
                    }
                    Err(error) => {
                        let _ = ready.send(Err(error));
                        return;
                    }
                };
                while let Some(command) = receiver.blocking_recv() {
                    match command {
                        Command::Read(query, params, reply) => {
                            let _ = reply.send(worker.read(query, params));
                        }
                        Command::Write(writes, reply) => {
                            let _ = reply.send(worker.write(writes));
                        }
                        Command::Explain(query, params, reply) => {
                            let _ = reply.send(worker.explain(query, params));
                        }
                        Command::Metrics(reply) => {
                            let _ = reply.send(Ok(worker.metrics()));
                        }
                        Command::Close(reply) => {
                            let _ = reply.send(Ok(()));
                            break;
                        }
                    }
                }
            })?;
        init.recv().map_err(|_| Error::Closed)??;
        Ok(Self { sender })
    }

    async fn request<T>(&self, build: impl FnOnce(Reply<T>) -> Command) -> Result<T> {
        let (reply, response) = oneshot::channel();
        self.sender
            .send(build(reply))
            .await
            .map_err(|_| Error::Closed)?;
        response.await.map_err(|_| Error::Closed)?
    }

    pub async fn read(&self, statement: Statement, params: Vec<Value>) -> Result<Vec<Value>> {
        self.request(|reply| Command::Read(statement, params, reply))
            .await
    }

    pub async fn write(&self, writes: Vec<Write>) -> Result<()> {
        self.request(|reply| Command::Write(writes, reply)).await
    }

    pub async fn explain(&self, statement: Statement, params: Vec<Value>) -> Result<Vec<String>> {
        self.request(|reply| Command::Explain(statement, params, reply))
            .await
    }

    pub async fn metrics(&self) -> Result<Metrics> {
        self.request(Command::Metrics).await
    }

    /// Stop this database worker, including access through existing clones.
    pub async fn close(&self) -> Result<()> {
        self.request(Command::Close).await
    }
}
