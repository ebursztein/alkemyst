use landlord::probability::{Category, hypergeometric, joint_probability};

#[test]
fn elementary_draws_and_joint_without_replacement_match_enumeration() {
    assert!((hypergeometric(60, 4, 7, 1, 4).unwrap() - (1.0 - 0.6005003742557427)).abs() < 1e-12);
    // Six cards: two equipment, two creatures, two other. Drawing two:
    // one equipment AND one creature has 2*2 / choose(6,2) = 4/15.
    let categories = [
        Category {
            population: 2,
            min: 1,
            max: 1,
        },
        Category {
            population: 2,
            min: 1,
            max: 1,
        },
        Category {
            population: 2,
            min: 0,
            max: 2,
        },
    ];
    assert!((joint_probability(2, &categories).unwrap() - 4.0 / 15.0).abs() < 1e-12);
    // Marginals are correlated; multiplying their individual probabilities is wrong.
    let marginal = hypergeometric(6, 2, 2, 1, 2).unwrap();
    assert!((marginal * marginal - 4.0 / 15.0).abs() > 0.05);
}

#[test]
fn probabilities_normalize_at_boundaries_and_large_decks() {
    for (n, k, d) in [
        (0, 0, 0),
        (60, 0, 7),
        (60, 60, 7),
        (1000, 500, 500),
        (10, 3, 10),
    ] {
        let distribution = (0..=d)
            .map(|x| hypergeometric(n, k, d, x, x).unwrap())
            .sum::<f64>();
        assert!(
            (distribution - 1.0).abs() < 1e-10,
            "{n}/{k}/{d}: {distribution}"
        );
    }
    assert_eq!(hypergeometric(60, 0, 7, 1, 7).unwrap(), 0.0);
    assert_eq!(joint_probability(0, &[]).unwrap(), 1.0);
    assert!(hypergeometric(4, 5, 2, 0, 2).is_err());
    assert!(hypergeometric(4, 2, 5, 0, 2).is_err());
    assert!(hypergeometric(4, 2, 2, 2, 1).is_err());
    assert!(hypergeometric(1001, 1, 1, 0, 1).is_err());
    assert!(
        joint_probability(
            1,
            &[Category {
                population: 1,
                min: 2,
                max: 1
            }]
        )
        .is_err()
    );
    assert!(
        joint_probability(
            0,
            &[Category {
                population: 0,
                min: 0,
                max: 0
            }; 17]
        )
        .is_err()
    );
}
