use alkemyst_core::analysis::{
    simulation::{BattlefieldRunner, Scenario},
    turns::*,
};
use landlord::{London, mana::ManaCost, probability::hypergeometric};
use std::sync::Arc;

fn scenario() -> Scenario {
    let deck = (0..60)
        .map(|i| CardSpec {
            id: if i < 4 { 1 } else { 2 },
            cost: ManaCost::default(),
            printed_mana_value: 0,
            legendary: false,
            keep_priority: 0,
            tags: Default::default(),
            adventure: None,
            kind: if i < 4 {
                Kind::Spell(SpellSpec {
                    permanent: true,
                    artifact: true,
                    equipment: true,
                    equip_cost: Some(ManaCost::parse("{1}").unwrap()),
                    ..Default::default()
                })
            } else {
                Kind::Ignored
            },
        })
        .collect::<Vec<_>>();
    Scenario {
        deck: Arc::from(deck),
        turns: 3,
        on_the_play: true,
        mulligan: London::never(),
        goal: Goal {
            equipment: 1,
            attached: 0,
            attack_ready: 0,
        },
    }
}

#[test]
fn chunk_boundaries_preserve_replay_and_partial_denominators() {
    let mut one = BattlefieldRunner::new(scenario(), 100, 42).unwrap();
    assert!(one.report().is_none());
    assert_eq!(one.advance(0).unwrap(), 0);
    assert_eq!(one.advance(7).unwrap(), 7);
    let partial = one.report().unwrap();
    assert_eq!(partial.completed_runs, 7);
    assert_eq!(partial.requested_runs, 100);
    assert_eq!(partial.turns[0].joint.samples, 7);
    while !one.finished() {
        one.advance(13).unwrap();
    }
    assert_eq!(one.advance(99).unwrap(), 0);
    let mut two = BattlefieldRunner::new(scenario(), 100, 42).unwrap();
    two.advance(1000).unwrap();
    assert_eq!(one.report(), two.report());
}

#[test]
fn battlefield_monte_carlo_matches_exact_free_equipment_draws() {
    let mut runner = BattlefieldRunner::new(scenario(), 20_000, 2026).unwrap();
    while !runner.finished() {
        runner.advance(512).unwrap();
    }
    let report = runner.report().unwrap();
    for turn in &report.turns {
        let exact = hypergeometric(60, 4, 7 + u16::from(turn.turn) - 1, 1, 4).unwrap();
        assert!(
            (turn.joint.probability - exact).abs() < 0.015,
            "turn {}: {} vs {exact}",
            turn.turn,
            turn.joint.probability
        );
        assert_eq!(turn.joint, turn.equipment);
        assert_eq!(turn.joint.samples, 20_000);
        assert!(
            turn.joint.interval95[0] <= turn.joint.probability
                && turn.joint.interval95[1] >= turn.joint.probability
        );
        assert_eq!(turn.decked, 0);
    }
    assert_eq!(report.seed, 2026);
}

#[test]
fn invalid_run_turn_and_goal_bounds_fail_before_simulating() {
    for runs in [0, 1_000_001] {
        assert!(BattlefieldRunner::new(scenario(), runs, 1).is_err());
    }
    for turns in [0, 31] {
        let mut s = scenario();
        s.turns = turns;
        assert!(BattlefieldRunner::new(s, 1, 1).is_err());
    }
    let mut s = scenario();
    s.goal.equipment = 1001;
    assert!(BattlefieldRunner::new(s, 1, 1).is_err());
}

#[test]
fn joint_attachment_probability_matches_multivariate_sampling() {
    let mut s = scenario();
    let mut deck = s.deck.to_vec();
    for c in &mut deck[4..8] {
        c.id = 3;
        c.kind = Kind::Spell(SpellSpec {
            permanent: true,
            creature: true,
            haste: true,
            ..Default::default()
        });
    }
    for c in &mut deck[..4] {
        let Kind::Spell(spell) = &mut c.kind else {
            unreachable!()
        };
        spell.equip_cost = Some(ManaCost::default());
    }
    s.deck = Arc::from(deck);
    s.turns = 1;
    s.goal = Goal {
        equipment: 1,
        attached: 1,
        attack_ready: 1,
    };
    let mut runner = BattlefieldRunner::new(s, 20_000, 13).unwrap();
    while !runner.finished() {
        runner.advance(512).unwrap();
    }
    let report = runner.report().unwrap();
    let first = &report.turns[0];
    let exact = 1.0 - 2.0 * hypergeometric(60, 4, 7, 0, 0).unwrap()
        + hypergeometric(60, 8, 7, 0, 0).unwrap();
    assert!((first.joint.probability - exact).abs() < 0.015);
    assert!(first.joint.probability <= first.equipment.probability);
    assert_eq!(first.joint, first.attached);
    assert_eq!(first.joint, first.attack_ready);
}

#[test]
fn mulligan_statistics_report_actual_retained_sizes_and_forced_keeps() {
    let mut s = scenario();
    s.mulligan = London {
        minimum_keep: 4,
        force_mulligans: 3,
        required_ids: vec![999],
        ..London::never()
    };
    let mut runner = BattlefieldRunner::new(s, 10, 1).unwrap();
    runner.advance(10).unwrap();
    let report = runner.report().unwrap();
    assert_eq!(report.mean_opening_size, 4.0);
    assert_eq!(report.mean_mulligans, 3.0);
    assert_eq!(report.forced_keeps, 10);
}

#[test]
fn more_legendary_copies_do_not_become_two_equipment_permanents() {
    let mut s = scenario();
    let mut cards = s.deck.to_vec();
    for c in &mut cards[..4] {
        c.legendary = true;
    }
    s.deck = Arc::from(cards);
    s.turns = 1;
    s.goal.equipment = 2;
    let mut runner = BattlefieldRunner::new(s, 20_000, 14).unwrap();
    while !runner.finished() {
        runner.advance(512).unwrap();
    }
    let report = runner.report().unwrap();
    let first = &report.turns[0];
    assert_eq!(first.equipment.successes, 0);
    assert_eq!(first.joint.successes, 0);
    let have_one = hypergeometric(60, 4, 7, 1, 4).unwrap();
    assert!((first.mean_equipment - have_one).abs() < 0.015);
    assert!((first.mean_legendary_bricks - (7.0 * 4.0 / 60.0 - have_one)).abs() < 0.015);
}

#[test]
fn reports_include_triggered_draws_creatures_and_story_sample_counts() {
    let mut s = scenario();
    let mut cards = s.deck[..8].to_vec();
    for c in &mut cards[..4] {
        c.legendary = true;
        c.keep_priority = 10;
        c.tags.dwarf = true;
        c.kind = Kind::Spell(SpellSpec {
            permanent: true,
            creature: true,
            entry_draw: Some(EntryDraw {
                allow_self: false,
                once_per_turn: true,
                discard: DiscardPolicy::None,
            }),
            ..Default::default()
        });
    }
    for c in &mut cards[4..] {
        c.kind = Kind::Spell(SpellSpec {
            permanent: true,
            artifact: true,
            equipment: true,
            equip_cost: Some(ManaCost::default()),
            ..Default::default()
        });
    }
    s.deck = Arc::from(cards);
    s.turns = 1;
    let mut runner = BattlefieldRunner::new(s, 20, 15).unwrap();
    runner.advance(20).unwrap();
    let report = runner.report().unwrap();
    let first = &report.turns[0];
    assert_eq!(first.mean_creatures, 1.0);
    assert_eq!(first.mean_cards_drawn, 1.0);
    assert_eq!(first.mean_cards_discarded, 0.0);
    assert_eq!(first.story_samples, 20);
}
