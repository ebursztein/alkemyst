use super::*;
use landlord::mana::ManaCost;

#[derive(Clone, Copy)]
pub(super) enum Context<'a> {
    Spell(&'a SpellSpec),
    Equip,
}
impl Context<'_> {
    fn permits(self, restriction: Restriction) -> bool {
        match (self, restriction) {
            (_, Restriction::Any) => true,
            (Self::Spell(s), Restriction::ArtifactSpell) => s.artifact,
            (Self::Spell(s), Restriction::EquipmentSpell) => s.equipment,
            (Self::Spell(s), Restriction::CreatureSpell) => s.creature,
            _ => false,
        }
    }
}
impl Trial {
    fn entry(&self, land: &LandSpec) -> (bool, u8) {
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
            .hand
            .iter()
            .filter_map(|i| {
                let Kind::Spell(s) = &self.deck[*i].kind else {
                    return None;
                };
                if self.duplicate_legend(*i)
                    || !s.permanent
                        && s.tutor
                            .as_ref()
                            .is_some_and(|t| self.tutor_target(t).is_none())
                {
                    return None;
                }
                self.payment_with_land(
                    &self.deck[*i].cost,
                    Context::Spell(s),
                    (!tapped).then_some(land),
                )
                .map(|_| {
                    (
                        std::cmp::Reverse(self.deck[*i].keep_priority),
                        self.deck[*i].cost.mana_value(),
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
        let card = self
            .hand
            .iter()
            .copied()
            .filter(|i| matches!(self.deck[*i].kind, Kind::Land(_)) && !self.duplicate_legend(*i))
            .min_by_key(|i| self.land_score(*i));
        if let Some(card) = card {
            let Kind::Land(land) = &self.deck[card].kind else {
                unreachable!()
            };
            let (tapped, life) = self.entry(land);
            self.life -= life;
            self.hand.retain(|i| *i != card);
            self.enter(card, tapped);
        }
    }
    fn payment_with_land(
        &self,
        cost: &ManaCost,
        context: Context<'_>,
        extra: Option<&LandSpec>,
    ) -> Option<Vec<usize>> {
        let mut lands = self
            .battlefield
            .iter()
            .enumerate()
            .filter_map(|(index, p)| {
                let Kind::Land(land) = &self.deck[p.card].kind else {
                    return None;
                };
                (!p.tapped && context.permits(land.restriction)).then_some((index, land))
            })
            .collect::<Vec<_>>();
        if let Some(land) = extra
            && context.permits(land.restriction)
        {
            lands.push((self.battlefield.len(), land));
        }
        lands.sort_by_key(|(_, l)| l.restriction == Restriction::Any);
        let sources = lands
            .iter()
            .map(|(_, land)| land.source)
            .collect::<Vec<_>>();
        cost.payment(&sources)
            .map(|payment| payment.iter().map(|i| lands[*i].0).collect())
    }
    pub(super) fn can_pay(&self, cost: &ManaCost, context: Context<'_>) -> bool {
        self.payment_with_land(cost, context, None).is_some()
    }
    pub(super) fn pay(&mut self, cost: &ManaCost, context: Context<'_>) -> bool {
        let Some(payment) = self.payment_with_land(cost, context, None) else {
            return false;
        };
        for index in payment {
            self.battlefield[index].tapped = true;
        }
        true
    }
}
