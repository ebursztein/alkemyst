use super::*;
use landlord::mana::{ManaCost, Source};

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
            (Self::Spell(s), Restriction::EquipmentSpell | Restriction::EquipmentOrEquip) => {
                s.equipment
            }
            (Self::Spell(s), Restriction::CreatureSpell) => s.creature,
            (Self::Equip, Restriction::EquipmentOrEquip) => true,
            _ => false,
        }
    }
}
impl Trial {
    pub(super) fn condition(&self, condition: Condition, extra: Option<&LandSpec>) -> bool {
        match condition {
            Condition::Always => true,
            Condition::HasBasicType(mask) => extra.is_some_and(|l| l.basic_types & mask != 0)
                || self.battlefield.iter().any(
                    |p| matches!(&self.deck[p.card].kind,Kind::Land(l) if l.basic_types & mask !=0),
                ),
            Condition::ControlsEquipment => self
                .battlefield
                .iter()
                .any(|p| matches!(&self.deck[p.card].kind,Kind::Spell(s) if s.equipment)),
        }
    }
    fn land_source(
        &self,
        land: &LandSpec,
        context: Context<'_>,
        extra: Option<&LandSpec>,
    ) -> Option<(Source, bool)> {
        let mut source = context
            .permits(land.restriction)
            .then_some((land.source, land.restriction == Restriction::Any));
        for ability in &land.conditional {
            if context.permits(ability.restriction) && self.condition(ability.condition, extra) {
                source = Some(source.map_or(
                    (ability.source, ability.restriction == Restriction::Any),
                    |(old, any)| {
                        (
                            old.union(ability.source),
                            any || ability.restriction == Restriction::Any,
                        )
                    },
                ));
            }
        }
        source
    }
    pub(super) fn payment_with_land(
        &self,
        cost: &ManaCost,
        context: Context<'_>,
        extra: Option<&LandSpec>,
    ) -> Option<Vec<usize>> {
        let mut sources = self
            .battlefield
            .iter()
            .enumerate()
            .filter_map(|(index, p)| {
                if p.tapped {
                    return None;
                }
                let (source, any) = match &self.deck[p.card].kind {
                    Kind::Land(land) => self.land_source(land, context, extra)?,
                    Kind::Spell(s) => {
                        let ability = s.mana_ability.as_ref()?;
                        if (s.creature && !self.ready(p.card))
                            || !context.permits(ability.restriction)
                            || !self.condition(ability.condition, extra)
                        {
                            return None;
                        }
                        (ability.source, ability.restriction == Restriction::Any)
                    }
                    Kind::Ignored => return None,
                };
                Some((index, source, any))
            })
            .collect::<Vec<_>>();
        if let Some(land) = extra
            && let Some((source, any)) = self.land_source(land, context, extra)
        {
            sources.push((self.battlefield.len(), source, any));
        }
        sources.sort_by_key(|(_, _, any)| *any);
        let options = sources
            .iter()
            .map(|(_, source, _)| *source)
            .collect::<Vec<_>>();
        cost.payment(&options)
            .map(|payment| payment.iter().map(|i| sources[*i].0).collect())
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
