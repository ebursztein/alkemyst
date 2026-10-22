mod card_setup_support;
use alkemyst_desktop::{CardCacheState, CardSources, NativeRuntime};
use card_setup_support::{asset, call, checked, metadata, options};

#[tokio::test]
async fn cold_start_imports_installed_identities_without_network_or_source_modification() {
    let dir = tempfile::tempdir().unwrap();
    let raw = dir.path().join("Raw");
    let source = raw.join("Raw_CardDatabase_fixture.mtga");
    asset(&source, false);
    let before = std::fs::read(&source).unwrap();
    let runtime = NativeRuntime::start_with_card_sources(
        options(&dir.path().join("app.sqlite3")),
        CardSources::Directories(vec![raw]),
    )
    .await
    .unwrap();
    assert_eq!(checked(&runtime, 0).await.state, CardCacheState::Ready);
    let status = call(&runtime, "/v1/catalog", None).await;
    assert_eq!(status["arena_aliases"], 1);
    assert_eq!(
        status["cards"], 0,
        "installed identities must not masquerade as full metadata"
    );
    assert!(call(&runtime, "/v1/cards/resolve-arena", Some(&[100])).await[0]["card"].is_null());
    runtime.stop().await.unwrap();
    assert_eq!(std::fs::read(source).unwrap(), before);
}

#[tokio::test]
async fn unchanged_sources_are_reused_and_new_cache_versions_invalidate_resolved_reads() {
    let dir = tempfile::tempdir().unwrap();
    let database = dir.path().join("app.sqlite3");
    metadata(&database).await;
    let raw = dir.path().join("Raw");
    let old = raw.join("Raw_CardDatabase_old.mtga");
    asset(&old, false);
    std::fs::OpenOptions::new()
        .write(true)
        .open(&old)
        .unwrap()
        .set_modified(std::time::UNIX_EPOCH)
        .unwrap();
    let runtime = NativeRuntime::start_with_card_sources(
        options(&database),
        CardSources::Directories(vec![raw.clone()]),
    )
    .await
    .unwrap();
    let health = checked(&runtime, 0).await;
    assert_eq!(health.state, CardCacheState::Ready);
    assert_eq!(
        call(&runtime, "/v1/cards/resolve-arena", Some(&[100])).await[0]["card"]["name"],
        "Forgeblade"
    );
    let revision = call(&runtime, "/v1/catalog", None).await["arena_alias_revision"].clone();
    runtime.retry_card_cache().await.unwrap();
    let health = checked(&runtime, health.checks).await;
    assert_eq!(health.state, CardCacheState::Ready);
    assert_eq!(
        call(&runtime, "/v1/catalog", None).await["arena_alias_revision"],
        revision
    );
    asset(&raw.join("Raw_CardDatabase_new.mtga"), true);
    runtime.retry_card_cache().await.unwrap();
    assert_eq!(
        checked(&runtime, health.checks).await.state,
        CardCacheState::Ready
    );
    assert_eq!(
        call(&runtime, "/v1/cards/resolve-arena", Some(&[100])).await[0]["card"]["name"],
        "Find the Blade"
    );
    runtime.stop().await.unwrap();
    assert!(runtime.retry_card_cache().await.is_err());
}

#[tokio::test]
async fn missing_and_invalid_caches_keep_the_app_available_and_manual_selection_can_recover() {
    let dir = tempfile::tempdir().unwrap();
    let database = dir.path().join("app.sqlite3");
    metadata(&database).await;
    let runtime = NativeRuntime::start_with_card_sources(
        options(&database),
        CardSources::Directories(vec![dir.path().join("missing")]),
    )
    .await
    .unwrap();
    let health = checked(&runtime, 0).await;
    assert_eq!(health.state, CardCacheState::Missing);
    assert_eq!(call(&runtime, "/v1/catalog", None).await["cards"], 3);
    let valid = dir.path().join("custom.sqlite3");
    asset(&valid, false);
    runtime.choose_card_cache(valid).await.unwrap();
    let health = checked(&runtime, health.checks).await;
    assert_eq!(health.state, CardCacheState::Ready);
    let invalid = dir.path().join("invalid.sqlite3");
    std::fs::write(&invalid, b"incomplete download").unwrap();
    runtime.choose_card_cache(invalid).await.unwrap();
    let failed = checked(&runtime, health.checks).await;
    assert_eq!(failed.state, CardCacheState::Degraded);
    assert!(
        !serde_json::to_string(&failed)
            .unwrap()
            .contains(&dir.path().to_string_lossy().to_string())
    );
    assert_eq!(
        call(&runtime, "/v1/cards/resolve-arena", Some(&[100])).await[0]["card"]["name"],
        "Forgeblade"
    );
    runtime.stop().await.unwrap();
}
