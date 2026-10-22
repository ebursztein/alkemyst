use alkemyst_core::analysis::probabilities::{
    Category, DrawProbability, JointProbability, draw, joint,
};
#[test]
fn exact_methods_preserve_joint_events_and_reject_invalid_categories() {
    let one = draw(DrawProbability {
        population: 60,
        successes: 4,
        draws: 7,
        min: 1,
        max: 4,
    })
    .unwrap();
    assert!((one.probability - 0.3994996257442573).abs() < 1e-12);
    let value = joint(JointProbability {
        draws: 2,
        categories: vec![
            Category {
                label: "Equipment".into(),
                population: 2,
                min: 1,
                max: 1,
            },
            Category {
                label: "Creature".into(),
                population: 2,
                min: 1,
                max: 1,
            },
            Category {
                label: "Other".into(),
                population: 2,
                min: 0,
                max: 2,
            },
        ],
    })
    .unwrap();
    assert!((value.probability - 4.0 / 15.0).abs() < 1e-12);
    assert!(
        draw(DrawProbability {
            population: 2,
            successes: 3,
            draws: 1,
            min: 0,
            max: 1
        })
        .is_err()
    );
    assert!(
        joint(JointProbability {
            draws: 0,
            categories: vec![Category {
                label: "".into(),
                population: 0,
                min: 0,
                max: 0
            }]
        })
        .is_err()
    );
}
