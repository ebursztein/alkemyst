use alkemyst_core::analysis::turns::*;
use landlord::mana::{Color, ManaCost, Source};
use std::sync::Arc;

pub fn cost(s: &str) -> ManaCost {
    ManaCost::parse(s).unwrap()
}
pub fn land(id: u32, colors: &[Color]) -> CardSpec {
    CardSpec {
        id,
        cost: cost(""),
        printed_mana_value: 0,
        legendary: false,
        keep_priority: 0,
        tags: Default::default(),
        adventure: None,
        kind: Kind::Land(LandSpec {
            source: Source::new(colors).unwrap(),
            enters: Enters::Untapped,
            restriction: Restriction::Any,
            basic_types: 0,
            conditional: vec![],
        }),
    }
}
pub fn equipment(id: u32, mana: &str, equip: &str) -> CardSpec {
    CardSpec {
        id,
        cost: cost(mana),
        printed_mana_value: cost(mana).mana_value() as u16,
        legendary: false,
        keep_priority: 0,
        tags: Default::default(),
        adventure: None,
        kind: Kind::Spell(SpellSpec {
            permanent: true,
            artifact: true,
            equipment: true,
            equip_cost: Some(cost(equip)),
            ..Default::default()
        }),
    }
}
pub fn creature(id: u32, mana: &str, haste: bool) -> CardSpec {
    CardSpec {
        id,
        cost: cost(mana),
        printed_mana_value: cost(mana).mana_value() as u16,
        legendary: false,
        keep_priority: 0,
        tags: Default::default(),
        adventure: None,
        kind: Kind::Spell(SpellSpec {
            permanent: true,
            creature: true,
            haste,
            ..Default::default()
        }),
    }
}
pub fn tutor(id: u32, mana: &str, destination: Destination, max: Option<u16>) -> CardSpec {
    CardSpec {
        id,
        cost: cost(mana),
        printed_mana_value: cost(mana).mana_value() as u16,
        legendary: false,
        keep_priority: 0,
        tags: Default::default(),
        adventure: None,
        kind: Kind::Spell(SpellSpec {
            tutor: Some(TutorSpec {
                scope: SearchScope::Library,
                types: SearchTypes::Equipment,
                timing: TutorTiming::SpellResolves,
                destination,
                max_mana_value: max,
            }),
            ..Default::default()
        }),
    }
}
pub fn trial(mut cards: Vec<CardSpec>, opening: Vec<usize>, play: bool) -> Trial {
    // Give elementary multi-turn cases an undrawn library. Filler has no effect.
    cards.extend((0..10).map(|i| CardSpec {
        id: 1000 + i,
        cost: cost(""),
        printed_mana_value: 0,
        legendary: false,
        keep_priority: 0,
        tags: Default::default(),
        adventure: None,
        kind: Kind::Ignored,
    }));
    let mut library = (0..cards.len())
        .filter(|i| !opening.contains(i))
        .collect::<Vec<_>>();
    library.sort_by_key(|i| !matches!(cards[*i].kind, Kind::Ignored));
    Trial::new(Arc::from(cards), opening, library, play).unwrap()
}
