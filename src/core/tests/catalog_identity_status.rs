mod support;
use alkemyst_core::{catalog::Catalog, db::Engine};

#[tokio::test]
async fn installed_identities_are_visible_before_metadata_and_invalidate_warm_status() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cards.sqlite3");
    let asset = rusqlite::Connection::open(&path).unwrap();
    asset.execute_batch("CREATE TABLE Cards(GrpId INTEGER,TitleId INTEGER,ExpansionCode TEXT,CollectorNumber TEXT,CollectorSuffix TEXT);
        CREATE TABLE Localizations_enUS(LocId INTEGER,Formatted INTEGER,Loc TEXT);
        INSERT INTO Cards VALUES(100,1,'TST','1','');
        INSERT INTO Localizations_enUS VALUES(1,1,'Forgeblade');").unwrap();
    asset.close().unwrap();
    let original = std::fs::read(&path).unwrap();
    let db = Engine::memory(4096).unwrap();
    let catalog = Catalog::new(db.clone());
    let empty = catalog.status().await.unwrap();
    assert_eq!(empty.arena_aliases, 0);
    let before = db.metrics().await.unwrap();
    catalog.status().await.unwrap();
    assert_eq!(
        db.metrics().await.unwrap().cache_hits,
        before.cache_hits + 1
    );
    catalog.import_arena_database(&path).await.unwrap();
    let installed = catalog.status().await.unwrap();
    assert_eq!(
        installed.cards, 0,
        "Arena identities do not substitute for Scryfall rules"
    );
    assert_eq!(installed.arena_aliases, 1);
    assert!(installed.arena_alias_revision > empty.arena_alias_revision);
    assert!(
        catalog.resolve_arena(&[100]).await.unwrap()[0]
            .card
            .is_none()
    );
    catalog
        .import_jsonl(std::io::Cursor::new(support::cards()), support::source())
        .await
        .unwrap();
    let result = catalog.resolve_arena(&[100]).await.unwrap();
    assert_eq!(result[0].card.as_ref().unwrap()["name"], "Forgeblade");
    assert_eq!(std::fs::read(&path).unwrap(), original);
    db.close().await.unwrap();
}
