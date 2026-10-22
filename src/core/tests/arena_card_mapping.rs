mod support;
use alkemyst_core::{
    catalog::Catalog,
    db::{Engine, Statement},
};
use serde_json::json;
use support::{cards, source};

#[tokio::test]
async fn local_asset_resolves_only_exact_printing_and_preserves_primary_mappings() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("cards.sqlite");
    let asset = rusqlite::Connection::open(&path).unwrap();
    asset.execute_batch("CREATE TABLE Cards(GrpId INTEGER,TitleId INTEGER,ExpansionCode TEXT,CollectorNumber TEXT,CollectorSuffix TEXT); CREATE TABLE Localizations_enUS(LocId INTEGER,Formatted INTEGER,Loc TEXT);
        INSERT INTO Cards VALUES(100,1,'TST','1',''),(101,2,'TST','1',''),(42,3,'TST','3',''),(102,4,'TST','99','');
        INSERT INTO Localizations_enUS VALUES(1,1,'Forgeblade'),(2,1,'Different Card'),(3,1,'Ember // Meadow'),(4,1,'Forgeblade');").unwrap();
    drop(asset);
    let original = std::fs::read(&path).unwrap();
    let db = Engine::memory(1024 * 1024).unwrap();
    let catalog = Catalog::new(db.clone());
    catalog
        .import_jsonl(std::io::Cursor::new(cards()), source())
        .await
        .unwrap();
    assert!(
        catalog
            .resolve_arena(&[100, 101, 42, 102, 999])
            .await
            .unwrap()[0]
            .card
            .is_none()
    );
    let aliases = catalog.import_arena_database(&path).await.unwrap();
    assert_eq!(aliases, 4);
    let result = catalog
        .resolve_arena(&[100, 101, 42, 102, 999])
        .await
        .unwrap();
    assert_eq!(result[0].card.as_ref().unwrap()["name"], "Forgeblade");
    assert_eq!(
        result[0].resolved_by.as_deref(),
        Some("arena_asset_printing")
    );
    assert!(
        result[1].card.is_none(),
        "different name cannot resolve by set/number alone"
    );
    assert_eq!(result[2].card.as_ref().unwrap()["name"], "Forgeblade");
    assert_eq!(result[2].resolved_by.as_deref(), Some("scryfall_arena_id"));
    assert!(
        result[3].card.is_none(),
        "same name is insufficient without exact printing"
    );
    assert!(result[4].card.is_none());
    assert_eq!(
        std::fs::read(&path).unwrap(),
        original,
        "source database untouched"
    );
    let wrong = Statement {
        name: "test.sourceWrite",
        sql: "DELETE FROM Cards",
        reads: &[],
        cacheable: false,
    };
    assert!(db.read_source(&path, wrong).await.is_err());
    let read = Statement {
        sql: "SELECT json_array(count(*)) FROM Cards",
        ..wrong
    };
    assert_eq!(db.read_source(&path, read).await.unwrap(), vec![json!([4])]);
}
