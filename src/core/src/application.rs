use crate::{
    catalog::{BulkProvider, Catalog, CatalogStatus, Scryfall},
    db::Engine,
    domain::Decks,
};

/// One instance is cloned into REST, MCP and the native shell. Its operations
/// share one SQL worker, one result cache and one catalog refresh coordinator.
#[derive(Clone)]
pub struct Application {
    pub catalog: Catalog,
    pub analysis: crate::analysis::Analysis,
    pub decks: Decks,
    pub ingest: crate::ingest::Ingestor,
    pub history: crate::history::History,
    provider: std::sync::Arc<dyn BulkProvider>,
}

impl Application {
    pub fn new(db: Engine) -> Self {
        Self::with_bulk_provider(db, std::sync::Arc::new(Scryfall))
    }

    pub fn with_bulk_provider(db: Engine, provider: std::sync::Arc<dyn BulkProvider>) -> Self {
        Self {
            catalog: Catalog::new(db.clone()),
            analysis: crate::analysis::Analysis::new(db.clone()),
            ingest: crate::ingest::Ingestor::new(db.clone()),
            history: crate::history::History::new(db.clone()),
            decks: Decks::new(db),
            provider,
        }
    }
    pub async fn refresh_catalog(&self) -> Result<CatalogStatus, crate::catalog::Error> {
        self.catalog.refresh_with(self.provider.clone()).await
    }
}
