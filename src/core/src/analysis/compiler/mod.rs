//! Reviewed card rules become explicit native model inputs. Unknown effects block.
mod card;
mod fingerprint;
mod generic;
mod lands;
mod models;
mod presets;
mod registry;
mod sources;
use super::{
    DeckSnapshot,
    simulation::{BattlefieldRunner, MODEL_VERSION, POLICY, Scenario},
    turns::{CardSpec, Goal},
};
use crate::domain::Zone;
use landlord::London;
pub use models::{CardProfile, Issue, IssueCode, Profile, SourceCount};
use std::{collections::BTreeMap, sync::Arc};
pub const COMPILER_VERSION: &str = "catalog-model-v1";
pub struct CompiledDeck {
    pub profile: Profile,
    models: Arc<[CardSpec]>,
    snapshot: DeckSnapshot,
}
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("deck contains unresolved or unsupported model inputs")]
    Unsupported,
    #[error("explicit model exclusions have not been accepted")]
    Exclusions,
    #[error(transparent)]
    Configuration(#[from] landlord::Error),
}
pub fn compile(snapshot: DeckSnapshot) -> CompiledDeck {
    let original = snapshot.clone();
    let keys = snapshot
        .cards
        .iter()
        .filter_map(|r| r.data.as_ref())
        .filter_map(|v| {
            v["card_faces"][0]["name"]
                .as_str()
                .or_else(|| v["name"].as_str())
        })
        .map(str::to_owned)
        .collect::<std::collections::BTreeSet<_>>();
    let identities = keys
        .into_iter()
        .enumerate()
        .map(|(i, k)| (k, (i + 1) as u32))
        .collect::<BTreeMap<_, _>>();
    let mut models = vec![];
    let mut cards = vec![];
    for input in &snapshot.cards {
        let name = input.data.as_ref().and_then(|v| {
            v["card_faces"][0]["name"]
                .as_str()
                .or_else(|| v["name"].as_str())
        });
        let id = name
            .and_then(|name| identities.get(name))
            .copied()
            .unwrap_or(0);
        let (profile, model) = card::compile(input, id);
        if input.zone == Zone::Main
            && let Some(model) = model
        {
            models.extend(std::iter::repeat_n(model, usize::from(input.quantity)));
        }
        cards.push(profile);
    }
    let can_simulate = !models.is_empty()
        && models.len() <= 1000
        && models.len() == snapshot.counts.main as usize
        && !cards
            .iter()
            .any(|c| c.zone == Zone::Main && c.issues.iter().any(|i| i.blocking));
    let profile = Profile {
        version_id: snapshot.version_id,
        deck_id: snapshot.deck_id,
        name: snapshot.name,
        format: snapshot.format,
        catalog_snapshot: snapshot.catalog_snapshot,
        catalog_updated_at: snapshot.catalog_updated_at,
        compiler_version: COMPILER_VERSION.into(),
        model_version: MODEL_VERSION.into(),
        policy_version: POLICY.into(),
        sources: sources::counts(&models),
        counts: snapshot.counts,
        curve: snapshot.curve,
        cards,
        can_simulate,
        assumptions: vec![
            "Precombat main phases only; no attacks, opponent interaction or combat damage.".into(),
            "Priority-greedy play, oldest entry draws first; loot ignored cards and known extra legends.".into(),
            "Only explicitly compiled abilities are executed; card-specific exclusions remain visible.".into(),
        ],
    };
    CompiledDeck {
        profile,
        models: Arc::from(models),
        snapshot: original,
    }
}
impl CompiledDeck {
    pub fn snapshot(&self) -> &DeckSnapshot {
        &self.snapshot
    }

    pub fn scenario(
        &self,
        turns: u8,
        on_the_play: bool,
        mulligan: London,
        goal: Goal,
        accept_exclusions: bool,
    ) -> Result<Scenario, Error> {
        if !self.profile.can_simulate {
            return Err(Error::Unsupported);
        }
        if !accept_exclusions
            && self
                .profile
                .cards
                .iter()
                .any(|c| c.zone == Zone::Main && !c.issues.is_empty())
        {
            return Err(Error::Exclusions);
        }
        let scenario = Scenario {
            deck: self.models.clone(),
            turns,
            on_the_play,
            mulligan,
            goal,
        };
        BattlefieldRunner::new(scenario.clone(), 1, 0)?;
        Ok(scenario)
    }
}
