mod support;
use alkemyst_core::{
    catalog::{Catalog, Error, Search},
    db::Engine,
};
use std::io::{BufRead, Read};
use support::{cards, source};

struct PausedReader {
    input: std::io::Cursor<Vec<u8>>,
    ready: Option<std::sync::mpsc::Sender<()>>,
    resume: std::sync::mpsc::Receiver<()>,
}
impl Read for PausedReader {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let available = self.fill_buf()?;
        let length = available.len().min(buffer.len());
        buffer[..length].copy_from_slice(&available[..length]);
        self.consume(length);
        Ok(length)
    }
}
impl BufRead for PausedReader {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        if self.input.position() == self.input.get_ref().len() as u64
            && let Some(ready) = self.ready.take()
        {
            let _ = ready.send(());
            self.resume.recv().map_err(std::io::Error::other)?;
        }
        self.input.fill_buf()
    }
    fn consume(&mut self, amount: usize) {
        self.input.consume(amount);
    }
}

#[tokio::test]
async fn cancelled_import_and_overlapping_refresh_preserve_active_snapshot() {
    let db = Engine::memory(1024 * 1024).unwrap();
    let catalog = Catalog::new(db.clone());
    let original = catalog
        .import_jsonl(std::io::Cursor::new(cards()), source())
        .await
        .unwrap();
    let first: serde_json::Value = serde_json::from_str(cards().lines().next().unwrap()).unwrap();
    let mut many = String::new();
    for n in 0..128 {
        let mut row = first.clone();
        row["id"] = serde_json::json!(format!("print-{n}"));
        many.push_str(&row.to_string());
        many.push('\n');
    }
    let (ready, wait_ready) = std::sync::mpsc::channel();
    let (resume, wait_resume) = std::sync::mpsc::channel();
    let reader = PausedReader {
        input: std::io::Cursor::new(many.into_bytes()),
        ready: Some(ready),
        resume: wait_resume,
    };
    let importing = catalog.clone();
    let task = tokio::spawn(async move { importing.import_jsonl(reader, source()).await });
    tokio::task::spawn_blocking(move || wait_ready.recv())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        catalog.status().await.unwrap().snapshot_id,
        original.snapshot_id
    );
    assert_eq!(
        catalog.search(Search::default()).await.unwrap().items.len(),
        3
    );
    assert!(matches!(
        catalog
            .import_jsonl(std::io::Cursor::new(cards()), source())
            .await,
        Err(Error::Busy)
    ));
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    resume.send(()).unwrap();
    assert_eq!(
        catalog.status().await.unwrap().snapshot_id,
        original.snapshot_id
    );
    let recovered = catalog
        .import_jsonl(std::io::Cursor::new(cards()), source())
        .await
        .unwrap();
    assert_eq!(recovered.cards, 3);
    assert_ne!(recovered.snapshot_id, original.snapshot_id);
}
