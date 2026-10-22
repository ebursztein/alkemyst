use super::{resources::Context, *};
use landlord::mana::ManaCost;
struct Choice {
    card: usize,
    target: usize,
    cost: ManaCost,
    base: usize,
    ready: bool,
}
impl Trial {
    fn free_equip(&self) -> bool {
        self.enduring_story
            && !self.first_equip_used
            && self
                .battlefield
                .iter()
                .any(|p| matches!(&self.deck[p.card].kind,Kind::Spell(s) if s.story_free_equip))
    }
    fn target_cost(&self, equipment: &SpellSpec, target: usize) -> Option<ManaCost> {
        let card = &self.deck[target];
        if equipment.worthy_only
            && (!card.legendary || card.tags.villain || card.tags.colors & (1 | 8) == 0)
        {
            return None;
        }
        let cost = equipment.equip_cost.as_ref()?;
        let Kind::Spell(target_spell) = &card.kind else {
            return None;
        };
        let discount = usize::from(target_spell.target_equip_reduction)
            + if equipment.equip_reduction_per_color {
                card.tags.colors.count_ones() as usize
            } else {
                0
            };
        Some(cost.reduced_generic(discount))
    }
    pub(super) fn attach_equipment(&mut self) {
        loop {
            let free = self.free_equip();
            let mut choices = vec![];
            for eq in &self.battlefield {
                let Kind::Spell(s) = &self.deck[eq.card].kind else {
                    continue;
                };
                if !s.equipment || eq.attached_to.is_some() {
                    continue;
                }
                for (target_index, target) in self.battlefield.iter().enumerate() {
                    if !matches!(&self.deck[target.card].kind,Kind::Spell(c) if c.creature) {
                        continue;
                    }
                    let Some(normal) = self.target_cost(s, target.card) else {
                        continue;
                    };
                    let cost = if free {
                        ManaCost::default()
                    } else {
                        normal.clone()
                    };
                    let Some(payment) = self.payment_with_land(&cost, Context::Equip, None) else {
                        continue;
                    };
                    choices.push(Choice {
                        card: eq.card,
                        target: target.card,
                        cost,
                        base: normal.mana_value(),
                        ready: self.ready(target.card) && !payment.contains(&target_index),
                    });
                }
            }
            let choice = choices.into_iter().min_by_key(|c| {
                (
                    !c.ready,
                    std::cmp::Reverse(if free { c.base } else { 0 }),
                    c.cost.mana_value(),
                    std::cmp::Reverse(self.deck[c.card].keep_priority),
                    std::cmp::Reverse(self.deck[c.target].keep_priority),
                    c.card,
                    c.target,
                )
            });
            let Some(choice) = choice else {
                return;
            };
            self.pay(&choice.cost, Context::Equip);
            self.first_equip_used = true;
            self.battlefield
                .iter_mut()
                .find(|p| p.card == choice.card)
                .expect("equipment is present")
                .attached_to = Some(choice.target);
        }
    }
}
