use super::{Error, NativeRuntime};
use crate::{CardCacheHealth, CardCacheState};
use std::path::PathBuf;

impl NativeRuntime {
    pub async fn card_cache_status(&self) -> CardCacheHealth {
        self.services.lock().await.cards.as_ref().map_or(
            CardCacheHealth {
                state: CardCacheState::Stopped,
                last_error: None,
                checks: 0,
            },
            |task| task.status(),
        )
    }
    pub async fn retry_card_cache(&self) -> Result<(), Error> {
        self.ensure_running()?;
        let services = self.services.lock().await;
        services.cards.as_ref().ok_or(Error::Stopped)?.retry();
        Ok(())
    }
    pub async fn choose_card_cache(&self, file: PathBuf) -> Result<(), Error> {
        self.ensure_running()?;
        let services = self.services.lock().await;
        services
            .cards
            .as_ref()
            .ok_or(Error::Stopped)?
            .choose(file)
            .await;
        Ok(())
    }
}
