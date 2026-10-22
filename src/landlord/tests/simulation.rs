use landlord::{
    Card, London,
    probability::hypergeometric,
    simulation::{DrawConfig, simulate_draws},
};

fn deck() -> Vec<Card> {
    (0..60)
        .map(|i| Card {
            id: if i < 4 { 1 } else { 2 },
            is_land: i >= 36,
            keep_priority: 0,
        })
        .collect()
}

#[test]
fn monte_carlo_is_seeded_streaming_and_agrees_with_exact_elementary_case() {
    let config = DrawConfig {
        runs: 50_000,
        seed: 2026,
        draws: 3,
        targets: vec![1],
        mulligan: London::never(),
    };
    let report = simulate_draws(&deck(), &config).unwrap();
    assert_eq!(report, simulate_draws(&deck(), &config).unwrap());
    assert_eq!(report.histogram.iter().sum::<u32>(), config.runs);
    let observed = 1.0 - report.histogram[0] as f64 / config.runs as f64;
    let exact = hypergeometric(60, 4, 10, 1, 4).unwrap();
    assert!((observed - exact).abs() < 0.01, "{observed} vs {exact}");
    assert_eq!(report.mean_opening_size, 7.0);
    assert_eq!(report.mean_mulligans, 0.0);
    assert_eq!(report.forced_keeps, 0);
    let changed = simulate_draws(
        &deck(),
        &DrawConfig {
            seed: 2027,
            ..config
        },
    )
    .unwrap();
    assert_ne!(report.histogram, changed.histogram);
}

#[test]
fn london_draws_preserve_bottoming_and_invalid_runs_are_errors() {
    let cards = vec![
        Card {
            id: 1,
            is_land: false,
            keep_priority: 0
        };
        7
    ];
    let config = DrawConfig {
        runs: 10,
        seed: 0,
        draws: 99,
        targets: vec![1, 1],
        mulligan: London {
            minimum_keep: 4,
            force_mulligans: 3,
            ..London::never()
        },
    };
    let report = simulate_draws(&cards, &config).unwrap();
    assert_eq!(report.histogram[7], 10);
    assert_eq!(report.mean_opening_size, 4.0);
    assert_eq!(report.mean_mulligans, 3.0);
    for runs in [0, 1_000_001] {
        assert!(
            simulate_draws(
                &cards,
                &DrawConfig {
                    runs,
                    ..config.clone()
                }
            )
            .is_err()
        );
    }
    assert!(simulate_draws(&[], &config).is_err());
    assert!(
        simulate_draws(
            &cards,
            &DrawConfig {
                draws: 1001,
                ..config.clone()
            }
        )
        .is_err()
    );
}
