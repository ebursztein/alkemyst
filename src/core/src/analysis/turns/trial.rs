use super::*;
use landlord::{Error, SimulationRng};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::Arc,
};

pub(super) struct Permanent {
    pub card: usize,
    pub entered: u8,
    pub tapped: bool,
    pub attached_to: Option<usize>,
}
/// Explicit priority-greedy play, then equipment attachment. It assumes no
/// opponent interaction and does not claim to find the optimal play sequence.
pub struct Trial {
    pub(super) deck: Arc<[CardSpec]>,
    pub(super) hand: Vec<usize>,
    pub(super) library: VecDeque<usize>,
    pub(super) battlefield: Vec<Permanent>,
    pub(super) turn: u8,
    pub(super) life: u8,
    pub(super) on_the_play: bool,
    pub(super) decked: bool,
    pub(super) spells_cast: usize,
    pub(super) tutors_cast: usize,
    pub(super) searches_resolved: usize,
}
impl Trial {
    pub fn new(
        deck: Arc<[CardSpec]>,
        opening: Vec<usize>,
        library: Vec<usize>,
        on_the_play: bool,
    ) -> Result<Self, Error> {
        if deck.is_empty()
            || deck.len() > 1000
            || opening.len() > 7
            || opening.len() + library.len() != deck.len()
        {
            return Err(Error::Configuration(
                "invalid physical deck/hand sizes".into(),
            ));
        }
        let indices: BTreeSet<_> = opening.iter().chain(&library).copied().collect();
        if indices.len() != deck.len() || indices.last() != Some(&(deck.len() - 1)) {
            return Err(Error::Configuration(
                "hand/library must partition physical deck".into(),
            ));
        }
        let mut identities = BTreeMap::new();
        for card in deck.iter() {
            if let Some(previous) = identities.insert(card.id, card)
                && previous != card
            {
                return Err(Error::Configuration(
                    "conflicting definitions for one card identity".into(),
                ));
            }
            match &card.kind {
                Kind::Spell(s)
                    if (s.equipment
                        && (!s.artifact
                            || !s.permanent
                            || s.creature
                            || s.equip_cost.is_none()))
                        || (s.equip_cost.is_some() != s.equipment)
                        || ((s.artifact || s.creature) && !s.permanent)
                        || (s.haste && !s.creature)
                        || (!s.permanent
                            && s.tutor
                                .as_ref()
                                .is_some_and(|t| t.timing == TutorTiming::EnterBattlefield))
                        || (card.legendary && !s.permanent) =>
                {
                    return Err(Error::Configuration(
                        "unsupported or inconsistent spell properties".into(),
                    ));
                }
                Kind::Land(l)
                    if card.cost.mana_value() != 0
                        || l.basic_types > 31
                        || matches!(l.enters,Enters::HasBasicType(mask) if mask==0 || mask>31) =>
                {
                    return Err(Error::Configuration("invalid modeled land".into()));
                }
                _ => {}
            }
        }
        Ok(Self::from_sampled(
            deck,
            landlord::OpeningHand {
                kept: opening,
                library,
                bottomed: vec![],
                mulligans: 0,
                forced_keep: false,
            },
            on_the_play,
        ))
    }
    /// Only the validated runner supplies hands produced by landlord here.
    pub(crate) fn from_sampled(
        deck: Arc<[CardSpec]>,
        hand: landlord::OpeningHand,
        on_the_play: bool,
    ) -> Self {
        Self {
            deck,
            hand: hand.kept,
            library: hand.library.into(),
            battlefield: vec![],
            turn: 0,
            life: 20,
            on_the_play,
            decked: false,
            spells_cast: 0,
            tutors_cast: 0,
            searches_resolved: 0,
        }
    }

    pub fn next_turn(&mut self, rng: &mut SimulationRng) -> Result<TurnSnapshot, Error> {
        if self.turn >= 30 {
            return Err(Error::Configuration(
                "turn model is bounded to 30 turns".into(),
            ));
        }
        self.turn += 1;
        self.spells_cast = 0;
        self.tutors_cast = 0;
        self.searches_resolved = 0;
        for permanent in &mut self.battlefield {
            permanent.tapped = false;
        }
        if !(self.decked || self.on_the_play && self.turn == 1) {
            if let Some(card) = self.library.pop_front() {
                self.hand.push(card);
            } else {
                self.decked = true;
            }
        }
        if !self.decked {
            self.play_land();
            self.cast_spells(rng);
            self.attach_equipment();
        }
        Ok(self.snapshot())
    }
    pub(super) fn duplicate_legend(&self, card: usize) -> bool {
        self.deck[card].legendary
            && self
                .battlefield
                .iter()
                .any(|p| self.deck[p.card].id == self.deck[card].id)
    }
    pub(super) fn enter(&mut self, card: usize, tapped: bool) {
        self.battlefield.push(Permanent {
            card,
            entered: self.turn,
            tapped,
            attached_to: None,
        });
    }
    pub(super) fn ready(&self, card: usize) -> bool {
        self.battlefield.iter().find(|p|p.card==card).is_some_and(|p| matches!(&self.deck[card].kind,Kind::Spell(s) if s.creature && (s.haste || p.entered<self.turn)))
    }
    fn snapshot(&self) -> TurnSnapshot {
        let equipment: Vec<_> = self
            .battlefield
            .iter()
            .filter(|p| matches!(&self.deck[p.card].kind,Kind::Spell(s) if s.equipment))
            .collect();
        let mut seen = BTreeMap::<u32, usize>::new();
        for card in self
            .hand
            .iter()
            .copied()
            .chain(self.battlefield.iter().map(|p| p.card))
        {
            if self.deck[card].legendary {
                *seen.entry(self.deck[card].id).or_default() += 1;
            }
        }
        TurnSnapshot {
            turn: self.turn,
            life: self.life,
            lands: self
                .battlefield
                .iter()
                .filter(|p| matches!(self.deck[p.card].kind, Kind::Land(_)))
                .count(),
            equipment: equipment.len(),
            attached: equipment.iter().filter(|p| p.attached_to.is_some()).count(),
            attack_ready: equipment
                .iter()
                .filter(|p| p.attached_to.is_some_and(|card| self.ready(card)))
                .count(),
            legendary_bricks: seen.values().map(|n| n.saturating_sub(1)).sum(),
            spells_cast: self.spells_cast,
            tutors_cast: self.tutors_cast,
            searches_resolved: self.searches_resolved,
            decked: self.decked,
        }
    }
}
