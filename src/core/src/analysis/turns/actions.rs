use super::*;
use landlord::SimulationRng;

impl Trial {
    pub(super) fn tutor_target(&self, tutor: &TutorSpec) -> Option<usize> {
        self.library
            .iter()
            .take(match tutor.scope {
                SearchScope::Library => self.library.len(),
                SearchScope::Top(n) => usize::from(n),
            })
            .copied()
            .filter(|i| {
                (matches!(&self.deck[*i].kind,Kind::Spell(s) if s.equipment)
                    || (tutor.types == SearchTypes::EquipmentOrDwarf && self.deck[*i].tags.dwarf))
                    && tutor
                        .max_mana_value
                        .is_none_or(|max| self.deck[*i].printed_mana_value <= max)
                    && !self.duplicate_legend(*i)
                    && !(self.deck[*i].legendary
                        && self
                            .hand
                            .iter()
                            .any(|h| self.deck[*h].id == self.deck[*i].id))
            })
            .min_by_key(|i| {
                (
                    std::cmp::Reverse(self.deck[*i].keep_priority),
                    self.deck[*i].cost.mana_value(),
                    self.deck[*i].id,
                    *i,
                )
            })
    }
    pub(super) fn resolve_searches(
        &mut self,
        mut pending: std::collections::VecDeque<TutorSpec>,
        rng: &mut SimulationRng,
    ) {
        while !self.decked {
            let Some(tutor) = pending.pop_front() else {
                break;
            };
            self.searches_resolved += 1;
            let target = self.tutor_target(&tutor);
            match tutor.scope {
                SearchScope::Library => {
                    if let Some(target) = target {
                        self.library.retain(|i| *i != target);
                    }
                    rng.shuffle(self.library.make_contiguous());
                }
                SearchScope::Top(n) => {
                    let count = usize::from(n).min(self.library.len());
                    let mut bottom = self
                        .library
                        .drain(..count)
                        .filter(|i| Some(*i) != target)
                        .collect::<Vec<_>>();
                    rng.shuffle(&mut bottom);
                    self.library.extend(bottom);
                }
            }
            // Entry triggers resolve after the search/selection instruction has
            // finished changing the library, including shuffle or bottoming.
            if let Some(target) = target {
                match tutor.destination {
                    Destination::Hand => self.hand.push(target),
                    Destination::Battlefield => {
                        self.enter(target, false);
                        if let Kind::Spell(s) = &self.deck[target].kind
                            && let Some(effect) = &s.tutor
                            && effect.timing == TutorTiming::EnterBattlefield
                        {
                            pending.push_back(effect.clone());
                        }
                    }
                }
            }
        }
    }
}
