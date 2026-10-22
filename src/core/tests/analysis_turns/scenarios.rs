use crate::analysis_support::*;
use alkemyst_core::analysis::turns::*;
use landlord::{SimulationRng, mana::Color::*};

#[test]
fn equipment_in_play_attached_and_attack_ready_are_separate_joint_events() {
    let mut t = trial(
        vec![
            land(1, &[White]),
            land(2, &[Red]),
            equipment(3, "{W}", "{1}"),
            creature(4, "{R}", false),
        ],
        vec![0, 1, 2, 3],
        true,
    );
    let mut rng = SimulationRng::seeded(1);
    let one = t.next_turn(&mut rng).unwrap();
    assert_eq!((one.equipment, one.attached, one.attack_ready), (1, 0, 0));
    assert_eq!(one.lands, 1);
    let two = t.next_turn(&mut rng).unwrap();
    assert_eq!((two.equipment, two.attached, two.attack_ready), (1, 1, 0));
    assert_eq!(two.spells_cast, 1);
    let three = t.next_turn(&mut rng).unwrap();
    assert_eq!(
        (three.equipment, three.attached, three.attack_ready),
        (1, 1, 1)
    );
    assert!(
        Goal {
            equipment: 1,
            attached: 1,
            attack_ready: 1
        }
        .met(&three)
    );
    assert!(
        !Goal {
            equipment: 1,
            attached: 1,
            attack_ready: 1
        }
        .met(&two)
    );
}

#[test]
fn tutor_mana_is_spent_before_its_hand_target_can_be_cast() {
    let mut t = trial(
        vec![
            land(1, &[White]),
            land(2, &[White]),
            tutor(3, "{W}", Destination::Hand, None),
            equipment(4, "{1}", "{1}"),
        ],
        vec![0, 1, 2],
        true,
    );
    let mut rng = SimulationRng::seeded(2);
    let one = t.next_turn(&mut rng).unwrap();
    assert_eq!((one.tutors_cast, one.equipment), (1, 0));
    let two = t.next_turn(&mut rng).unwrap();
    assert_eq!(two.equipment, 1);
}

#[test]
fn battlefield_tutors_bypass_cast_cost_but_honor_printed_mana_value_filter() {
    let mut high = equipment(4, "{9}", "{1}");
    high.printed_mana_value = 1;
    let mut t = trial(
        vec![
            land(1, &[White]),
            tutor(2, "{W}", Destination::Battlefield, Some(1)),
            high,
        ],
        vec![0, 1],
        true,
    );
    let one = t.next_turn(&mut SimulationRng::seeded(3)).unwrap();
    assert_eq!((one.tutors_cast, one.equipment), (1, 1));
    assert_eq!(one.spells_cast, 1);
    let mut t = trial(
        vec![
            land(1, &[White]),
            tutor(2, "{W}", Destination::Battlefield, Some(0)),
            equipment(3, "{1}", "{1}"),
        ],
        vec![0, 1],
        true,
    );
    assert_eq!(
        t.next_turn(&mut SimulationRng::seeded(3))
            .unwrap()
            .tutors_cast,
        0
    );
}

#[test]
fn legendary_duplicates_are_bricks_and_do_not_inflate_battlefield_count() {
    let mut eq = equipment(3, "", "{0}");
    eq.legendary = true;
    let mut t = trial(
        vec![eq.clone(), eq, creature(4, "", true)],
        vec![0, 1, 2],
        true,
    );
    let one = t.next_turn(&mut SimulationRng::seeded(4)).unwrap();
    assert_eq!(
        (
            one.equipment,
            one.attached,
            one.attack_ready,
            one.legendary_bricks
        ),
        (1, 1, 1, 1)
    );
    assert_eq!(one.spells_cast, 2);
}

#[test]
fn land_selection_uses_cast_costs_instead_of_hand_order() {
    let mut t = trial(
        vec![
            land(1, &[White]),
            land(2, &[Red]),
            equipment(3, "{R}", "{1}"),
        ],
        vec![0, 1, 2],
        true,
    );
    assert_eq!(
        t.next_turn(&mut SimulationRng::seeded(5))
            .unwrap()
            .equipment,
        1
    );
}

#[test]
fn conditional_entry_and_spell_only_sources_are_evaluated_per_action() {
    let mut tap = land(1, &[White]);
    if let Kind::Land(l) = &mut tap.kind {
        l.enters = Enters::Tapped;
    }
    let mut t = trial(
        vec![tap, equipment(2, "{W}", "{W}"), creature(3, "", true)],
        vec![0, 1, 2],
        true,
    );
    let mut rng = SimulationRng::seeded(6);
    assert_eq!(t.next_turn(&mut rng).unwrap().equipment, 0);
    let two = t.next_turn(&mut rng).unwrap();
    assert_eq!((two.equipment, two.attached), (1, 0));
    assert_eq!(t.next_turn(&mut rng).unwrap().attached, 1);

    let mut restricted = land(1, &[White]);
    if let Kind::Land(l) = &mut restricted.kind {
        l.restriction = Restriction::EquipmentSpell;
    }
    let mut t = trial(
        vec![
            restricted,
            equipment(2, "{W}", "{W}"),
            creature(3, "", true),
            tutor(4, "{W}", Destination::Hand, None),
        ],
        vec![0, 1, 2, 3],
        true,
    );
    assert_eq!(t.next_turn(&mut rng).unwrap().equipment, 1);
    let two = t.next_turn(&mut rng).unwrap();
    assert_eq!((two.attached, two.tutors_cast), (0, 0));
}

#[test]
fn check_fast_slow_and_life_paid_lands_have_explicit_conditions() {
    for enters in [Enters::OtherLandsAtLeast(1), Enters::HasBasicType(1)] {
        let mut basic = land(1, &[White]);
        if let Kind::Land(l) = &mut basic.kind {
            l.basic_types = 1;
        }
        let mut conditional = land(2, &[Red]);
        if let Kind::Land(l) = &mut conditional.kind {
            l.enters = enters;
        }
        let mut t = trial(
            vec![basic, conditional, equipment(3, "{W}{R}", "{1}")],
            vec![0, 1, 2],
            true,
        );
        let mut rng = SimulationRng::seeded(7);
        assert_eq!(t.next_turn(&mut rng).unwrap().equipment, 0);
        assert_eq!(t.next_turn(&mut rng).unwrap().equipment, 1);
    }
    let mut fast = land(1, &[White]);
    if let Kind::Land(l) = &mut fast.kind {
        l.enters = Enters::OtherLandsAtMost(0);
    }
    let mut t = trial(vec![fast, equipment(2, "{W}", "{1}")], vec![0, 1], true);
    assert_eq!(
        t.next_turn(&mut SimulationRng::seeded(8))
            .unwrap()
            .equipment,
        1
    );
    let mut shock = land(1, &[White]);
    if let Kind::Land(l) = &mut shock.kind {
        l.enters = Enters::PayLife(2);
    }
    let mut t = trial(vec![shock, equipment(2, "{W}", "{1}")], vec![0, 1], true);
    let one = t.next_turn(&mut SimulationRng::seeded(9)).unwrap();
    assert_eq!((one.equipment, one.life), (1, 18));
}

#[test]
fn higher_tutor_curve_delays_target_deployment_and_cannot_reuse_spent_mana() {
    let mut t = trial(
        vec![
            land(1, &[White]),
            land(2, &[White]),
            land(3, &[White]),
            tutor(4, "{2}{W}", Destination::Hand, None),
            equipment(5, "{1}", "{1}"),
        ],
        vec![0, 1, 2, 3],
        true,
    );
    let mut rng = SimulationRng::seeded(10);
    for _ in 0..2 {
        let turn = t.next_turn(&mut rng).unwrap();
        assert_eq!((turn.tutors_cast, turn.equipment), (0, 0));
    }
    let three = t.next_turn(&mut rng).unwrap();
    assert_eq!((three.tutors_cast, three.equipment), (1, 0));
    assert_eq!(t.next_turn(&mut rng).unwrap().equipment, 1);
}

#[test]
fn creature_only_mana_cannot_cast_equipment_and_artifact_only_mana_can() {
    for (restriction, expected) in [
        (Restriction::CreatureSpell, 0),
        (Restriction::ArtifactSpell, 1),
    ] {
        let mut l = land(1, &[White]);
        if let Kind::Land(spec) = &mut l.kind {
            spec.restriction = restriction;
        }
        let mut t = trial(vec![l, equipment(2, "{W}", "{1}")], vec![0, 1], true);
        assert_eq!(
            t.next_turn(&mut SimulationRng::seeded(11))
                .unwrap()
                .equipment,
            expected
        );
    }
}
