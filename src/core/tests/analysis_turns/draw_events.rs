use crate::analysis_support::*;
use alkemyst_core::analysis::turns::*;
use landlord::{SimulationRng, mana::Color::*};
use std::sync::Arc;
fn drawer(id: u32, allow_self: bool, once: bool, discard: DiscardPolicy) -> CardSpec {
    let mut c = creature(id, "", false);
    c.tags.dwarf = true;
    c.keep_priority = 10;
    if let Kind::Spell(s) = &mut c.kind {
        s.entry_draw = Some(EntryDraw {
            allow_self,
            once_per_turn: once,
            discard,
        });
    }
    c
}
#[test]
fn kili_draws_for_another_dwarf_or_equipment_once_per_turn() {
    let mut d = creature(4, "", false);
    d.tags.dwarf = true;
    let mut t = trial(
        vec![
            drawer(1, false, true, DiscardPolicy::None),
            equipment(2, "", ""),
            equipment(3, "", ""),
            d,
        ],
        vec![0, 1, 2, 3],
        true,
    );
    let one = t.next_turn(&mut SimulationRng::seeded(1)).unwrap();
    assert_eq!(one.cards_drawn, 1);
    assert_eq!(one.equipment, 2);
}
#[test]
fn once_per_turn_draw_resets_and_preserves_normal_draws() {
    let cards = vec![
        drawer(1, false, true, DiscardPolicy::None),
        equipment(2, "", ""),
        equipment(3, "{R}", ""),
        land(4, &[Red]),
        land(5, &[White]),
    ];
    let mut t = Trial::new(Arc::from(cards), vec![0, 1], vec![2, 3, 4], true).unwrap();
    let mut rng = SimulationRng::seeded(2);
    assert_eq!(t.next_turn(&mut rng).unwrap().cards_drawn, 1);
    let two = t.next_turn(&mut rng).unwrap();
    assert_eq!((two.cards_drawn, two.equipment), (2, 2));
}
#[test]
fn giott_discards_before_drawing_and_does_not_draw_with_an_empty_hand() {
    let mut ignored = land(2, &[White]);
    ignored.kind = Kind::Ignored;
    let mut t = Trial::new(
        Arc::from(vec![
            drawer(1, true, false, DiscardPolicy::Bricks),
            ignored,
            equipment(3, "", ""),
        ]),
        vec![0, 1],
        vec![2],
        true,
    )
    .unwrap();
    let one = t.next_turn(&mut SimulationRng::seeded(3)).unwrap();
    assert_eq!(
        (one.cards_drawn, one.cards_discarded, one.equipment),
        (1, 1, 1)
    );
}
#[test]
fn a_failed_triggered_draw_stops_further_casts_and_attachments() {
    let mut t = Trial::new(
        Arc::from(vec![
            drawer(1, false, true, DiscardPolicy::None),
            equipment(2, "", ""),
            equipment(3, "", ""),
        ]),
        vec![0, 1, 2],
        vec![],
        true,
    )
    .unwrap();
    let one = t.next_turn(&mut SimulationRng::seeded(4)).unwrap();
    assert!(one.decked);
    assert_eq!((one.equipment, one.spells_cast), (1, 2));
}

#[test]
fn a_land_drawn_mid_turn_can_use_the_still_unused_land_drop() {
    let cards = vec![
        drawer(1, false, true, DiscardPolicy::None),
        equipment(2, "", ""),
        equipment(3, "{R}", ""),
        land(4, &[Red]),
        land(5, &[White]),
    ];
    let mut t = Trial::new(Arc::from(cards), vec![0, 1, 2], vec![3, 4], true).unwrap();
    let one = t.next_turn(&mut SimulationRng::seeded(7)).unwrap();
    assert_eq!((one.equipment, one.lands), (2, 1));
}
