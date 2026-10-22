use super::*;
impl Trial {
    pub(super) fn draw_card(&mut self) {
        if let Some(card) = self.library.pop_front() {
            self.hand.push(card);
            self.cards_drawn += 1;
        } else {
            self.decked = true;
        }
    }
    fn discard_choice(&self, policy: DiscardPolicy) -> Option<usize> {
        self.hand
            .iter()
            .copied()
            .filter(|i| {
                policy == DiscardPolicy::LowestPriority
                    || matches!(self.deck[*i].kind, Kind::Ignored)
                    || (self.deck[*i].adventure.is_none()
                        && (self.duplicate_legend(*i)
                            || (self.deck[*i].legendary
                                && self
                                    .hand
                                    .iter()
                                    .filter(|j| self.deck[**j].id == self.deck[*i].id)
                                    .count()
                                    > 1)))
            })
            .min_by_key(|i| {
                (
                    self.deck[*i].keep_priority,
                    std::cmp::Reverse(self.deck[*i].cost.mana_value()),
                    *i,
                )
            })
    }
    pub(super) fn entry_draws(&mut self, incoming: usize) {
        let card = &self.deck[incoming];
        if !(card.tags.dwarf || matches!(&card.kind,Kind::Spell(s) if s.equipment)) {
            return;
        }
        let triggers = self
            .battlefield
            .iter()
            .enumerate()
            .filter_map(|(index, p)| {
                let Kind::Spell(s) = &self.deck[p.card].kind else {
                    return None;
                };
                let draw = s.entry_draw.as_ref()?;
                ((draw.allow_self || p.card != incoming)
                    && !(draw.once_per_turn && p.draw_triggered_turn == self.turn))
                    .then_some((index, draw.clone()))
            })
            .collect::<Vec<_>>();
        // Owned entry draws resolve oldest source first, before the incoming
        // permanent's search. This is an explicit controller ordering policy.
        for (index, draw) in triggers {
            if self.decked {
                break;
            }
            self.battlefield[index].draw_triggered_turn = self.turn;
            if draw.discard != DiscardPolicy::None {
                let Some(card) = self.discard_choice(draw.discard) else {
                    continue;
                };
                self.hand.retain(|i| *i != card);
                self.graveyard.push(card);
                self.cards_discarded += 1;
            }
            self.draw_card();
        }
    }
}
