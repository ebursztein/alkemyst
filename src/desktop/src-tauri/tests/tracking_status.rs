use alkemyst_core::ingest::TrackingState;
use alkemyst_desktop::{NativeRuntime, RuntimeOptions, Tracking};

#[tokio::test]
async fn native_status_distinguishes_waiting_tracking_and_paused_without_leaking_paths() {
    let dir = tempfile::tempdir().unwrap();
    let current = dir.path().join("Player.log");
    let runtime = NativeRuntime::start(RuntimeOptions {
        database: dir.path().join("health.sqlite3"),
        tracking: Tracking::Files {
            previous: dir.path().join("Player-prev.log"),
            current: current.clone(),
        },
        port: 0,
        development: false,
    })
    .await
    .unwrap();
    assert_eq!(
        runtime.tracking_status().await.health.state,
        TrackingState::Waiting
    );
    std::fs::write(&current, "").unwrap();
    tokio::time::timeout(std::time::Duration::from_secs(5), async {
        while runtime.tracking_status().await.health.state != TrackingState::Tracking {
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    runtime.set_tracking_paused(true).await.unwrap();
    assert!(runtime.tracking_paused().await);
    assert_eq!(
        runtime.tracking_status().await.health.state,
        TrackingState::Stopped
    );
    assert!(runtime.tracking_status().await.health.last_error.is_none());
    let serialized = serde_json::to_string(&runtime.tracking_status().await.health).unwrap();
    assert!(!serialized.contains(&dir.path().to_string_lossy().to_string()));
    runtime.stop().await.unwrap();
}
