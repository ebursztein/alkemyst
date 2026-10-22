use super::{Catalog, CatalogSource, CatalogStatus, Error, Result};
use reqwest::Client;
use serde_json::Value;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::time::Duration;
use tokio::io::{AsyncWrite, AsyncWriteExt};

const MAX_ARCHIVE: u64 = 512 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct BulkDescriptor {
    pub source: CatalogSource,
    pub compressed_size: u64,
}

/// Cancellation-safe acquisition: dropping the future must stop network work.
#[async_trait::async_trait]
pub trait BulkProvider: Send + Sync + 'static {
    async fn descriptor(&self) -> Result<BulkDescriptor>;
    async fn download(&self, descriptor: &BulkDescriptor) -> Result<Box<dyn BufRead + Send>>;
}

pub struct Scryfall;

#[async_trait::async_trait]
impl BulkProvider for Scryfall {
    async fn descriptor(&self) -> Result<BulkDescriptor> {
        let mut response = client()?
            .get("https://api.scryfall.com/bulk-data/default_cards")
            .send()
            .await?
            .error_for_status()?;
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await? {
            if bytes.len() + chunk.len() > 1024 * 1024 {
                return Err(Error::Invalid("oversized bulk manifest"));
            }
            bytes.extend_from_slice(&chunk);
        }
        descriptor(serde_json::from_slice(&bytes)?)
    }

    async fn download(&self, descriptor: &BulkDescriptor) -> Result<Box<dyn BufRead + Send>> {
        validate(descriptor)?;
        let response = client()?
            .get(&descriptor.source.url)
            .send()
            .await?
            .error_for_status()?;
        let mut file = tempfile::NamedTempFile::new()?;
        let mut writer = tokio::fs::File::from_std(file.reopen()?);
        copy_archive(response, &mut writer, descriptor.compressed_size).await?;
        drop(writer);
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
        let descriptor = provider.descriptor().await?;
        validate(&descriptor)?;
        let status = self.status().await?;
        if status.source.as_ref() == Some(&descriptor.source) {
            return Ok(status);
        }
        let reader = provider.download(&descriptor).await?;
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

async fn copy_archive(
    mut reader: reqwest::Response,
    writer: &mut (impl AsyncWrite + Unpin),
    expected: u64,
) -> Result<()> {
    let mut received = 0_u64;
    while let Some(chunk) = reader.chunk().await? {
        received = received
            .checked_add(chunk.len() as u64)
            .ok_or(Error::Invalid("oversized bulk archive"))?;
        if received > expected {
            return Err(Error::Invalid("oversized bulk archive"));
        }
        writer.write_all(&chunk).await?;
    }
    if received != expected {
        return Err(Error::Invalid("truncated bulk archive"));
    }
    writer.flush().await?;
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/unit/catalog_download.rs"]
mod tests;

#[async_trait::async_trait]
impl<P: BulkProvider + ?Sized> BulkProvider for std::sync::Arc<P> {
    async fn descriptor(&self) -> Result<BulkDescriptor> {
        self.as_ref().descriptor().await
    }
    async fn download(&self, descriptor: &BulkDescriptor) -> Result<Box<dyn BufRead + Send>> {
        self.as_ref().download(descriptor).await
    }
}
