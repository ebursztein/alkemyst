use super::{resources::Context, *};
use landlord::{SimulationRng, mana::ManaCost};
pub(super) struct CastOption {
    pub card: usize,
    pub spell: SpellSpec,
    pub cost: ManaCost,
    adventure: bool,
    from_exile: bool,
}
impl Trial {
    fn tutor_playable(&self, spell: &SpellSpec) -> bool {
        spell.permanent
            || spell.tutor.as_ref().is_none_or(|t| {
                matches!(t.scope, SearchScope::Top(_)) || self.tutor_target(t).is_some()
            })
    }
    pub(super) fn cast_options(&self) -> Vec<CastOption> {
        let mut options = vec![];
        for (card, from_exile) in self
            .hand
            .iter()
            .map(|i| (*i, false))
            .chain(self.adventure_exile.iter().map(|i| (*i, true)))
        {
            let spec = &self.deck[card];
            let Kind::Spell(spell) = &spec.kind else {
                continue;
            };
            if !self.duplicate_legend(card) && self.tutor_playable(spell) {
                options.push(CastOption {
                    card,
                    spell: spell.clone(),
                    cost: spec.cost.clone(),
                    adventure: false,
                    from_exile,
                });
            }
            if !from_exile && let Some(adventure) = &spec.adventure {
                let spell = SpellSpec {
                    tutor: Some(adventure.tutor.clone()),
                    ..Default::default()
                };
                if self.tutor_playable(&spell) {
                    options.push(CastOption {
                        card,
                        spell,
                        cost: adventure.cost.clone(),
                        adventure: true,
                        from_exile: false,
                    });
                }
            }
        }
        options
    }
    pub(super) fn cast_spells(&mut self, rng: &mut SimulationRng) {
        while !self.decked {
            let option = self
                .cast_options()
                .into_iter()
                .filter(|o| self.can_pay(&o.cost, Context::Spell(&o.spell)))
                .min_by_key(|o| {
                    (
                        std::cmp::Reverse(self.deck[o.card].keep_priority),
                        o.cost.mana_value(),
                        !o.adventure,
                        self.deck[o.card].id,
                        o.card,
                    )
                });
            let Some(option) = option else {
                break;
            };
            if !self.pay(&option.cost, Context::Spell(&option.spell)) {
                break;
            }
            if option.from_exile {
                self.adventure_exile.retain(|i| *i != option.card);
            } else {
                self.hand.retain(|i| *i != option.card);
            }
            self.spells_cast += 1;
            if option.spell.permanent {
                self.enter(option.card, false);
            } else if !option.adventure {
                self.graveyard.push(option.card);
            }
            if let Some(tutor) = option.spell.tutor {
                self.tutors_cast += 1;
                self.resolve_searches(std::collections::VecDeque::from([tutor]), rng);
            }
            if option.adventure && !self.decked {
                self.adventure_exile.push(option.card);
            }
        }
    }
}
