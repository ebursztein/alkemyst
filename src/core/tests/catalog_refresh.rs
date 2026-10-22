mod support;
use alkemyst_core::{
    catalog::{BulkDescriptor, BulkProvider, Catalog, Error},
    db::Engine,
};
use std::{
    io::{BufRead, Cursor},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use support::{cards, source};

struct Provider {
    downloads: Arc<AtomicUsize>,
    fail_manifest: bool,
    fail_download: bool,
}
#[async_trait::async_trait]
impl BulkProvider for Provider {
    async fn descriptor(&self) -> Result<BulkDescriptor, Error> {
        if self.fail_manifest {
            return Err(Error::Invalid("offline"));
        }
        Ok(BulkDescriptor {
            source: source(),
            compressed_size: 100,
        })
    }
    async fn download(&self, _: &BulkDescriptor) -> Result<Box<dyn BufRead + Send>, Error> {
        self.downloads.fetch_add(1, Ordering::SeqCst);
        if self.fail_download {
            return Err(Error::Invalid("interrupted download"));
        }
        Ok(Box::new(Cursor::new(cards())))
    }
}

#[tokio::test]
async fn refresh_reuses_unchanged_snapshot_and_offline_reads_remain_available() {
    let catalog = Catalog::new(Engine::memory(1024 * 1024).unwrap());
    let downloads = Arc::new(AtomicUsize::new(0));
    let first = catalog
        .refresh_with(Provider {
            downloads: downloads.clone(),
            fail_manifest: false,
            fail_download: false,
        })
        .await
        .unwrap();
    assert_eq!(first.cards, 3);
    let again = catalog
        .refresh_with(Provider {
            downloads: downloads.clone(),
            fail_manifest: false,
            fail_download: false,
        })
        .await
        .unwrap();
    assert_eq!(first.snapshot_id, again.snapshot_id);
    assert_eq!(downloads.load(Ordering::SeqCst), 1);
    assert!(
        catalog
            .refresh_with(Provider {
                downloads,
                fail_manifest: true,
                fail_download: false
            })
            .await
            .is_err()
    );
    assert_eq!(catalog.status().await.unwrap().cards, 3);
    assert!(catalog.card("oracle-equipment").await.unwrap().is_some());
}

#[tokio::test]
async fn failed_download_does_not_publish_an_empty_catalog() {
    let catalog = Catalog::new(Engine::memory(1024 * 1024).unwrap());
    assert!(
        catalog
            .refresh_with(Provider {
                downloads: Arc::default(),
                fail_manifest: false,
                fail_download: true,
            })
            .await
            .is_err()
    );
    assert_eq!(catalog.status().await.unwrap().cards, 0);
}

#[tokio::test]
async fn corrupt_empty_duplicate_and_oversized_records_never_replace_valid_catalog() {
    let catalog = Catalog::new(Engine::memory(1024 * 1024).unwrap());
    let original = catalog
        .import_jsonl(Cursor::new(cards()), source())
        .await
        .unwrap();
    let first = cards().lines().next().unwrap().to_string();
    for broken in [
        "".to_string(),
        " \n".to_string(),
        "{}".to_string(),
        format!("{first}\n{first}"),
        first.replace("\"cmc\":2", "\"cmc\":\"two\""),
        "x".repeat(1024 * 1024 + 1),
    ] {
        assert!(
            catalog
                .import_jsonl(Cursor::new(broken), source())
                .await
                .is_err()
        );
        assert_eq!(
            catalog.status().await.unwrap().snapshot_id,
            original.snapshot_id
        );
    }
    assert!(
        catalog
            .import_jsonl(
                Cursor::new(cards()),
                alkemyst_core::catalog::CatalogSource {
                    url: String::new(),
                    ..source()
                }
            )
            .await
            .is_err()
    );
}
