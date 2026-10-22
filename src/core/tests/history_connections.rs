mod history_support;
use alkemyst_core::{
    db::Engine,
    history::{History, MissingData},
    ingest::Ingestor,
};
use history_support::{deck, start, state};
use serde_json::json;
fn transition(old: &str, new: &str) -> String {
    format!(
        "[UnityCrossThreadLogger]STATE CHANGED {}\n",
        json!({"old":old,"new":new})
    )
}
#[tokio::test]
async fn healthy_transitions_and_post_game_teardown_are_not_missing_capture_data() {
    for (before, between, after, interrupted) in [
        (
            transition("None", "ConnectedToMatchDoor"),
            transition("Playing", "MatchCompleted"),
            transition("MatchCompleted", "Disconnected"),
            false,
        ),
        (
            String::new(),
            transition("Playing", "Disconnected"),
            String::new(),
            true,
        ),
        (
            String::new(),
            String::new(),
            transition("Playing", "Disconnected"),
            false,
        ),
    ] {
        let db = Engine::memory(4096).unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("Player.log");
        std::fs::write(
            &path,
            deck()
                + &start("connection", "Ladder")
                + &before
                + &state("connection", 1, 1, 2, None)
                + &between
                + &state("connection", 2, 1, 2, Some(20))
                + &after,
        )
        .unwrap();
        Ingestor::new(db.clone()).import_file(&path).await.unwrap();
        let game = History::new(db)
            .games(Default::default())
            .await
            .unwrap()
            .items
            .remove(0);
        assert_eq!(
            game.missing_data.contains(&MissingData::Connection),
            interrupted,
            "{:?}",
            game.missing_data
        );
        assert_eq!(game.capture_complete, !interrupted);
    }
}
