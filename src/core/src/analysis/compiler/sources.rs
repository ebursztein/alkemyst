use super::models::SourceCount;
use crate::analysis::turns::*;
use landlord::mana::Color;
pub(super) fn counts(cards: &[CardSpec]) -> Vec<SourceCount> {
    [
        ("W", Color::White),
        ("U", Color::Blue),
        ("B", Color::Black),
        ("R", Color::Red),
        ("G", Color::Green),
        ("C", Color::Colorless),
    ]
    .into_iter()
    .map(|(name, color)| {
        let mut count = SourceCount {
            color: name.into(),
            unrestricted_lands: 0,
            conditional_lands: 0,
            nonland_sources: 0,
        };
        for card in cards {
            match &card.kind {
                Kind::Land(l) => {
                    let always = (l.source.supports(color) && l.restriction == Restriction::Any)
                        || l.conditional.iter().any(|a| {
                            a.source.supports(color)
                                && a.restriction == Restriction::Any
                                && a.condition == Condition::Always
                        });
                    let potential = l.source.supports(color)
                        || l.conditional.iter().any(|a| a.source.supports(color));
                    if always {
                        count.unrestricted_lands += 1;
                    } else if potential {
                        count.conditional_lands += 1;
                    }
                }
                Kind::Spell(s)
                    if s.mana_ability
                        .as_ref()
                        .is_some_and(|a| a.source.supports(color)) =>
                {
                    count.nonland_sources += 1
                }
                _ => {}
            }
        }
        count
    })
    .collect()
}
