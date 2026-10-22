mod api_support;
use alkemyst_core::{
    Application,
    api::{AuthConfig, router},
    catalog::{BulkDescriptor, BulkProvider, Error},
    db::Engine,
};
use api_support::{TOKEN, call};
use axum::http::StatusCode;
use std::{
    io::BufRead,
    sync::{Arc, Mutex},
};

struct BlockedProvider {
    started: Arc<tokio::sync::Notify>,
    resume: Mutex<std::sync::mpsc::Receiver<()>>,
}
impl BulkProvider for BlockedProvider {
    fn descriptor(&self) -> Result<BulkDescriptor, Error> {
        self.started.notify_one();
        self.resume.lock().unwrap().recv().unwrap();
        Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::ConnectionReset,
            "test outage",
        )))
    }
    fn download(&self, _: &BulkDescriptor) -> Result<Box<dyn BufRead + Send>, Error> {
        unreachable!("manifest fails first")
    }
}

#[tokio::test]
async fn refresh_busy_and_network_failures_are_typed_and_keep_local_reads_available() {
    let started = Arc::new(tokio::sync::Notify::new());
    let (resume, receiver) = std::sync::mpsc::channel();
    let app = Application::with_bulk_provider(
        Engine::memory(4096).unwrap(),
        Arc::new(BlockedProvider {
            started: started.clone(),
            resume: Mutex::new(receiver),
        }),
    );
    let routes = router(app, AuthConfig::new(TOKEN.into(), false).unwrap());
    let first_routes = routes.clone();
    let refreshing =
        tokio::spawn(async move { call(&first_routes, "POST", "/v1/catalog/refresh", None).await });
    started.notified().await;
    let busy = call(&routes, "POST", "/v1/catalog/refresh", None).await;
    assert_eq!(busy.0, StatusCode::CONFLICT);
    assert_eq!(busy.1["code"], "busy");
    assert_eq!(
        call(&routes, "GET", "/v1/catalog", None).await.1["cards"],
        0
    );
    resume.send(()).unwrap();
    let failed = refreshing.await.unwrap();
    assert_eq!(failed.0, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(failed.1["code"], "busy");
}
