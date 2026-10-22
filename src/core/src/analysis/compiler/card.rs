use super::{models::*, presets, registry};
use crate::analysis::{ResolvedCard, turns::*};
use landlord::mana::ManaCost;
use serde_json::Value;
fn block(profile: &mut CardProfile, code: IssueCode, detail: &str) {
    profile.issues.push(Issue {
        code,
        blocking: true,
        detail: detail.into(),
    });
}
fn field(value: &Value, key: &str) -> Option<String> {
    value[key]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
}
fn colors(front: &Value, cost: &str) -> u8 {
    [("W", 1), ("U", 2), ("B", 4), ("R", 8), ("G", 16)]
        .iter()
        .filter(|(letter, _)| {
            front["colors"]
                .as_array()
                .map_or_else(|| cost.contains(letter), |a| a.iter().any(|c| c == *letter))
        })
        .fold(0, |mask, (_, bit)| mask | bit)
}
pub(super) fn compile(input: &ResolvedCard, id: u32) -> (CardProfile, Option<CardSpec>) {
    let mut profile = CardProfile {
        reference: input.reference.clone(),
        quantity: input.quantity,
        zone: input.zone,
        oracle_id: None,
        name: None,
        cast_cost: None,
        adventure_cost: None,
        equip_cost: None,
        issues: vec![],
    };
    let Some(raw) = &input.data else {
        block(
            &mut profile,
            IssueCode::MissingCard,
            "Card reference is unresolved in the active local catalog.",
        );
        return (profile, None);
    };
    profile.name = field(raw, "name");
    profile.oracle_id = field(raw, "oracle_id");
    let front = raw["card_faces"]
        .as_array()
        .and_then(|f| f.first())
        .unwrap_or(raw);
    let types = front["type_line"].as_str().unwrap_or("");
    let value = raw["cmc"].as_f64();
    if profile.name.is_none()
        || profile.oracle_id.is_none()
        || types.is_empty()
        || value.is_none_or(|n| !n.is_finite() || n < 0.0 || n.fract() != 0.0 || n > 1000.0)
        || input.quantity == 0
        || input.quantity > 250
    {
        block(
            &mut profile,
            IssueCode::InvalidMetadata,
            "Required card identity, type, quantity or integral printed mana value is invalid.",
        );
        return (profile, None);
    }
    let rule = match registry::rule(raw) {
        Ok(rule) => rule,
        Err(()) => {
            block(
                &mut profile,
                IssueCode::RulesChanged,
                "Reviewed rules fingerprint changed; this model requires review.",
            );
            return (profile, None);
        }
    };
    if rule.is_none() && raw["card_faces"].is_array() {
        block(
            &mut profile,
            IssueCode::UnsupportedLayout,
            "This multi-face card has no reviewed mode model.",
        );
        return (profile, None);
    }
    let cost = front["mana_cost"].as_str().unwrap_or("");
    profile.cast_cost = Some(cost.into());
    if !types.contains("Land") && cost.is_empty() {
        block(
            &mut profile,
            IssueCode::UnsupportedCost,
            "No printed casting cost requires an explicit alternate-cost model.",
        );
        return (profile, None);
    }
    let cost_parsed = match ManaCost::parse(cost) {
        Ok(v) => v,
        Err(_) => {
            block(
                &mut profile,
                IssueCode::UnsupportedCost,
                "The cast cost requires an explicit unsupported cost choice.",
            );
            return (profile, None);
        }
    };
    let mut card = CardSpec {
        id,
        cost: cost_parsed,
        printed_mana_value: value.unwrap() as u16,
        legendary: types.contains("Legendary"),
        keep_priority: 10,
        tags: Tags {
            colors: colors(front, cost),
            dwarf: types.split_whitespace().any(|w| w == "Dwarf"),
            villain: types.split_whitespace().any(|w| w == "Villain"),
            saga: types.split_whitespace().any(|w| w == "Saga"),
        },
        adventure: None,
        kind: Kind::Ignored,
    };
    if let Some(rule) = rule {
        match presets::apply(rule, raw, &mut card) {
            Ok(details) => {
                profile.adventure_cost = details.adventure;
                profile.equip_cost = details.equip;
                profile.issues.extend(details.issues);
            }
            Err(()) => {
                block(
                    &mut profile,
                    IssueCode::InvalidMetadata,
                    "Reviewed rule inputs could not construct a valid model.",
                );
                return (profile, None);
            }
        }
    } else if let Some(land) =
        super::lands::basic(types, front["oracle_text"].as_str().unwrap_or(""))
    {
        card.kind = Kind::Land(land);
    } else if let Some(spell) = super::generic::creature(types, front) {
        card.kind = Kind::Spell(spell);
        if !front["oracle_text"].as_str().unwrap_or("").is_empty() {
            profile.issues.push(presets::warning(
                IssueCode::CombatExcluded,
                "Only haste eligibility is modeled; other combat keywords are excluded.",
            ));
        }
    } else {
        block(
            &mut profile,
            IssueCode::UnknownRules,
            "Unreviewed card abilities are not guessed or silently ignored.",
        );
        return (profile, None);
    }
    (profile, Some(card))
}
