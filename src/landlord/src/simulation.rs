//! Seeded, bounded Monte Carlo draws without retaining every simulated hand.
use crate::{Card, Error, London, REPLAY_VERSION, SimulationRng, sample_hand};
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct DrawConfig {
    pub runs: u32,
    pub seed: u64,
    /// Number of draw steps after the retained opening hand; play/draw is
    /// resolved by the caller, not an assumed Arena smoothing model.
    pub draws: u16,
    pub targets: Vec<u32>,
    pub mulligan: London,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DrawReport {
    pub runs: u32,
    pub seed: u64,
    pub replay_version: &'static str,
    /// Index = physical target copies seen in kept hand plus requested draws.
    pub histogram: Vec<u32>,
    pub mean_opening_size: f64,
    pub mean_mulligans: f64,
    pub forced_keeps: u32,
}

pub fn simulate_draws(deck: &[Card], config: &DrawConfig) -> Result<DrawReport, Error> {
    if config.runs == 0
        || config.runs > 1_000_000
        || config.draws > 1000
        || config.targets.len() > 1000
    {
        return Err(Error::Configuration(
            "invalid draw simulation bounds".into(),
        ));
    }
    config.mulligan.validate()?;
    let mut rng = SimulationRng::seeded(config.seed);
    let targets: BTreeSet<_> = config.targets.iter().copied().collect();
    let mut histogram = vec![0; deck.len().min(7 + usize::from(config.draws)) + 1];
    let (mut opening, mut mulligans, mut forced) = (0u64, 0u64, 0);
    for _ in 0..config.runs {
        let hand = sample_hand(deck, &config.mulligan, &mut rng)?;
        let hits = hand
            .kept
            .iter()
            .chain(hand.draws(usize::from(config.draws)))
            .filter(|i| targets.contains(&deck[**i].id))
            .count();
        histogram[hits] += 1;
        opening += hand.kept.len() as u64;
        mulligans += u64::from(hand.mulligans);
        forced += u32::from(hand.forced_keep);
    }
    Ok(DrawReport {
        runs: config.runs,
        seed: config.seed,
        replay_version: REPLAY_VERSION,
        histogram,
        mean_opening_size: opening as f64 / f64::from(config.runs),
        mean_mulligans: mulligans as f64 / f64::from(config.runs),
        forced_keeps: forced,
    })
}
