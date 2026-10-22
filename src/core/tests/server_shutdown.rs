use alkemyst_core::{
    Application,
    api::{AuthConfig, RunningServer},
    db::Engine,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    time::{Duration, timeout},
};

#[tokio::test]
async fn stop_closes_an_active_request_with_an_incomplete_body() {
    let db = Engine::memory(4096).unwrap();
    let app = Application::new(db.clone());
    let auth = AuthConfig::generate();
    let server = RunningServer::start(app.clone(), auth.clone(), "127.0.0.1:0".parse().unwrap())
        .await
        .unwrap();
    let address = server.address();
    let mut socket = TcpStream::connect(address).await.unwrap();
    socket.write_all(format!(
        "POST /v1/decks HTTP/1.1\r\nHost: {address}\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: 1024\r\nExpect: 100-continue\r\nIdempotency-Key: shutdown-pending-body\r\n\r\n",
        auth.credential(),
    ).as_bytes()).await.unwrap();
    let mut ready = vec![0; b"HTTP/1.1 100 Continue\r\n\r\n".len()];
    timeout(Duration::from_secs(2), socket.read_exact(&mut ready))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(ready, b"HTTP/1.1 100 Continue\r\n\r\n");
    let stopped = timeout(Duration::from_millis(500), server.stop()).await;
    // Unblock the old implementation after its measured failure so the fixture cleans up.
    drop(socket);
    app.analysis.stop_jobs().await.unwrap();
    db.close().await.unwrap();
    assert!(
        matches!(stopped, Ok(Ok(()))),
        "stop waited for a request body that never arrived"
    );
}

struct PendingDownload {
    started: std::sync::Arc<tokio::sync::Notify>,
    resume: std::sync::Arc<tokio::sync::Notify>,
    dropped: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
struct DownloadGuard(std::sync::Arc<std::sync::atomic::AtomicBool>);
impl Drop for DownloadGuard {
    fn drop(&mut self) {
        self.0.store(true, std::sync::atomic::Ordering::Release);
    }
}
#[async_trait::async_trait]
impl alkemyst_core::catalog::BulkProvider for PendingDownload {
    async fn descriptor(
        &self,
    ) -> Result<alkemyst_core::catalog::BulkDescriptor, alkemyst_core::catalog::Error> {
        Ok(alkemyst_core::catalog::BulkDescriptor {
            source: alkemyst_core::catalog::CatalogSource {
                url: "https://data.scryfall.io/next.jsonl.gz".into(),
                updated_at: "2026-10-22T00:00:00Z".into(),
            },
            compressed_size: 3,
        })
    }
    async fn download(
        &self,
        _: &alkemyst_core::catalog::BulkDescriptor,
    ) -> Result<Box<dyn std::io::BufRead + Send>, alkemyst_core::catalog::Error> {
        let _guard = DownloadGuard(self.dropped.clone());
        self.started.notify_one();
        self.resume.notified().await;
        Err(alkemyst_core::catalog::Error::Invalid(
            "test download interrupted",
        ))
    }
}

#[tokio::test]
async fn stop_cancels_pending_catalog_acquisition_and_keeps_the_previous_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("catalog.sqlite3");
    let db = Engine::open(&path, 4096).unwrap();
    let catalog = alkemyst_core::catalog::Catalog::new(db.clone());
    let old = catalog
        .import_jsonl(
            std::io::Cursor::new(include_str!("fixtures/cards.jsonl")),
            alkemyst_core::catalog::CatalogSource {
                url: "https://data.scryfall.io/old.jsonl.gz".into(),
                updated_at: "2026-10-21T00:00:00Z".into(),
            },
        )
        .await
        .unwrap();
    let started = std::sync::Arc::new(tokio::sync::Notify::new());
    let dropped = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let resume = std::sync::Arc::new(tokio::sync::Notify::new());
    let app = Application::with_bulk_provider(
        db.clone(),
        std::sync::Arc::new(PendingDownload {
            started: started.clone(),
            resume: resume.clone(),
            dropped: dropped.clone(),
        }),
    );
    let auth = AuthConfig::generate();
    let server = RunningServer::start(app.clone(), auth.clone(), "127.0.0.1:0".parse().unwrap())
        .await
        .unwrap();
    let url = format!("http://{}/v1/catalog/refresh", server.address());
    let request = tokio::spawn(async move {
        reqwest::Client::new()
            .post(url)
            .bearer_auth(auth.credential())
            .send()
            .await
    });
    timeout(Duration::from_secs(2), started.notified())
        .await
        .unwrap();
    let stopped = timeout(Duration::from_millis(500), server.stop()).await;
    let acquisition_cancelled = dropped.load(std::sync::atomic::Ordering::Acquire);
    // Keep cleanup explicit if an implementation fails to cancel its pending provider.
    resume.notify_one();
    let _ = timeout(Duration::from_secs(2), request).await.unwrap();
    app.analysis.stop_jobs().await.unwrap();
    db.close().await.unwrap();
    let reopened = Engine::open(&path, 4096).unwrap();
    let current = alkemyst_core::catalog::Catalog::new(reopened.clone())
        .status()
        .await
        .unwrap();
    assert_eq!(current.snapshot_id, old.snapshot_id);
    assert_eq!(current.cards, old.cards);
    reopened.close().await.unwrap();
    assert!(
        matches!(stopped, Ok(Ok(()))),
        "stop waited for metadata acquisition"
    );
    assert!(acquisition_cancelled, "a download task outlived shutdown");
}
