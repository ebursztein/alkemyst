use alkemyst_desktop::{
    CloseAction, ExitAction, Lifecycle, NativeRuntime, RuntimeOptions, Tracking,
};
#[test]
fn closing_hides_until_explicit_quit_and_shutdown_is_started_only_once() {
    let lifecycle = Lifecycle::default();
    assert_eq!(lifecycle.close_action(), CloseAction::Hide);
    assert_eq!(lifecycle.exit_action(), ExitAction::BeginShutdown);
    assert_eq!(lifecycle.close_action(), CloseAction::Allow);
    assert_eq!(lifecycle.exit_action(), ExitAction::Wait);
    lifecycle.complete_shutdown();
    assert_eq!(lifecycle.exit_action(), ExitAction::Exit);
}
#[tokio::test]
async fn native_service_bootstrap_is_authenticated_and_quit_stops_the_same_service() {
    let dir = tempfile::tempdir().unwrap();
    let runtime = NativeRuntime::start(RuntimeOptions {
        database: dir.path().join("native.sqlite3"),
        tracking: Tracking::Disabled,
        port: 0,
        development: true,
    })
    .await
    .unwrap();
    let config = runtime.bootstrap().unwrap();
    assert!(config.base_url.starts_with("http://127.0.0.1:"));
    assert_eq!(config.token.len(), 64);
    let client = reqwest::Client::new();
    assert_eq!(
        client
            .get(format!("{}/v1/decks", config.base_url))
            .send()
            .await
            .unwrap()
            .status(),
        401
    );
    let response = client
        .get(format!("{}/v1/decks", config.base_url))
        .bearer_auth(&config.token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let (a, b) = tokio::join!(runtime.stop(), runtime.stop());
    a.unwrap();
    b.unwrap();
    assert!(runtime.bootstrap().is_err());
    assert!(
        client
            .get(format!("{}/v1/health", config.base_url))
            .send()
            .await
            .is_err()
    );
}
#[tokio::test]
async fn tracking_and_mcp_remain_active_without_any_window_and_pause_does_not_stop_the_api() {
    let dir = tempfile::tempdir().unwrap();
    let current = dir.path().join("Player.log");
    std::fs::write(
        &current,
        include_str!("../../../core/tests/fixtures/arena-game.txt"),
    )
    .unwrap();
    let runtime = NativeRuntime::start(RuntimeOptions {
        database: dir.path().join("native.sqlite3"),
        tracking: Tracking::Files {
            previous: dir.path().join("Player-prev.log"),
            current,
        },
        port: 0,
        development: true,
    })
    .await
    .unwrap();
    let config = runtime.bootstrap().unwrap();
    let client = reqwest::Client::new();
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        loop {
            let value: serde_json::Value = client
                .get(format!("{}/v1/capture", config.base_url))
                .bearer_auth(&config.token)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            if value["games"].as_u64().unwrap_or(0) > 0 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    runtime.set_tracking_paused(true).await.unwrap();
    assert!(runtime.tracking_paused().await);
    let initialize = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": { "name": "native-test", "version": "1" }
        }
    });
    let response = client
        .post(format!("{}/mcp", config.base_url))
        .bearer_auth(&config.token)
        .header("Accept", "application/json, text/event-stream")
        .json(&initialize)
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    runtime.set_tracking_paused(false).await.unwrap();
    assert!(!runtime.tracking_paused().await);
    runtime.stop().await.unwrap();
}
