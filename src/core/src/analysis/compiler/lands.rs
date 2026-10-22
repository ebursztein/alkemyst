use super::super::turns::*;
use super::registry::Rule;
use landlord::mana::{Color, Source};
fn source(colors: &[Color]) -> Source {
    Source::new(colors).expect("nonempty reviewed source")
}
pub(super) fn land(rule: Rule) -> Option<LandSpec> {
    use Color::*;
    let (colors, enters, basic_types) = match rule {
        Rule::Plains => (vec![White], Enters::Untapped, 1),
        Rule::Mountain => (vec![Red], Enters::Untapped, 8),
        Rule::Vantage => (vec![White, Red], Enters::OtherLandsAtMost(2), 0),
        Rule::Sundown => (vec![White, Red], Enters::OtherLandsAtLeast(2), 0),
        Rule::Foundry => (vec![White, Red], Enters::PayLife(2), 1 | 8),
        Rule::Lonely => (vec![Red], Enters::ControlsEquipment, 8),
        Rule::Verge => (vec![White], Enters::Untapped, 0),
        _ => return None,
    };
    let conditional = if matches!(rule, Rule::Verge) {
        vec![ManaAbility {
            source: source(&[Red]),
            condition: Condition::HasBasicType(1 | 8),
            restriction: Restriction::Any,
        }]
    } else {
        vec![]
    };
    Some(LandSpec {
        source: source(&colors),
        enters,
        basic_types,
        restriction: Restriction::Any,
        conditional,
    })
}
pub(super) fn basic(type_line: &str, text: &str) -> Option<LandSpec> {
    if !type_line.split_whitespace().any(|s| s == "Basic") || !type_line.contains("Land") {
        return None;
    }
    for (name, color, letter, mask) in [
        ("Plains", Color::White, "W", 1),
        ("Island", Color::Blue, "U", 2),
        ("Swamp", Color::Black, "B", 4),
        ("Mountain", Color::Red, "R", 8),
        ("Forest", Color::Green, "G", 16),
    ] {
        if type_line.split_whitespace().any(|s| s == name)
            && (text.is_empty() || text == format!("({{T}}: Add {{{letter}}}.)"))
        {
            return Some(LandSpec {
                source: source(&[color]),
                enters: Enters::Untapped,
                basic_types: mask,
                restriction: Restriction::Any,
                conditional: vec![],
            });
        }
    }
    None
}
