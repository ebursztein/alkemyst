use super::{
    models::{Issue, IssueCode},
    registry::Rule,
};
use crate::analysis::turns::*;
use landlord::mana::{Color, ManaCost, Source};
use serde_json::Value;
#[derive(Default)]
pub(super) struct Details {
    pub equip: Option<String>,
    pub adventure: Option<String>,
    pub issues: Vec<Issue>,
}
pub(super) fn warning(code: IssueCode, detail: &str) -> Issue {
    Issue {
        code,
        blocking: false,
        detail: detail.into(),
    }
}
fn search(timing: TutorTiming, scope: SearchScope, types: SearchTypes) -> TutorSpec {
    TutorSpec {
        timing,
        scope,
        types,
        destination: Destination::Hand,
        max_mana_value: None,
    }
}
fn equip(text: &str, worthy: bool) -> Option<String> {
    let marker = if worthy { "Equip worthy " } else { "Equip " };
    let mut rest = text.split_once(marker)?.1;
    let mut cost = String::new();
    while rest.starts_with('{') {
        let end = rest.find('}')?;
        cost.push_str(&rest[..=end]);
        rest = &rest[end + 1..];
    }
    (!cost.is_empty()).then_some(cost)
}
pub(super) fn apply(rule: Rule, raw: &Value, card: &mut CardSpec) -> Result<Details, ()> {
    let mut details = Details::default();
    if let Some(land) = super::lands::land(rule) {
        card.kind = Kind::Land(land);
        if matches!(rule, Rule::Lonely) {
            details.issues.push(warning(
                IssueCode::ActivatedAbilityExcluded,
                "The Lonely Mountain's token activation is not used by this policy.",
            ));
        }
        return Ok(details);
    }
    let front = raw["card_faces"]
        .as_array()
        .and_then(|f| f.first())
        .unwrap_or(raw);
    let types = front["type_line"].as_str().ok_or(())?;
    let mut spell = SpellSpec {
        permanent: types.contains("Creature")
            || types.contains("Artifact")
            || types.contains("Enchantment"),
        creature: types.contains("Creature"),
        artifact: types.contains("Artifact"),
        ..Default::default()
    };
    if types.contains("Equipment") {
        let cost = equip(
            front["oracle_text"].as_str().ok_or(())?,
            matches!(rule, Rule::Mjolnir),
        )
        .ok_or(())?;
        spell.equipment = true;
        spell.equip_cost = Some(ManaCost::parse(&cost).map_err(|_| ())?);
        details.equip = Some(cost);
        card.keep_priority = 60;
    }
    use Rule::*;
    match rule {
        Basilisk | BusterSword | Chainsaw => {}
        Tentacles => spell.auto_attach_min_value = Some(5),
        Dragonfire => spell.equip_reduction_per_color = true,
        Mjolnir => spell.worthy_only = true,
        Mauler => {
            spell.target_equip_reduction = 2;
            card.keep_priority = 20;
        }
        Cloud => {
            spell.tutor = Some(search(
                TutorTiming::EnterBattlefield,
                SearchScope::Library,
                SearchTypes::Equipment,
            ));
            card.keep_priority = 90;
        }
        Dain => {
            spell.tutor = Some(search(
                TutorTiming::EnterBattlefield,
                SearchScope::Top(4),
                SearchTypes::EquipmentOrDwarf,
            ));
            card.keep_priority = 75;
        }
        Freya => {
            spell.mana_ability = Some(ManaAbility {
                source: Source::new(&[Color::Red]).map_err(|_| ())?,
                condition: Condition::Always,
                restriction: Restriction::EquipmentOrEquip,
            });
            card.keep_priority = 85;
        }
        Giott => {
            spell.entry_draw = Some(EntryDraw {
                allow_self: true,
                once_per_turn: false,
                discard: DiscardPolicy::Bricks,
            });
            card.keep_priority = 70;
        }
        Kili => {
            spell.story_free_equip = true;
            spell.entry_draw = Some(EntryDraw {
                allow_self: false,
                once_per_turn: true,
                discard: DiscardPolicy::None,
            });
            card.keep_priority = 100;
        }
        Kellan => {
            let cost = raw["card_faces"][1]["mana_cost"].as_str().ok_or(())?;
            card.adventure = Some(Adventure {
                cost: ManaCost::parse(cost).map_err(|_| ())?,
                tutor: search(
                    TutorTiming::SpellResolves,
                    SearchScope::Library,
                    SearchTypes::Equipment,
                ),
            });
            details.adventure = Some(cost.into());
            card.keep_priority = 80;
        }
        Thorin => {
            spell.attach_on_enter = true;
            card.keep_priority = 65;
        }
        Raubahn | Thor => card.keep_priority = 30,
        Sheltered => {
            card.kind = Kind::Ignored;
            card.keep_priority = -100;
            details.issues.push(warning(
                IssueCode::IgnoredCard,
                "This Aura is not cast in the no-opponent main-phase model.",
            ));
            return Ok(details);
        }
        _ => return Err(()),
    }
    if !matches!(rule, Mauler | Kili) {
        details.issues.push(warning(IssueCode::CombatExcluded,"Combat, damage, protection and combat-dependent draw/free-cast effects are not resolved."));
    }
    if matches!(rule, Thor) {
        details.issues.push(warning(
            IssueCode::RecursionExcluded,
            "Thor's graveyard permission is not used by this policy.",
        ));
    }
    if matches!(rule, Mjolnir) {
        details.issues.push(warning(
            IssueCode::ActivatedAbilityExcluded,
            "The discard/damage mode is not used.",
        ));
    }
    if matches!(rule, Cloud) {
        details.issues.push(warning(IssueCode::ActivatedAbilityExcluded,"Equipped trigger copying is omitted; supported main-phase attachment copies are idempotent in this registry."));
    }
    card.kind = Kind::Spell(spell);
    Ok(details)
}
