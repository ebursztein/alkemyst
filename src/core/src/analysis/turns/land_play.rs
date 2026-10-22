use super::{resources::Context, *};
impl Trial {
    pub(super) fn entry(&self, land: &LandSpec) -> (bool, u8) {
        let others = self
            .battlefield
            .iter()
            .filter_map(|p| match &self.deck[p.card].kind {
                Kind::Land(l) => Some(l),
                _ => None,
            })
            .collect::<Vec<_>>();
        let untapped = match land.enters {
            Enters::Untapped => true,
            Enters::Tapped => false,
            Enters::OtherLandsAtLeast(n) => others.len() >= usize::from(n),
            Enters::OtherLandsAtMost(n) => others.len() <= usize::from(n),
            Enters::HasBasicType(mask) => others.iter().any(|l| l.basic_types & mask != 0),
            Enters::PayLife(n) => self.life > n,
            Enters::ControlsEquipment => self.condition(Condition::ControlsEquipment, None),
        };
        let paid = if untapped {
            if let Enters::PayLife(n) = land.enters {
                n
            } else {
                0
            }
        } else {
            0
        };
        (!untapped, paid)
    }
    fn land_score(
        &self,
        card: usize,
    ) -> (
        bool,
        (std::cmp::Reverse<i16>, usize),
        bool,
        std::cmp::Reverse<i16>,
        usize,
    ) {
        let Kind::Land(land) = &self.deck[card].kind else {
            unreachable!()
        };
        let tapped = self.entry(land).0;
        let best = self
            .cast_options()
            .iter()
            .filter_map(|option| {
                self.payment_with_land(
                    &option.cost,
                    Context::Spell(&option.spell),
                    (!tapped).then_some(land),
                )
                .map(|_| {
                    (
                        std::cmp::Reverse(self.deck[option.card].keep_priority),
                        option.cost.mana_value(),
                    )
                })
            })
            .min();
        (
            best.is_none(),
            best.unwrap_or((std::cmp::Reverse(i16::MIN), usize::MAX)),
            tapped,
            std::cmp::Reverse(self.deck[card].keep_priority),
            card,
        )
    }
    pub(super) fn play_land(&mut self) {
        if self.land_played {
            return;
        }
        let card = self
            .hand
            .iter()
            .copied()
            .filter(|i| matches!(self.deck[*i].kind, Kind::Land(_)) && !self.duplicate_legend(*i))
            .min_by_key(|i| self.land_score(*i));
        if let Some(card) = card {
            self.land_played = true;
            let Kind::Land(land) = &self.deck[card].kind else {
                unreachable!()
            };
            let (tapped, life) = self.entry(land);
            self.life -= life;
            self.hand.retain(|i| *i != card);
            self.enter(card, tapped);
        }
    }
}
