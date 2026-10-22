use crate::analysis_support::*;
use alkemyst_core::analysis::turns::*;
use landlord::SimulationRng;

#[test]
fn thorins_entry_attaches_without_activating_a_restricted_equip_ability() {
    let mut hammer = equipment(1, "", "{5}");
    if let Kind::Spell(s) = &mut hammer.kind {
        s.worthy_only = true;
    }
    let body = creature(2, "", true);
    let mut thorin = creature(3, "", false);
    thorin.keep_priority = -1;
    if let Kind::Spell(s) = &mut thorin.kind {
        s.attach_on_enter = true;
    }
    let mut t = trial(vec![hammer, body, thorin], vec![0, 1, 2], true);
    let one = t.next_turn(&mut SimulationRng::seeded(7)).unwrap();
    assert_eq!((one.attached, one.attack_ready), (1, 1));
}

#[test]
fn tentacles_trigger_for_a_new_large_creature_not_one_already_present() {
    for (equipment_first, expected) in [(true, 1), (false, 0)] {
        let mut tentacles = equipment(1, "", "{5}");
        if let Kind::Spell(s) = &mut tentacles.kind {
            s.auto_attach_min_value = Some(5);
        }
        let mut thor = creature(2, "", true);
        thor.printed_mana_value = 5;
        if equipment_first {
            tentacles.keep_priority = 1;
        } else {
            thor.keep_priority = 1;
        }
        let mut t = trial(vec![tentacles, thor], vec![0, 1], true);
        let one = t.next_turn(&mut SimulationRng::seeded(8)).unwrap();
        assert_eq!((one.attached, one.attack_ready), (expected, expected));
    }
}
