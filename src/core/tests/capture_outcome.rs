use alkemyst_core::domain::ObservedOutcome;

#[test]
fn observed_outcomes_require_an_explicit_result_and_valid_team_mapping() {
    use ObservedOutcome::{Draw, Lost, Won};
    let win_loss = Some("ResultType_WinLoss");
    assert_eq!(
        ObservedOutcome::from_result(win_loss, Some(20), Some(20)),
        Some(Won)
    );
    assert_eq!(
        ObservedOutcome::from_result(win_loss, Some(10), Some(20)),
        Some(Lost)
    );
    assert_eq!(
        ObservedOutcome::from_result(Some("ResultType_Draw"), None, None),
        Some(Draw)
    );
    for (kind, winner, team) in [
        (None, Some(1), Some(2)),
        (win_loss, None, Some(2)),
        (win_loss, Some(0), Some(2)),
        (win_loss, Some(1), None),
        (win_loss, Some(1), Some(0)),
        (Some("new-result-type"), Some(1), Some(2)),
    ] {
        assert_eq!(ObservedOutcome::from_result(kind, winner, team), None);
    }
}
