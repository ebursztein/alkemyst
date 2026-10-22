use alkemyst_core::{
    db::Engine,
    domain::{CreateDeck, DeckCard, Decks, Origin, Zone},
};
use alkemyst_desktop::{NativeRuntime, RuntimeOptions, Tracking};
use std::path::Path;

async fn seed(path: &Path, name: &str) {
    let db = Engine::open(path, 4096).unwrap();
    Decks::new(db.clone())
        .create(
            "fixture",
            CreateDeck {
                name: name.into(),
                format: "test".into(),
                cards: vec![DeckCard {
                    reference: "oracle:equipment".into(),
                    quantity: 4,
                    zone: Zone::Main,
                }],
            },
            Origin::User,
        )
        .await
        .unwrap();
    db.close().await.unwrap();
}

async fn start(path: &Path) -> NativeRuntime {
    NativeRuntime::start(RuntimeOptions {
        database: path.to_owned(),
        tracking: Tracking::Disabled,
        port: 0,
        development: false,
    })
    .await
    .unwrap()
}

async fn names(runtime: &NativeRuntime) -> Vec<String> {
    let connection = runtime.bootstrap().unwrap();
    let value: serde_json::Value = reqwest::Client::new()
        .get(format!("{}/v1/decks", connection.base_url))
        .bearer_auth(connection.token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    value["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["name"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn selection_is_inert_until_confirmed_and_cancel_keeps_the_current_library() {
    let dir = tempfile::tempdir().unwrap();
    let current = dir.path().join("alkemyst.sqlite3");
    let selected = dir.path().join("selected.sqlite3");
    seed(&current, "Current deck").await;
    seed(&selected, "Saved deck").await;
    let runtime = start(&current).await;
    runtime.prepare_restore(&selected).await.unwrap();
    assert_eq!(names(&runtime).await, ["Current deck"]);
    runtime.cancel_restore().await;
    assert!(runtime.confirm_restore().await.is_err());
    runtime.prepare_restore(&selected).await.unwrap();
    runtime.stop().await.unwrap();
    drop(runtime);
    let reopened = start(&current).await;
    assert_eq!(names(&reopened).await, ["Current deck"]);
    reopened.stop().await.unwrap();
}

#[tokio::test]
async fn confirmed_restore_applies_before_api_start_and_keeps_a_recoverable_safety_backup() {
    let dir = tempfile::tempdir().unwrap();
    let current = dir.path().join("alkemyst.sqlite3");
    let selected = dir.path().join("selected.sqlite3");
    seed(&current, "Current deck").await;
    seed(&selected, "Saved deck").await;
    let runtime = start(&current).await;
    let exported = dir.path().join("exported.sqlite3");
    runtime.backup_to(&exported).await.unwrap();
    runtime.prepare_restore(&selected).await.unwrap();
    runtime.confirm_restore().await.unwrap();
    assert_eq!(names(&runtime).await, ["Current deck"]);
    runtime.stop().await.unwrap();
    drop(runtime);
    let restored = start(&current).await;
    assert_eq!(names(&restored).await, ["Saved deck"]);
    let status = serde_json::to_value(restored.restore_status()).unwrap();
    assert_eq!(status["state"], "restored");
    let safety = Path::new(status["safety_backup"].as_str().unwrap());
    let db = Engine::open(safety, 4096).unwrap();
    assert_eq!(
        Decks::new(db.clone()).list(10, None).await.unwrap().items[0].name,
        "Current deck"
    );
    db.close().await.unwrap();
    restored.stop().await.unwrap();
    drop(restored);
    let second = start(&current).await;
    assert_eq!(
        serde_json::to_value(second.restore_status()).unwrap()["state"],
        "none"
    );
    assert_eq!(names(&second).await, ["Saved deck"]);
    second.stop().await.unwrap();
}

#[tokio::test]
async fn a_damaged_pending_restore_is_quarantined_and_the_current_library_still_opens() {
    let dir = tempfile::tempdir().unwrap();
    let current = dir.path().join("alkemyst.sqlite3");
    let selected = dir.path().join("selected.sqlite3");
    seed(&current, "Current deck").await;
    seed(&selected, "Saved deck").await;
    let runtime = start(&current).await;
    runtime.prepare_restore(&selected).await.unwrap();
    runtime.confirm_restore().await.unwrap();
    runtime.stop().await.unwrap();
    drop(runtime);
    let pending = dir.path().join("pending-restore.sqlite3");
    std::fs::write(&pending, b"interrupted or damaged snapshot").unwrap();
    let reopened = start(&current).await;
    assert_eq!(names(&reopened).await, ["Current deck"]);
    assert_eq!(
        serde_json::to_value(reopened.restore_status()).unwrap()["state"],
        "failed"
    );
    assert!(!pending.exists());
    reopened.stop().await.unwrap();
}
