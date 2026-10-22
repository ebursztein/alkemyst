use crate::analysis_support::*;
use alkemyst_core::analysis::turns::*;
use landlord::{SimulationRng, mana::Color::*};
use std::sync::Arc;

#[test]
fn dain_looks_only_at_top_four_and_bottoms_the_rest_without_shuffling_the_library() {
    let mut dain = creature(1, "", false);
    if let Kind::Spell(s) = &mut dain.kind {
        s.tutor = Some(TutorSpec {
            timing: TutorTiming::EnterBattlefield,
            destination: Destination::Hand,
            max_mana_value: None,
            scope: SearchScope::Top(4),
            types: SearchTypes::EquipmentOrDwarf,
        });
    }
    let mut dwarf = creature(2, "", false);
    dwarf.tags.dwarf = true;
    let mut outside = equipment(6, "", "");
    outside.keep_priority = 20;
    let cards = vec![
        dain,
        land(3, &[White]),
        dwarf,
        land(4, &[White]),
        land(5, &[White]),
        outside,
    ];
    let mut t = Trial::new(Arc::from(cards), vec![0], vec![1, 2, 3, 4, 5], true).unwrap();
    let mut rng = SimulationRng::seeded(5);
    let one = t.next_turn(&mut rng).unwrap();
    assert_eq!((one.creatures, one.equipment), (2, 0));
    assert_eq!(t.next_turn(&mut rng).unwrap().equipment, 1);
}

#[test]
fn a_top_selection_spell_is_not_skipped_using_knowledge_of_unseen_cards() {
    let mut selection = tutor(1, "", Destination::Hand, None);
    if let Kind::Spell(s) = &mut selection.kind {
        s.tutor.as_mut().unwrap().scope = SearchScope::Top(4);
    }
    let cards = vec![
        selection,
        land(2, &[White]),
        land(3, &[White]),
        land(4, &[White]),
        land(5, &[White]),
        equipment(6, "", ""),
    ];
    let mut t = Trial::new(Arc::from(cards), vec![0], vec![1, 2, 3, 4, 5], true).unwrap();
    let one = t.next_turn(&mut SimulationRng::seeded(6)).unwrap();
    assert_eq!((one.searches_resolved, one.spells_cast), (1, 1));
}
