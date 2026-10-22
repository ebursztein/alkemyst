use landlord::{Card, London, SimulationRng, sample_hand};

fn deck(n: usize) -> Vec<Card> {
    (0..n)
        .map(|i| Card {
            id: i as u32,
            is_land: i < 24,
            keep_priority: 0,
        })
        .collect()
}

#[test]
fn seeded_london_replays_and_bottomed_cards_are_at_the_actual_library_end() {
    let deck = deck(60);
    let policy = London {
        minimum_keep: 4,
        force_mulligans: 3,
        ..London::never()
    };
    let hand = sample_hand(&deck, &policy, &mut SimulationRng::seeded(42)).unwrap();
    assert_eq!(
        hand,
        sample_hand(&deck, &policy, &mut SimulationRng::seeded(42)).unwrap()
    );
    assert_ne!(
        hand,
        sample_hand(&deck, &policy, &mut SimulationRng::seeded(43)).unwrap()
    );
    assert_eq!(
        (hand.kept.len(), hand.bottomed.len(), hand.mulligans),
        (4, 3, 3)
    );
    assert_eq!(&hand.library[hand.library.len() - 3..], hand.bottomed);
    let mut seen = hand
        .kept
        .iter()
        .chain(&hand.library)
        .copied()
        .collect::<Vec<_>>();
    seen.sort_unstable();
    assert_eq!(seen, (0..60).collect::<Vec<_>>());
    assert_eq!(hand.draws(100).len(), 56);
    assert!(!hand.draws(53).iter().any(|i| hand.bottomed.contains(i)));
}

#[test]
fn policy_retains_required_cards_and_lands_and_stops_at_floor() {
    let deck = deck(60);
    let policy = London {
        minimum_keep: 5,
        min_lands: 2,
        max_lands: 4,
        required_ids: vec![59],
        ..London::never()
    };
    for seed in 0..50 {
        let hand = sample_hand(&deck, &policy, &mut SimulationRng::seeded(seed)).unwrap();
        if !hand.forced_keep {
            assert!(hand.kept.iter().any(|i| deck[*i].id == 59));
            assert!((2..=4).contains(&hand.kept.iter().filter(|i| deck[**i].is_land).count()));
        }
        assert!(hand.kept.len() >= 5);
    }
    let impossible = London {
        minimum_keep: 0,
        min_lands: 7,
        max_lands: 7,
        required_ids: vec![999],
        ..London::never()
    };
    let hand = sample_hand(&deck, &impossible, &mut SimulationRng::seeded(1)).unwrap();
    assert_eq!(hand.kept.len(), 0);
    assert!(hand.forced_keep);
}

#[test]
fn small_decks_exhaust_without_underflow_and_invalid_policies_fail() {
    for n in 1..=8 {
        let d = deck(n);
        let hand = sample_hand(&d, &London::never(), &mut SimulationRng::seeded(0)).unwrap();
        assert_eq!(hand.kept.len(), n.min(7));
        assert_eq!(hand.draws(99).len(), n.saturating_sub(7));
    }
    assert!(sample_hand(&[], &London::never(), &mut SimulationRng::seeded(0)).is_err());
    for p in [
        London {
            minimum_keep: 8,
            ..London::never()
        },
        London {
            min_lands: 5,
            max_lands: 1,
            ..London::never()
        },
        London {
            force_mulligans: 1,
            ..London::never()
        },
    ] {
        assert!(sample_hand(&deck(60), &p, &mut SimulationRng::seeded(0)).is_err());
    }
}

#[test]
fn bottom_policy_prefers_higher_priority_after_required_cards_and_lands() {
    let mut d = deck(7);
    for c in &mut d {
        c.is_land = false;
        c.keep_priority = c.id as i16;
    }
    let p = London {
        minimum_keep: 3,
        force_mulligans: 4,
        ..London::never()
    };
    let hand = sample_hand(&d, &p, &mut SimulationRng::seeded(1)).unwrap();
    let mut ids = hand.kept.iter().map(|i| d[*i].id).collect::<Vec<_>>();
    ids.sort_unstable();
    assert_eq!(ids, vec![4, 5, 6]);
}

#[test]
fn bottoming_can_meet_maximum_lands_when_the_drawn_seven_allow_it() {
    let mut d = deck(7);
    for (i, c) in d.iter_mut().enumerate() {
        c.is_land = i < 6;
        c.keep_priority = if c.is_land { 10 } else { 0 };
    }
    let p = London {
        minimum_keep: 4,
        force_mulligans: 3,
        min_lands: 2,
        max_lands: 3,
        ..London::never()
    };
    for seed in 0..20 {
        let hand = sample_hand(&d, &p, &mut SimulationRng::seeded(seed)).unwrap();
        assert!(
            !hand.forced_keep,
            "valid 3-land/1-spell retained hand exists"
        );
        assert_eq!(hand.kept.iter().filter(|i| d[**i].is_land).count(), 3);
    }
}
