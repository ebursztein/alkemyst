use crate::analysis_support::*;
use alkemyst_core::analysis::turns::*;
use landlord::{SimulationRng, mana::Color::*};

#[test]
fn worthy_requires_a_legendary_non_villain_red_or_white_creature() {
    for (legendary, colors, villain, expected) in [
        (true, 1, false, 1),
        (false, 1, false, 0),
        (true, 2, false, 0),
        (true, 8, true, 0),
    ] {
        let mut hammer = equipment(2, "", "{1}");
        if let Kind::Spell(s) = &mut hammer.kind {
            s.worthy_only = true;
        }
        let mut body = creature(3, "", true);
        body.legendary = legendary;
        body.tags.colors = colors;
        body.tags.villain = villain;
        let mut t = trial(vec![land(1, &[White]), hammer, body], vec![0, 1, 2], true);
        assert_eq!(
            t.next_turn(&mut SimulationRng::seeded(1)).unwrap().attached,
            expected
        );
    }
}

#[test]
fn dragonfire_reduction_uses_target_colors_and_preserves_each_cost() {
    for (colors, expected) in [(1 | 8, 1), (0, 0)] {
        let mut blade = equipment(3, "", "{4}");
        if let Kind::Spell(s) = &mut blade.kind {
            s.equip_reduction_per_color = true;
        }
        let mut body = creature(4, "", true);
        body.tags.colors = colors;
        let mut t = trial(
            vec![land(1, &[White]), land(2, &[Red]), blade, body],
            vec![0, 1, 2, 3],
            true,
        );
        let mut rng = SimulationRng::seeded(2);
        assert_eq!(t.next_turn(&mut rng).unwrap().attached, 0);
        assert_eq!(t.next_turn(&mut rng).unwrap().attached, expected);
    }
}

fn kili() -> CardSpec {
    let mut card = creature(1, "", false);
    card.legendary = true;
    if let Kind::Spell(s) = &mut card.kind {
        s.story_free_equip = true;
    }
    card
}
#[test]
fn a_legendary_artifact_counts_once_toward_an_enduring_story() {
    let mut hammer = equipment(2, "", "{4}");
    hammer.legendary = true;
    let mut t = trial(vec![kili(), hammer, land(3, &[White])], vec![0, 1, 2], true);
    assert_eq!(
        t.next_turn(&mut SimulationRng::seeded(3)).unwrap().attached,
        0
    );
}

#[test]
fn story_first_equip_is_free_once_per_turn_and_does_not_waste_it_on_a_zero_cost() {
    let mut hero = creature(2, "", true);
    hero.legendary = true;
    let mut t = trial(
        vec![kili(), hero, equipment(3, "", ""), equipment(4, "", "{4}")],
        vec![0, 1, 2, 3],
        true,
    );
    assert_eq!(
        t.next_turn(&mut SimulationRng::seeded(4)).unwrap().attached,
        2
    );
    let mut hero = creature(2, "", true);
    hero.legendary = true;
    let mut t = trial(
        vec![
            kili(),
            hero,
            equipment(3, "", "{3}"),
            equipment(4, "", "{4}"),
        ],
        vec![0, 1, 2, 3],
        true,
    );
    let mut rng = SimulationRng::seeded(5);
    assert_eq!(t.next_turn(&mut rng).unwrap().attached, 1);
    assert_eq!(t.next_turn(&mut rng).unwrap().attached, 2);
}

#[test]
fn dwarven_mauler_discount_applies_only_when_it_is_the_equip_target() {
    let mut mauler = creature(2, "", true);
    if let Kind::Spell(s) = &mut mauler.kind {
        s.target_equip_reduction = 2;
    }
    let mut t = trial(
        vec![land(1, &[White]), mauler, equipment(3, "", "{3}")],
        vec![0, 1, 2],
        true,
    );
    assert_eq!(
        t.next_turn(&mut SimulationRng::seeded(6)).unwrap().attached,
        1
    );
}
