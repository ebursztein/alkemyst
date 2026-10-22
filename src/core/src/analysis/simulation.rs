use super::{Estimate, turns::*};
use landlord::{Card, Error, London, REPLAY_VERSION, SimulationRng, sample_hand};
use std::sync::Arc;

pub const MODEL_VERSION: &str = "equipment-goldfish-v2";
pub const POLICY: &str = "priority-greedy-v2";

#[derive(Clone)]
pub struct Scenario {
    pub deck: Arc<[CardSpec]>,
    pub turns: u8,
    pub on_the_play: bool,
    pub mulligan: London,
    pub goal: Goal,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct TurnEstimate {
    pub turn: u8,
    pub equipment: Estimate,
    pub attached: Estimate,
    pub attack_ready: Estimate,
    pub joint: Estimate,
    pub mean_equipment: f64,
    pub mean_creatures: f64,
    pub mean_cards_drawn: f64,
    pub mean_cards_discarded: f64,
    pub story_samples: u32,
    pub mean_legendary_bricks: f64,
    pub mean_life: f64,
    pub mean_tutors_cast: f64,
    pub decked: u32,
}
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct SimulationReport {
    pub requested_runs: u32,
    pub completed_runs: u32,
    #[serde(with = "super::seed")]
    #[schema(value_type=String,pattern="^[0-9a-f]{16}$")]
    pub seed: u64,
    pub replay_version: String,
    pub model_version: String,
    pub policy: String,
    pub goal: Goal,
    pub mean_opening_size: f64,
    pub mean_mulligans: f64,
    pub forced_keeps: u32,
    pub turns: Vec<TurnEstimate>,
}
#[derive(Default)]
struct Totals {
    equipment: u32,
    attached: u32,
    ready: u32,
    joint: u32,
    decked: u32,
    equipment_sum: u64,
    creatures_sum: u64,
    drawn_sum: u64,
    discarded_sum: u64,
    story: u32,
    bricks_sum: u64,
    life_sum: u64,
    tutors_sum: u64,
}
/// The job worker checks cancellation between bounded advances. The RNG stream
/// survives batch boundaries, so scheduling and progress reads cannot alter results.
pub struct BattlefieldRunner {
    scenario: Scenario,
    sampling_deck: Vec<Card>,
    runs: u32,
    completed: u32,
    seed: u64,
    rng: SimulationRng,
    totals: Vec<Totals>,
    opening_sum: u64,
    mulligan_sum: u64,
    forced_keeps: u32,
}
impl BattlefieldRunner {
    pub fn new(scenario: Scenario, runs: u32, seed: u64) -> Result<Self, Error> {
        if runs == 0
            || runs > 1_000_000
            || scenario.turns == 0
            || scenario.turns > 30
            || [
                scenario.goal.equipment,
                scenario.goal.attached,
                scenario.goal.attack_ready,
            ]
            .iter()
            .any(|n| *n > 1000)
        {
            return Err(Error::Configuration(
                "invalid battlefield simulation bounds".into(),
            ));
        }
        // Validate the compiled deck once. Every trial's hand partition then comes
        // from the seeded landlord sampler, which preserves all physical copies.
        Trial::new(
            scenario.deck.clone(),
            vec![],
            (0..scenario.deck.len()).collect(),
            scenario.on_the_play,
        )?;
        let sampling_deck = scenario
            .deck
            .iter()
            .map(|c| Card {
                id: c.id,
                is_land: matches!(c.kind, Kind::Land(_)),
                keep_priority: c.keep_priority,
            })
            .collect::<Vec<_>>();
        // Also validate the London policy before exposing a job as runnable.
        sample_hand(
            &sampling_deck,
            &scenario.mulligan,
            &mut SimulationRng::seeded(seed),
        )?;
        let totals = (0..scenario.turns).map(|_| Totals::default()).collect();
        Ok(Self {
            scenario,
            sampling_deck,
            runs,
            completed: 0,
            seed,
            rng: SimulationRng::seeded(seed),
            totals,
            opening_sum: 0,
            mulligan_sum: 0,
            forced_keeps: 0,
        })
    }
    pub fn finished(&self) -> bool {
        self.completed == self.runs
    }
    pub fn completed(&self) -> u32 {
        self.completed
    }
    pub fn advance(&mut self, batch: u32) -> Result<u32, Error> {
        let count = batch.min(1024).min(self.runs - self.completed);
        for _ in 0..count {
            let hand = sample_hand(&self.sampling_deck, &self.scenario.mulligan, &mut self.rng)?;
            self.opening_sum += hand.kept.len() as u64;
            self.mulligan_sum += u64::from(hand.mulligans);
            self.forced_keeps += u32::from(hand.forced_keep);
            let mut trial =
                Trial::from_sampled(self.scenario.deck.clone(), hand, self.scenario.on_the_play);
            for total in &mut self.totals {
                let frame = trial.next_turn(&mut self.rng)?;
                total.equipment +=
                    u32::from(!frame.decked && frame.equipment >= self.scenario.goal.equipment);
                total.attached +=
                    u32::from(!frame.decked && frame.attached >= self.scenario.goal.attached);
                total.ready += u32::from(
                    !frame.decked && frame.attack_ready >= self.scenario.goal.attack_ready,
                );
                total.joint += u32::from(self.scenario.goal.met(&frame));
                total.decked += u32::from(frame.decked);
                total.equipment_sum += frame.equipment as u64;
                total.creatures_sum += frame.creatures as u64;
                total.drawn_sum += frame.cards_drawn as u64;
                total.discarded_sum += frame.cards_discarded as u64;
                total.story += u32::from(frame.enduring_story);
                total.bricks_sum += frame.legendary_bricks as u64;
                total.life_sum += u64::from(frame.life);
                total.tutors_sum += frame.tutors_cast as u64;
            }
            self.completed += 1;
        }
        Ok(count)
    }
    pub fn report(&self) -> Option<SimulationReport> {
        if self.completed == 0 {
            return None;
        }
        let n = f64::from(self.completed);
        let turns = self
            .totals
            .iter()
            .enumerate()
            .map(|(i, t)| TurnEstimate {
                turn: (i + 1) as u8,
                equipment: Estimate::new(t.equipment, self.completed),
                attached: Estimate::new(t.attached, self.completed),
                attack_ready: Estimate::new(t.ready, self.completed),
                joint: Estimate::new(t.joint, self.completed),
                mean_equipment: t.equipment_sum as f64 / n,
                mean_creatures: t.creatures_sum as f64 / n,
                mean_cards_drawn: t.drawn_sum as f64 / n,
                mean_cards_discarded: t.discarded_sum as f64 / n,
                story_samples: t.story,
                mean_legendary_bricks: t.bricks_sum as f64 / n,
                mean_life: t.life_sum as f64 / n,
                mean_tutors_cast: t.tutors_sum as f64 / n,
                decked: t.decked,
            })
            .collect();
        Some(SimulationReport {
            requested_runs: self.runs,
            completed_runs: self.completed,
            seed: self.seed,
            replay_version: REPLAY_VERSION.into(),
            model_version: MODEL_VERSION.into(),
            policy: POLICY.into(),
            goal: self.scenario.goal,
            mean_opening_size: self.opening_sum as f64 / n,
            mean_mulligans: self.mulligan_sum as f64 / n,
            forced_keeps: self.forced_keeps,
            turns,
        })
    }
}
