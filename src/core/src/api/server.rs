use super::{AuthConfig, router_with_mcp};
use crate::Application;
use hyper::server::conn::http1;
use hyper_util::{rt::TokioIo, service::TowerToHyperService};
use std::{io, net::SocketAddr};
use tokio::task::{JoinHandle, JoinSet};
use tokio_util::sync::CancellationToken;

pub struct RunningServer {
    address: SocketAddr,
    analysis: crate::analysis::Analysis,
    mcp_stop: CancellationToken,
    task: Option<JoinHandle<io::Result<()>>>,
}
impl RunningServer {
    pub async fn start(
        app: Application,
        auth: AuthConfig,
        address: SocketAddr,
    ) -> io::Result<Self> {
        if !address.ip().is_loopback() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Alkemyst only binds loopback",
            ));
        }
        let listener = tokio::net::TcpListener::bind(address).await?;
        let address = listener.local_addr()?;
        let config = rmcp::transport::streamable_http_server::StreamableHttpServerConfig::default();
        let mcp_stop = config.cancellation_token.clone();
        let stop = mcp_stop.clone();
        let analysis = app.analysis.clone();
        analysis.start_jobs().await.map_err(io::Error::other)?;
        let routes = router_with_mcp(app, auth, config);
        let task = tokio::spawn(async move {
            let mut connections = JoinSet::new();
            let mut failure = None;
            loop {
                tokio::select! {
                    biased;
                    _ = stop.cancelled() => break,
                    _ = connections.join_next(), if !connections.is_empty() => {},
                    accepted = listener.accept() => {
                        let (stream, _) = match accepted {
                            Ok(connection) => connection,
                            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                            Err(error) => { failure = Some(error); stop.cancel(); break; }
                        };
                        let service = TowerToHyperService::new(routes.clone());
                        let stopped = stop.clone();
                        connections.spawn(async move {
                            // Dropping the connection future also cancels its active handler,
                            // including partial request bodies and async metadata acquisition.
                            tokio::select! {
                                biased;
                                _ = stopped.cancelled() => {},
                                _ = http1::Builder::new().serve_connection(TokioIo::new(stream), service) => {},
                            }
                        });
                    }
                }
            }
            drop(listener);
            while connections.join_next().await.is_some() {}
            failure.map_or(Ok(()), Err)
        });
        Ok(Self {
            address,
            analysis,
            mcp_stop,
            task: Some(task),
        })
    }
    pub fn address(&self) -> SocketAddr {
        self.address
    }
    pub async fn stop(mut self) -> io::Result<()> {
        self.signal();
        let serving = match self.task.take() {
            Some(task) => task
                .await
                .map_err(io::Error::other)
                .and_then(|result| result),
            None => Ok(()),
        };
        // Join jobs even if the listener failed; queued SQL writes must finish before DB close.
        let jobs = self.analysis.stop_jobs().await.map_err(io::Error::other);
        serving.and(jobs)
    }
    fn signal(&mut self) {
        self.mcp_stop.cancel();
        self.analysis.signal_jobs_stop();
    }
}
impl Drop for RunningServer {
    fn drop(&mut self) {
        self.signal();
    }
}
