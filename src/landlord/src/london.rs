//! London mulligans derived from landlord's draw-seven/keep/bottom workflow.
//! Full libraries replace upstream's shortened swap approximation, so bottomed
//! cards cannot be drawn before the undrawn library has been exhausted.
use crate::{Card, Error, SimulationRng};

#[derive(Debug, Clone)]
pub struct London {
    pub minimum_keep: u8,
    pub force_mulligans: u8,
    pub min_lands: u8,
    pub max_lands: u8,
    /// Each identity must occur at least once in an acceptable retained hand.
    pub required_ids: Vec<u32>,
}
impl London {
    pub fn never() -> Self {
        Self {
            minimum_keep: 7,
            force_mulligans: 0,
            min_lands: 0,
            max_lands: 7,
            required_ids: vec![],
        }
    }
    pub(crate) fn validate(&self) -> Result<(), Error> {
        if self.minimum_keep > 7
            || self.force_mulligans > 7 - self.minimum_keep
            || self.min_lands > self.max_lands
            || self.max_lands > 7
            || self.required_ids.len() > 7
        {
            return Err(Error::Configuration("invalid London policy".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpeningHand {
    /// Indices into the supplied deck, preserving individual physical copies.
    pub kept: Vec<usize>,
    pub library: Vec<usize>,
    pub bottomed: Vec<usize>,
    pub mulligans: u8,
    pub forced_keep: bool,
}
impl OpeningHand {
    pub fn draws(&self, count: usize) -> &[usize] {
        &self.library[..count.min(self.library.len())]
    }
}

pub fn sample_hand(
    deck: &[Card],
    policy: &London,
    rng: &mut SimulationRng,
) -> Result<OpeningHand, Error> {
    policy.validate()?;
    if deck.is_empty() || deck.len() > 1000 {
        return Err(Error::Configuration(
            "deck must contain 1..=1000 cards".into(),
        ));
    }
    let starting = deck.len().min(7);
    let floor = usize::from(policy.minimum_keep).min(starting);
    let mut shuffled: Vec<_> = (0..deck.len()).collect();
    for round in 0..=starting - floor {
        rng.shuffle(&mut shuffled);
        let keep_count = starting - round;
        let mut candidates = shuffled[..starting].to_vec();
        candidates.sort_by_key(|i| std::cmp::Reverse(deck[*i].keep_priority));
        let kept = choose_kept(deck, &candidates, keep_count, policy);
        let lands = kept.iter().filter(|i| deck[**i].is_land).count();
        let acceptable = (usize::from(policy.min_lands)..=usize::from(policy.max_lands))
            .contains(&lands)
            && policy
                .required_ids
                .iter()
                .all(|id| kept.iter().any(|i| deck[*i].id == *id));
        let forced = round == starting - floor;
        if forced || (round >= usize::from(policy.force_mulligans) && acceptable) {
            let bottomed: Vec<_> = candidates
                .into_iter()
                .filter(|i| !kept.contains(i))
                .collect();
            let library = shuffled[starting..]
                .iter()
                .chain(&bottomed)
                .copied()
                .collect();
            return Ok(OpeningHand {
                kept,
                library,
                bottomed,
                mulligans: round as u8,
                forced_keep: !acceptable,
            });
        }
    }
    unreachable!("London loop always keeps at its floor")
}

fn choose_kept(deck: &[Card], candidates: &[usize], count: usize, policy: &London) -> Vec<usize> {
    let mut kept = Vec::with_capacity(count);
    for id in &policy.required_ids {
        if let Some(i) = candidates.iter().find(|i| deck[**i].id == *id)
            && kept.len() < count
            && !kept.contains(i)
        {
            kept.push(*i);
        }
    }
    for i in candidates {
        let lands = kept.iter().filter(|i| deck[**i].is_land).count();
        if lands < usize::from(policy.min_lands)
            && deck[*i].is_land
            && kept.len() < count
            && !kept.contains(i)
        {
            kept.push(*i);
        }
    }
    // Respect the upper land bound before falling back to an unavoidable
    // forced keep. Required identities and minimum lands already have priority.
    for respect_max in [true, false] {
        for i in candidates {
            let lands = kept.iter().filter(|i| deck[**i].is_land).count();
            if respect_max && deck[*i].is_land && lands >= usize::from(policy.max_lands) {
                continue;
            }
            if kept.len() < count && !kept.contains(i) {
                kept.push(*i);
            }
        }
    }
    kept
}
