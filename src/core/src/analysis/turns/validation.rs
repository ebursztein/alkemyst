use super::*;
use landlord::Error;
fn condition(condition: Condition) -> bool {
    !matches!(condition,Condition::HasBasicType(mask) if mask==0 || mask>31)
}
pub(super) fn card(card: &CardSpec) -> Result<(), Error> {
    let invalid = || Error::Configuration("unsupported or inconsistent card properties".into());
    if let Some(adventure) = &card.adventure
        && (!matches!(&card.kind,Kind::Spell(s) if s.permanent)
            || adventure.tutor.timing != TutorTiming::SpellResolves
            || adventure.tutor.scope == SearchScope::Top(0))
    {
        return Err(invalid());
    }
    if card.tags.colors > 31 {
        return Err(invalid());
    }
    match &card.kind {
        Kind::Land(l) => {
            if card.cost.mana_value() != 0
                || l.basic_types > 31
                || l.conditional.len() > 8
                || matches!(l.enters,Enters::HasBasicType(mask) if mask==0 || mask>31)
                || l.conditional
                    .iter()
                    .any(|ability| !condition(ability.condition))
            {
                return Err(invalid());
            }
        }
        Kind::Spell(s) => {
            if s.tutor
                .as_ref()
                .is_some_and(|t| t.scope == SearchScope::Top(0))
            {
                return Err(invalid());
            }
            if s.equipment && (!s.artifact || !s.permanent || s.creature) {
                return Err(invalid());
            }
            if s.equipment != s.equip_cost.is_some()
                || ((s.worthy_only
                    || s.equip_reduction_per_color
                    || s.auto_attach_min_value.is_some())
                    && !s.equipment)
            {
                return Err(invalid());
            }
            if (s.artifact
                || s.creature
                || s.story_free_equip
                || s.attach_on_enter
                || s.mana_ability.is_some()
                || s.entry_draw.is_some())
                && !s.permanent
            {
                return Err(invalid());
            }
            if (s.haste || s.target_equip_reduction > 0) && !s.creature {
                return Err(invalid());
            }
            if card.legendary && !s.permanent {
                return Err(invalid());
            }
            if s.mana_ability
                .as_ref()
                .is_some_and(|ability| !condition(ability.condition))
            {
                return Err(invalid());
            }
            if !s.permanent
                && s.tutor
                    .as_ref()
                    .is_some_and(|t| t.timing == TutorTiming::EnterBattlefield)
            {
                return Err(invalid());
            }
        }
        Kind::Ignored => {}
    }
    Ok(())
}
