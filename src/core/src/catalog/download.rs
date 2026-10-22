use super::{Catalog, CatalogSource, CatalogStatus, Error, Result};
use reqwest::blocking::Client;
use serde_json::Value;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::time::Duration;

const MAX_ARCHIVE: u64 = 512 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct BulkDescriptor {
    pub source: CatalogSource,
    pub compressed_size: u64,
}

/// Blocking acquisition boundary: callers execute these methods off the UI thread.
pub trait BulkProvider: Send + Sync + 'static {
    fn descriptor(&self) -> Result<BulkDescriptor>;
    fn download(&self, descriptor: &BulkDescriptor) -> Result<Box<dyn BufRead + Send>>;
}

pub struct Scryfall;

impl BulkProvider for Scryfall {
    fn descriptor(&self) -> Result<BulkDescriptor> {
        let response = client()?
            .get("https://api.scryfall.com/bulk-data/default_cards")
            .send()?
            .error_for_status()?;
        let value: Value = serde_json::from_reader(response.take(1024 * 1024))?;
        descriptor(value)
    }

    fn download(&self, descriptor: &BulkDescriptor) -> Result<Box<dyn BufRead + Send>> {
        validate(descriptor)?;
        let response = client()?
            .get(&descriptor.source.url)
            .send()?
            .error_for_status()?;
        let mut file = tempfile::NamedTempFile::new()?;
        copy_archive(response, &mut file, descriptor.compressed_size)?;
        file.as_file_mut().seek(SeekFrom::Start(0))?;
        Ok(Box::new(BufReader::new(flate2::read::MultiGzDecoder::new(
            file,
        ))))
    }
}

impl Catalog {
    pub async fn refresh(&self) -> Result<CatalogStatus> {
        self.refresh_with(Scryfall).await
    }

    pub async fn refresh_with<P: BulkProvider>(&self, provider: P) -> Result<CatalogStatus> {
        let _refresh = self.refresh.try_lock().map_err(|_| Error::Busy)?;
        let provider = std::sync::Arc::new(provider);
        let fetch = provider.clone();
        let descriptor = tokio::task::spawn_blocking(move || fetch.descriptor())
            .await
            .map_err(|_| Error::ParserStopped)??;
        validate(&descriptor)?;
        let status = self.status().await?;
        if status.source.as_ref() == Some(&descriptor.source) {
            return Ok(status);
        }
        let fetch_descriptor = descriptor.clone();
        let reader = tokio::task::spawn_blocking(move || provider.download(&fetch_descriptor))
            .await
            .map_err(|_| Error::ParserStopped)??;
        self.import_inner(reader, descriptor.source).await
    }
}

fn client() -> Result<Client> {
    Ok(Client::builder()
        .user_agent("Alkemyst/0.1 (+https://github.com/ebursztein/alkemyst)")
        .timeout(Duration::from_secs(180))
        .connect_timeout(Duration::from_secs(15))
        .redirect(reqwest::redirect::Policy::none())
        .build()?)
}

fn descriptor(value: Value) -> Result<BulkDescriptor> {
    if value["type"] != "default_cards" {
        return Err(Error::Invalid("expected default_cards bulk"));
    }
    let result = BulkDescriptor {
        source: CatalogSource {
            url: value["jsonl_download_uri"]
                .as_str()
                .ok_or(Error::Invalid("missing JSONL archive URL"))?
                .into(),
            updated_at: value["updated_at"]
                .as_str()
                .ok_or(Error::Invalid("missing bulk timestamp"))?
                .into(),
        },
        compressed_size: value["compressed_size"]
            .as_u64()
            .ok_or(Error::Invalid("missing bulk size"))?,
    };
    validate(&result)?;
    Ok(result)
}

fn validate(descriptor: &BulkDescriptor) -> Result<()> {
    let url = reqwest::Url::parse(&descriptor.source.url)
        .map_err(|_| Error::Invalid("invalid bulk URL"))?;
    if url.scheme() != "https"
        || url.host_str() != Some("data.scryfall.io")
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || descriptor.source.updated_at.is_empty()
        || descriptor.compressed_size == 0
        || descriptor.compressed_size > MAX_ARCHIVE
    {
        return Err(Error::Invalid("unexpected bulk source or size"));
    }
    Ok(())
}

fn copy_archive(reader: impl Read, writer: &mut impl Write, expected: u64) -> Result<()> {
    let received = std::io::copy(&mut reader.take(expected + 1), writer)?;
    if received != expected {
        return Err(Error::Invalid("truncated or oversized bulk archive"));
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/catalog_download.rs"]
mod tests;

impl<P: BulkProvider + ?Sized> BulkProvider for std::sync::Arc<P> {
    fn descriptor(&self) -> Result<BulkDescriptor> {
        self.as_ref().descriptor()
    }
    fn download(&self, descriptor: &BulkDescriptor) -> Result<Box<dyn BufRead + Send>> {
        self.as_ref().download(descriptor)
    }
}
