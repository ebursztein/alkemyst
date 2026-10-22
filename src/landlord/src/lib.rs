//! Native MTG simulation primitives. Card data is supplied by the caller.
//! Mana matching derives from MIT mtgoncurve/landlord; see UPSTREAM.md.
mod bipartite;
pub mod mana;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("invalid or unsupported mana cost: {0}")]
    Mana(String),
    #[error("invalid simulation configuration: {0}")]
    Configuration(String),
}

mod london;
mod rng;
pub use london::{London, OpeningHand, sample_hand};
pub use rng::{REPLAY_VERSION, SimulationRng};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card {
    pub id: u32,
    pub is_land: bool,
    pub keep_priority: i16,
}

pub mod probability;
pub mod simulation;
