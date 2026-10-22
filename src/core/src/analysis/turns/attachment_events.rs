use super::*;
impl Trial {
    pub(super) fn entry_attachments(&mut self, incoming: usize) {
        let Kind::Spell(entered) = &self.deck[incoming].kind else {
            return;
        };
        if entered.creature {
            let value = self.deck[incoming].printed_mana_value;
            for p in &mut self.battlefield {
                if let Kind::Spell(equipment) = &self.deck[p.card].kind
                    && equipment
                        .auto_attach_min_value
                        .is_some_and(|min| value >= min)
                {
                    p.attached_to = Some(incoming);
                }
            }
        }
        // With multiple owned entry effects, resolve specific attachment first,
        // then the incoming permanent's optional "attach any number" effect.
        if entered.attach_on_enter {
            let target = self
                .battlefield
                .iter()
                .filter(|p| matches!(&self.deck[p.card].kind,Kind::Spell(s) if s.creature))
                .min_by_key(|p| {
                    (
                        !self.ready(p.card),
                        std::cmp::Reverse(self.deck[p.card].keep_priority),
                        p.card,
                    )
                })
                .map(|p| p.card);
            if let Some(target) = target {
                for p in &mut self.battlefield {
                    if matches!(&self.deck[p.card].kind,Kind::Spell(s) if s.equipment) {
                        p.attached_to = Some(target);
                    }
                }
            }
        }
    }
}
