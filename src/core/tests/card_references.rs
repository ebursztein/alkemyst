mod support;
use alkemyst_core::{catalog::Catalog, db::Engine};
use support::{cards, source};

#[tokio::test]
async fn mixed_card_references_resolve_in_order_with_one_prepared_read() {
    let db = Engine::memory(1024 * 1024).unwrap();
    let catalog = Catalog::new(db.clone());
    catalog
        .import_jsonl(std::io::Cursor::new(cards()), source())
        .await
        .unwrap();
    let refs = [
        "oracle:oracle-equipment",
        "scryfall:print-tutor",
        "arena:42",
        "arena:999",
        "unknown",
    ]
    .map(str::to_string);
    let before = db.metrics().await.unwrap().statements;
    let rows = catalog.resolve_references(&refs).await.unwrap();
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0].card.as_ref().unwrap()["name"], "Forgeblade");
    assert_eq!(rows[1].card.as_ref().unwrap()["name"], "Find the Blade");
    assert_eq!(rows[2].card.as_ref().unwrap()["name"], "Forgeblade");
    assert!(rows[3].card.is_none());
    assert!(rows[4].card.is_none());
    assert_eq!(db.metrics().await.unwrap().statements - before, 1);
    assert!(
        catalog
            .resolve_references(&vec!["a".into(); 1001])
            .await
            .is_err()
    );
    assert!(
        catalog
            .resolve_references(&["a".repeat(201)])
            .await
            .is_err()
    );
}
