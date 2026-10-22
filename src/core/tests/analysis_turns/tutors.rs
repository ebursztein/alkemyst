use crate::analysis_support::*;
use alkemyst_core::analysis::turns::*;
use landlord::{SimulationRng, mana::Color::*};

#[test]
fn creature_tutor_can_be_cast_for_its_body_without_a_search_target() {
    let mut body = creature(1, "", false);
    let Kind::Spell(s) = &mut body.kind else {
        unreachable!()
    };
    s.tutor = Some(TutorSpec {
        scope: SearchScope::Library,
        types: SearchTypes::Equipment,
        destination: Destination::Hand,
        max_mana_value: None,
        timing: TutorTiming::EnterBattlefield,
    });
    let mut t = trial(vec![body], vec![0], true);
    let first = t.next_turn(&mut SimulationRng::seeded(1)).unwrap();
    assert_eq!(
        (first.spells_cast, first.searches_resolved, first.equipment),
        (1, 1, 0)
    );
}

#[test]
fn battlefield_entries_trigger_searches_but_do_not_trigger_spell_resolution_effects() {
    for (timing, expected_equipment, searches) in [
        (TutorTiming::EnterBattlefield, 2, 2),
        (TutorTiming::SpellResolves, 1, 1),
    ] {
        let mut fetched = equipment(3, "{1}", "{1}");
        let Kind::Spell(s) = &mut fetched.kind else {
            unreachable!()
        };
        s.tutor = Some(TutorSpec {
            scope: SearchScope::Library,
            types: SearchTypes::Equipment,
            destination: Destination::Battlefield,
            max_mana_value: Some(0),
            timing,
        });
        let mut target = equipment(4, "{9}", "{1}");
        target.printed_mana_value = 0;
        let mut t = trial(
            vec![
                land(1, &[White]),
                tutor(2, "{W}", Destination::Battlefield, Some(1)),
                fetched,
                target,
            ],
            vec![0, 1],
            true,
        );
        let first = t.next_turn(&mut SimulationRng::seeded(2)).unwrap();
        assert_eq!(
            (
                first.equipment,
                first.spells_cast,
                first.tutors_cast,
                first.searches_resolved
            ),
            (expected_equipment, 1, 1, searches)
        );
    }
}
