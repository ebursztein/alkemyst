//! Explicit development-only replacement for the external Scryfall service.
//! The native application does not read these fixture environment variables.
use alkemyst_core::{
    Application,
    catalog::{BulkDescriptor, BulkProvider, CatalogSource, Error},
    db::Engine,
};
use std::{
    io::{BufRead, BufReader, Cursor},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

pub fn application(db: Engine) -> Application {
    match std::env::var_os("ALKEMYST_FIXTURE_CATALOG").filter(|path| !path.is_empty()) {
        Some(path) => Application::with_bulk_provider(
            db,
            Arc::new(Fixture {
                path: path.into(),
                fail_once: AtomicBool::new(
                    std::env::var("ALKEMYST_FIXTURE_FAIL_FIRST").as_deref() == Ok("1"),
                ),
            }),
        ),
        None => Application::new(db),
    }
}

struct Fixture {
    path: PathBuf,
    fail_once: AtomicBool,
}
#[async_trait::async_trait]
impl BulkProvider for Fixture {
    async fn descriptor(&self) -> Result<BulkDescriptor, Error> {
        if self.fail_once.swap(false, Ordering::AcqRel) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::ConnectionReset,
                "fixture upstream outage",
            )
            .into());
        }
        Ok(BulkDescriptor {
            source: CatalogSource {
                url: "https://data.scryfall.io/default-cards/fixture.jsonl.gz".into(),
                updated_at: "2026-10-21T21:00:00Z".into(),
            },
            compressed_size: tokio::fs::metadata(&self.path).await?.len(),
        })
    }
    async fn download(&self, _: &BulkDescriptor) -> Result<Box<dyn BufRead + Send>, Error> {
        let bytes = tokio::fs::read(&self.path).await?;
        Ok(Box::new(BufReader::new(flate2::read::MultiGzDecoder::new(
            Cursor::new(bytes),
        ))))
    }
}
