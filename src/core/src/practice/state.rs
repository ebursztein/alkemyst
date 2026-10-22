use super::*;
use landlord::SimulationRng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, VecDeque};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Placed {
    pub card: u16,
    pub face: u8,
    pub tapped: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct State {
    pub phase: PracticePhase,
    pub mulligans: u8,
    pub turn: u8,
    pub land_played: bool,
    pub decked: bool,
    pub hand: Vec<u16>,
    pub library: VecDeque<u16>,
    pub battlefield: Vec<Placed>,
    pub graveyard: Vec<u16>,
    pub exile: Vec<u16>,
}
impl State {
    pub fn deal(total: usize, seed: u64, mulligans: u8) -> Self {
        let mut library = (0..total as u16).collect::<Vec<_>>();
        let mut rng = SimulationRng::seeded(seed);
        for _ in 0..=mulligans {
            rng.shuffle(&mut library);
        }
        let opening = total.min(7);
        let hand = library.drain(..opening).collect();
        Self {
            phase: PracticePhase::Opening,
            mulligans,
            turn: 0,
            land_played: false,
            decked: false,
            hand,
            library: library.into(),
            battlefield: vec![],
            graveyard: vec![],
            exile: vec![],
        }
    }
    pub fn draw(&mut self) {
        if let Some(id) = self.library.pop_front() {
            self.hand.push(id);
        } else {
            self.decked = true;
            self.phase = PracticePhase::Finished;
        }
    }
    pub fn apply(
        &mut self,
        action: &PracticeAction,
        cards: &[PracticeCard],
        seed: u64,
        on_the_play: bool,
    ) -> Result<(), Error> {
        if self.phase == PracticePhase::Finished {
            return Err(Error::Invalid("session has finished"));
        }
        match action {
            PracticeAction::Finish => {
                self.phase = PracticePhase::Finished;
                return Ok(());
            }
            PracticeAction::Mulligan => {
                if self.phase != PracticePhase::Opening
                    || usize::from(self.mulligans) >= cards.len().min(7)
                {
                    return Err(Error::Invalid("mulligan unavailable"));
                }
                *self = Self::deal(cards.len(), seed, self.mulligans + 1);
                return Ok(());
            }
            PracticeAction::Keep { bottom } => {
                if self.phase != PracticePhase::Opening
                    || bottom.len() != usize::from(self.mulligans)
                    || bottom.iter().collect::<BTreeSet<_>>().len() != bottom.len()
                    || bottom.iter().any(|id| !self.hand.contains(id))
                {
                    return Err(Error::Invalid(
                        "choose exactly the owed distinct cards from the opening hand",
                    ));
                }
                self.hand.retain(|id| !bottom.contains(id));
                self.library.extend(bottom.iter().copied());
                self.phase = PracticePhase::Main;
                self.turn = 1;
                if !on_the_play {
                    self.draw();
                }
                return Ok(());
            }
            _ => {}
        }
        if self.phase != PracticePhase::Main {
            return Err(Error::Invalid("keep an opening hand first"));
        }
        match action {
            PracticeAction::Draw => self.draw(),
            PracticeAction::NextTurn => {
                if self.turn >= 30 {
                    return Err(Error::Invalid("practice is bounded to thirty turns"));
                }
                self.turn += 1;
                self.land_played = false;
                for p in &mut self.battlefield {
                    p.tapped = false;
                }
                self.draw();
            }
            PracticeAction::PlayLand { card, face } => {
                if self.land_played
                    || !cards
                        .get(usize::from(*card))
                        .is_some_and(|c| super::cards::is_land(c, *face))
                {
                    return Err(Error::Invalid(
                        "land play unavailable for this card or turn",
                    ));
                }
                self.play(*card, *face, PracticeZone::Battlefield, cards)?;
                self.land_played = true;
            }
            PracticeAction::Play {
                card,
                face,
                destination,
            } => {
                if cards
                    .get(usize::from(*card))
                    .is_some_and(|c| super::cards::is_land(c, *face))
                {
                    return Err(Error::Invalid("use Play Land for land cards"));
                }
                self.play(*card, *face, *destination, cards)?;
            }
            PracticeAction::Move { card, destination } => self.move_card(*card, *destination)?,
            PracticeAction::Tap { card, tapped } => {
                self.battlefield
                    .iter_mut()
                    .find(|p| p.card == *card)
                    .ok_or(Error::Invalid("card is not on the battlefield"))?
                    .tapped = *tapped;
            }
            _ => unreachable!("opening and finish actions return earlier"),
        }
        Ok(())
    }
}
