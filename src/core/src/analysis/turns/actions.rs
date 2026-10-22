use super::{resources::Context, *};
use landlord::SimulationRng;

impl Trial {
    pub(super) fn tutor_target(&self, tutor: &TutorSpec) -> Option<usize> {
        self.library
            .iter()
            .copied()
            .filter(|i| {
                matches!(&self.deck[*i].kind,Kind::Spell(s) if s.equipment)
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
    pub(super) fn cast_spells(&mut self, rng: &mut SimulationRng) {
        loop {
            let card = self
                .hand
                .iter()
                .copied()
                .filter(|i| {
                    let Kind::Spell(s) = &self.deck[*i].kind else {
                        return false;
                    };
                    !self.duplicate_legend(*i)
                        && (s.permanent
                            || s.tutor
                                .as_ref()
                                .is_none_or(|t| self.tutor_target(t).is_some()))
                        && self.can_pay(&self.deck[*i].cost, Context::Spell(s))
                })
                .min_by_key(|i| {
                    (
                        std::cmp::Reverse(self.deck[*i].keep_priority),
                        self.deck[*i].cost.mana_value(),
                        self.deck[*i].id,
                        *i,
                    )
                });
            let Some(card) = card else {
                break;
            };
            let Kind::Spell(spell) = self.deck[card].kind.clone() else {
                unreachable!()
            };
            let cost = self.deck[card].cost.clone();
            self.pay(&cost, Context::Spell(&spell));
            self.hand.retain(|i| *i != card);
            self.spells_cast += 1;
            if spell.permanent {
                self.enter(card, false);
            }
            if let Some(tutor) = spell.tutor {
                self.tutors_cast += 1;
                // A cast permanent both resolves as a spell and enters. Its
                // explicitly selected tutor event therefore fires exactly once.
                self.resolve_searches(std::collections::VecDeque::from([tutor]), rng);
            }
        }
    }
    fn resolve_searches(
        &mut self,
        mut pending: std::collections::VecDeque<TutorSpec>,
        rng: &mut SimulationRng,
    ) {
        while let Some(tutor) = pending.pop_front() {
            self.searches_resolved += 1;
            if let Some(target) = self.tutor_target(&tutor) {
                self.library.retain(|i| *i != target);
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
            rng.shuffle(self.library.make_contiguous());
        }
    }
    pub(super) fn attach_equipment(&mut self) {
        loop {
            let Some(target) = self
                .battlefield
                .iter()
                .filter(|p| matches!(&self.deck[p.card].kind,Kind::Spell(s) if s.creature))
                .min_by_key(|p| {
                    (
                        !self.ready(p.card),
                        std::cmp::Reverse(self.deck[p.card].keep_priority),
                        p.entered,
                        p.card,
                    )
                })
                .map(|p| p.card)
            else {
                return;
            };
            let equipment = self
                .battlefield
                .iter()
                .filter_map(|p| {
                    let Kind::Spell(s) = &self.deck[p.card].kind else {
                        return None;
                    };
                    let cost = s.equip_cost.as_ref()?;
                    (s.equipment && p.attached_to.is_none() && self.can_pay(cost, Context::Equip))
                        .then_some((p.card, cost.clone()))
                })
                .min_by_key(|(card, cost)| {
                    (
                        cost.mana_value(),
                        std::cmp::Reverse(self.deck[*card].keep_priority),
                        *card,
                    )
                });
            let Some((card, cost)) = equipment else {
                return;
            };
            self.pay(&cost, Context::Equip);
            self.battlefield
                .iter_mut()
                .find(|p| p.card == card)
                .expect("equipment is present")
                .attached_to = Some(target);
        }
    }
}
