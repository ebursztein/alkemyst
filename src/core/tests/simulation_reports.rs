use alkemyst_core::analysis::simulation::{MODEL_VERSION, POLICY, SimulationReport};
use alkemyst_core::analysis::turns::Goal;
#[test]
fn reports_round_trip_full_width_seeds_without_javascript_precision_loss() {
    let report = SimulationReport {
        requested_runs: 1,
        completed_runs: 1,
        seed: u64::MAX,
        replay_version: landlord::REPLAY_VERSION.into(),
        model_version: MODEL_VERSION.into(),
        policy: POLICY.into(),
        goal: Goal::default(),
        mean_opening_size: 7.0,
        mean_mulligans: 0.0,
        forced_keeps: 0,
        turns: vec![],
    };
    let encoded = serde_json::to_value(&report).unwrap();
    assert_eq!(encoded["seed"], "ffffffffffffffff");
    let decoded: SimulationReport = serde_json::from_value(encoded.clone()).unwrap();
    assert_eq!(decoded, report);
    for bad in [
        serde_json::json!(42),
        serde_json::json!("abc"),
        serde_json::json!("zzzzzzzzzzzzzzzz"),
    ] {
        let mut encoded = encoded.clone();
        encoded["seed"] = bad;
        assert!(serde_json::from_value::<SimulationReport>(encoded).is_err());
    }
}
