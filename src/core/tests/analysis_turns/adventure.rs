use crate::analysis_support::*;
use alkemyst_core::analysis::turns::*;
use landlord::{SimulationRng, mana::Color::*};
use std::sync::Arc;
fn adventurer(front: &str, back: &str) -> CardSpec {
    let mut c = creature(1, front, false);
    c.legendary = true;
    c.adventure = Some(Adventure {
        cost: cost(back),
        tutor: TutorSpec {
            scope: SearchScope::Library,
            types: SearchTypes::Equipment,
            timing: TutorTiming::SpellResolves,
            destination: Destination::Hand,
            max_mana_value: None,
        },
    });
    c
}
#[test]
fn birthright_boon_searches_then_kellan_can_be_cast_from_exile_later() {
    let mut t = trial(
        vec![
            land(2, &[White]),
            land(3, &[Red]),
            land(4, &[Red]),
            adventurer("{2}{R}", "{1}{W}"),
            equipment(5, "", ""),
        ],
        vec![0, 1, 2, 3],
        true,
    );
    let mut rng = SimulationRng::seeded(1);
    assert_eq!(t.next_turn(&mut rng).unwrap().creatures, 0);
    let two = t.next_turn(&mut rng).unwrap();
    assert_eq!(
        (two.equipment, two.creatures, two.adventure_exile),
        (1, 0, 1)
    );
    let three = t.next_turn(&mut rng).unwrap();
    assert_eq!((three.creatures, three.adventure_exile), (1, 0));
}
#[test]
fn another_copy_can_adventure_while_a_legendary_front_is_controlled() {
    let c = adventurer("", "{R}");
    let mut t = Trial::new(
        Arc::from(vec![
            c.clone(),
            c,
            land(2, &[Red]),
            equipment(3, "", ""),
            land(4, &[White]),
        ]),
        vec![0, 1],
        vec![2, 3, 4],
        true,
    )
    .unwrap();
    let mut rng = SimulationRng::seeded(2);
    let one = t.next_turn(&mut rng).unwrap();
    assert_eq!((one.creatures, one.legendary_bricks), (1, 0));
    let two = t.next_turn(&mut rng).unwrap();
    assert_eq!(
        (
            two.creatures,
            two.equipment,
            two.adventure_exile,
            two.legendary_bricks
        ),
        (1, 1, 1, 1)
    );
}
