use alkemyst_core::analysis::turns::*;
use landlord::{
    SimulationRng,
    mana::{Color, ManaCost, Source},
};
use std::sync::Arc;
fn filler(id: u32) -> CardSpec {
    CardSpec {
        id,
        cost: ManaCost::default(),
        printed_mana_value: 0,
        legendary: false,
        keep_priority: 0,
        tags: Default::default(),
        adventure: None,
        kind: Kind::Ignored,
    }
}
fn accepts(cards: Vec<CardSpec>, hand: Vec<usize>, library: Vec<usize>) -> bool {
    Trial::new(Arc::from(cards), hand, library, true).is_ok()
}

#[test]
fn malformed_partitions_and_ambiguous_card_definitions_are_rejected() {
    assert!(!accepts(vec![], vec![], vec![]));
    assert!(!accepts(vec![filler(1); 1001], vec![], (0..1001).collect()));
    assert!(!accepts(vec![filler(1); 8], (0..8).collect(), vec![]));
    assert!(!accepts(vec![filler(1)], vec![], vec![]));
    assert!(!accepts(vec![filler(1), filler(2)], vec![0], vec![0]));
    assert!(!accepts(vec![filler(1)], vec![2], vec![]));
    let mut different = filler(1);
    different.keep_priority = 1;
    assert!(!accepts(vec![filler(1), different], vec![0], vec![1]));
}

#[test]
fn invalid_modeled_abilities_fail_instead_of_being_silently_ignored() {
    for spell in [
        SpellSpec {
            tutor: Some(TutorSpec {
                scope: SearchScope::Library,
                types: SearchTypes::Equipment,
                timing: TutorTiming::EnterBattlefield,
                destination: Destination::Hand,
                max_mana_value: None,
            }),
            ..Default::default()
        },
        SpellSpec {
            equipment: true,
            ..Default::default()
        },
        SpellSpec {
            creature: true,
            ..Default::default()
        },
        SpellSpec {
            haste: true,
            ..Default::default()
        },
        SpellSpec {
            equip_cost: Some(ManaCost::default()),
            ..Default::default()
        },
        SpellSpec {
            artifact: true,
            ..Default::default()
        },
    ] {
        let mut c = filler(1);
        c.kind = Kind::Spell(spell);
        assert!(!accepts(vec![c], vec![0], vec![]));
    }
    let mut c = filler(1);
    c.legendary = true;
    c.kind = Kind::Spell(SpellSpec::default());
    assert!(!accepts(vec![c], vec![0], vec![]));
    for (cost, basic_types, enters) in [
        ("{1}", 0, Enters::Untapped),
        ("", 32, Enters::Untapped),
        ("", 0, Enters::HasBasicType(0)),
        ("", 0, Enters::HasBasicType(32)),
    ] {
        let mut c = filler(1);
        c.cost = ManaCost::parse(cost).unwrap();
        c.kind = Kind::Land(LandSpec {
            source: Source::new(&[Color::White]).unwrap(),
            enters,
            restriction: Restriction::Any,
            basic_types,
            conditional: vec![],
        });
        assert!(!accepts(vec![c], vec![0], vec![]));
    }
}

#[test]
fn empty_library_is_explicit_and_turns_are_bounded() {
    let mut trial = Trial::new(Arc::from(vec![filler(1)]), vec![0], vec![], false).unwrap();
    let mut rng = SimulationRng::seeded(1);
    let one = trial.next_turn(&mut rng).unwrap();
    assert!(one.decked);
    assert!(!Goal::default().met(&one));
    for _ in 1..30 {
        assert!(trial.next_turn(&mut rng).unwrap().decked);
    }
    assert!(trial.next_turn(&mut rng).is_err());
}

#[test]
fn invalid_conditional_sources_and_equip_modifiers_are_rejected() {
    let ability = ManaAbility {
        source: Source::new(&[Color::White]).unwrap(),
        restriction: Restriction::Any,
        condition: Condition::HasBasicType(0),
    };
    let mut c = filler(1);
    c.kind = Kind::Spell(SpellSpec {
        permanent: true,
        mana_ability: Some(ability.clone()),
        ..Default::default()
    });
    assert!(!accepts(vec![c], vec![0], vec![]));
    let mut c = filler(1);
    c.tags.colors = 32;
    assert!(!accepts(vec![c], vec![0], vec![]));
    for s in [
        SpellSpec {
            worthy_only: true,
            ..Default::default()
        },
        SpellSpec {
            equip_reduction_per_color: true,
            ..Default::default()
        },
        SpellSpec {
            story_free_equip: true,
            ..Default::default()
        },
        SpellSpec {
            target_equip_reduction: 2,
            ..Default::default()
        },
    ] {
        let mut c = filler(1);
        c.kind = Kind::Spell(s);
        assert!(!accepts(vec![c], vec![0], vec![]));
    }
    for conditional in [
        vec![ability],
        vec![
            ManaAbility {
                source: Source::new(&[Color::White]).unwrap(),
                restriction: Restriction::Any,
                condition: Condition::Always
            };
            9
        ],
    ] {
        let mut c = filler(1);
        c.kind = Kind::Land(LandSpec {
            source: Source::new(&[Color::White]).unwrap(),
            enters: Enters::Untapped,
            restriction: Restriction::Any,
            basic_types: 0,
            conditional,
        });
        assert!(!accepts(vec![c], vec![0], vec![]));
    }
}

#[test]
fn search_and_adventure_events_must_have_consistent_timing() {
    let search = TutorSpec {
        scope: SearchScope::Top(0),
        types: SearchTypes::Equipment,
        timing: TutorTiming::SpellResolves,
        destination: Destination::Hand,
        max_mana_value: None,
    };
    let mut c = filler(1);
    c.kind = Kind::Spell(SpellSpec {
        tutor: Some(search.clone()),
        ..Default::default()
    });
    assert!(!accepts(vec![c], vec![0], vec![]));
    let mut c = filler(1);
    c.adventure = Some(Adventure {
        cost: ManaCost::default(),
        tutor: search,
    });
    assert!(!accepts(vec![c], vec![0], vec![]));
    let mut c = filler(1);
    c.kind = Kind::Spell(SpellSpec {
        entry_draw: Some(EntryDraw {
            allow_self: true,
            once_per_turn: false,
            discard: DiscardPolicy::None,
        }),
        ..Default::default()
    });
    assert!(!accepts(vec![c], vec![0], vec![]));
}
