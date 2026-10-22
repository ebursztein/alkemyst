use crate::analysis_support::*;
use alkemyst_core::analysis::turns::*;
use landlord::{
    SimulationRng,
    mana::{Color::*, Source},
};
use std::sync::Arc;

#[test]
fn sunbillow_keeps_white_and_unlocks_red_when_a_typed_land_enters() {
    let mut verge = land(1, &[White]);
    if let Kind::Land(l) = &mut verge.kind {
        l.conditional.push(ManaAbility {
            source: Source::new(&[Red]).unwrap(),
            restriction: Restriction::Any,
            condition: Condition::HasBasicType(1 | 8),
        });
    }
    let mut plains = land(3, &[White]);
    if let Kind::Land(l) = &mut plains.kind {
        l.basic_types = 1;
    }
    let mut t = Trial::new(
        Arc::from(vec![
            verge,
            equipment(2, "{R}", "{1}"),
            plains,
            land(4, &[White]),
        ]),
        vec![0, 1],
        vec![2, 3],
        true,
    )
    .unwrap();
    let mut rng = SimulationRng::seeded(1);
    assert_eq!(t.next_turn(&mut rng).unwrap().equipment, 0);
    assert_eq!(t.next_turn(&mut rng).unwrap().equipment, 1);
}

#[test]
fn freya_cannot_tap_with_summoning_sickness_and_tapping_removes_attack_readiness() {
    let mut freya = creature(1, "{R}", false);
    freya.keep_priority = 10;
    if let Kind::Spell(s) = &mut freya.kind {
        s.mana_ability = Some(ManaAbility {
            source: Source::new(&[Red]).unwrap(),
            restriction: Restriction::EquipmentOrEquip,
            condition: Condition::Always,
        });
    }
    let mut t = trial(
        vec![land(2, &[Red]), freya, equipment(3, "{2}", "")],
        vec![0, 1, 2],
        true,
    );
    let mut rng = SimulationRng::seeded(2);
    let one = t.next_turn(&mut rng).unwrap();
    assert_eq!((one.equipment, one.spells_cast), (0, 1));
    let two = t.next_turn(&mut rng).unwrap();
    assert_eq!((two.equipment, two.attached, two.attack_ready), (1, 1, 0));
    assert_eq!(t.next_turn(&mut rng).unwrap().attack_ready, 1);
}

#[test]
fn freyas_equipment_or_equip_source_does_not_pay_for_tutors() {
    let mut freya = creature(1, "", true);
    if let Kind::Spell(s) = &mut freya.kind {
        s.mana_ability = Some(ManaAbility {
            source: Source::new(&[Red]).unwrap(),
            restriction: Restriction::EquipmentOrEquip,
            condition: Condition::Always,
        });
    }
    let mut t = trial(
        vec![
            freya,
            tutor(2, "{R}", Destination::Hand, None),
            equipment(3, "{R}", ""),
        ],
        vec![0, 1],
        true,
    );
    let one = t.next_turn(&mut SimulationRng::seeded(3)).unwrap();
    assert_eq!((one.tutors_cast, one.equipment), (0, 0));
}
