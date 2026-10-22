mod support;
use alkemyst_core::{
    catalog::{Catalog, CatalogSource, Search},
    db::Engine,
};
use serde_json::{Value, json};
use support::{cards, source};

#[tokio::test]
async fn catalog_import_search_and_batch_resolution_use_one_statement_per_read() {
    let db = Engine::memory(1024 * 1024).unwrap();
    let catalog = Catalog::new(db.clone());
    assert_eq!(catalog.status().await.unwrap().cards, 0);
    let status = catalog
        .import_jsonl(std::io::Cursor::new(cards()), source())
        .await
        .unwrap();
    assert_eq!(
        (status.cards, status.printings, status.arena_ids),
        (3, 4, 2)
    );
    let before = db.metrics().await.unwrap();
    let page = catalog
        .search(Search {
            text: "equipment".into(),
            type_line: Some("Equipment".into()),
            ..Search::default()
        })
        .await
        .unwrap();
    assert_eq!(page.items.len(), 1);
    assert_eq!(page.items[0].name, "Forgeblade");
    let after = db.metrics().await.unwrap();
    assert_eq!(after.statements - before.statements, 1);
    catalog
        .search(Search {
            text: "equipment".into(),
            type_line: Some("Equipment".into()),
            ..Search::default()
        })
        .await
        .unwrap();
    assert_eq!(db.metrics().await.unwrap().cache_hits, after.cache_hits + 1);
    let resolved = catalog.resolve_arena(&[42, 999, 43]).await.unwrap();
    assert_eq!(resolved.len(), 3);
    assert_eq!(resolved[0].card.as_ref().unwrap()["name"], "Forgeblade");
    assert!(resolved[1].card.is_none());
    assert_eq!(resolved[2].card.as_ref().unwrap()["name"], "Find the Blade");
    assert_eq!(db.metrics().await.unwrap().statements - after.statements, 1);
    let detail = catalog.card("oracle-equipment").await.unwrap().unwrap();
    assert_eq!(detail["custom_public_field"], json!({"preserved":true}));
    assert!(catalog.card("absent").await.unwrap().is_none());
}

#[tokio::test]
async fn refresh_publishes_atomically_and_old_cache_survives_failed_import() {
    let db = Engine::memory(1024 * 1024).unwrap();
    let catalog = Catalog::new(db.clone());
    catalog
        .import_jsonl(std::io::Cursor::new(cards()), source())
        .await
        .unwrap();
    let old = catalog.search(Search::default()).await.unwrap();
    let broken = format!("{}\n{{broken", cards());
    assert!(
        catalog
            .import_jsonl(std::io::Cursor::new(broken), source())
            .await
            .is_err()
    );
    assert_eq!(
        catalog.search(Search::default()).await.unwrap().items,
        old.items
    );
    let mut replacement: Value = serde_json::from_str(cards().lines().next().unwrap()).unwrap();
    replacement["name"] = json!("New Equipment");
    let status = catalog
        .import_jsonl(
            std::io::Cursor::new(replacement.to_string()),
            CatalogSource {
                updated_at: "2026-10-22T00:00:00Z".into(),
                ..source()
            },
        )
        .await
        .unwrap();
    assert_eq!(status.cards, 1);
    let new = catalog.search(Search::default()).await.unwrap();
    assert_eq!(new.items.len(), 1);
    assert_eq!(new.items[0].name, "New Equipment");
    assert_ne!(new.items, old.items);
}
