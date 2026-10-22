mod support;
use alkemyst_core::{
    analysis::Analysis,
    catalog::Catalog,
    db::Engine,
    domain::{CreateDeck, DeckCard, Decks, Origin, Zone},
};
use serde_json::Value;

async fn setup() -> (Engine, Catalog, String) {
    let db = Engine::memory(4 * 1024 * 1024).unwrap();
    let catalog = Catalog::new(db.clone());
    let mut records = support::cards()
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .collect::<Vec<_>>();
    let mut plains = records[0].clone();
    plains["id"] = "print-plains".into();
    plains["oracle_id"] = "oracle-plains".into();
    plains["name"] = "Plains".into();
    plains["cmc"] = 0.into();
    plains["mana_cost"] = "".into();
    plains["type_line"] = "Basic Land — Plains".into();
    plains["oracle_text"] = "({T}: Add {W}.)".into();
    plains["colors"] = serde_json::json!([]);
    records.push(plains);
    let data = records
        .iter()
        .map(Value::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    catalog
        .import_jsonl(std::io::Cursor::new(data), support::source())
        .await
        .unwrap();
    let version = Decks::new(db.clone())
        .create(
            "profile-input",
            CreateDeck {
                name: "Analysis test".into(),
                format: "standard".into(),
                cards: vec![
                    DeckCard {
                        reference: "oracle:oracle-equipment".into(),
                        quantity: 4,
                        zone: Zone::Main,
                    },
                    DeckCard {
                        reference: "scryfall:print-plains".into(),
                        quantity: 2,
                        zone: Zone::Main,
                    },
                    DeckCard {
                        reference: "arena:999999".into(),
                        quantity: 1,
                        zone: Zone::Main,
                    },
                    DeckCard {
                        reference: "oracle:missing-sideboard".into(),
                        quantity: 3,
                        zone: Zone::Sideboard,
                    },
                ],
            },
            Origin::User,
        )
        .await
        .unwrap();
    (db, catalog, version.id)
}

#[tokio::test]
async fn snapshot_joins_cards_and_computes_printed_curve_in_one_cached_statement() {
    let (db, _, version) = setup().await;
    let analysis = Analysis::new(db.clone());
    let before = db.metrics().await.unwrap();
    let input = analysis.snapshot(&version).await.unwrap().unwrap();
    assert_eq!(input.version_id, version);
    assert!(input.catalog_snapshot.is_some());
    assert_eq!(
        (
            input.counts.main,
            input.counts.lands,
            input.counts.equipment,
            input.counts.unresolved_main
        ),
        (7, 2, 4, 1)
    );
    assert_eq!(input.counts.sideboard, 3);
    assert_eq!(input.cards.len(), 4);
    assert!(
        input
            .cards
            .iter()
            .find(|c| c.reference == "arena:999999")
            .unwrap()
            .data
            .is_none()
    );
    assert!(
        input
            .curve
            .iter()
            .any(|b| b.mana_value == Some(2.0) && b.quantity == 4)
    );
    assert!(
        input
            .curve
            .iter()
            .any(|b| b.mana_value.is_none() && b.quantity == 1)
    );
    let warm = analysis.snapshot(&version).await.unwrap().unwrap();
    assert_eq!(input, warm);
    let after = db.metrics().await.unwrap();
    assert_eq!(after.statements - before.statements, 1);
    assert_eq!(after.cache_hits - before.cache_hits, 1);
}
#[tokio::test]
async fn snapshot_keeps_missing_catalog_missing_and_validates_identifiers() {
    let db = Engine::memory(4096).unwrap();
    let analysis = Analysis::new(db.clone());
    assert!(analysis.snapshot("").await.is_err());
    assert!(analysis.snapshot(&"v".repeat(257)).await.is_err());
    assert_eq!(db.metrics().await.unwrap().statements, 0);
    assert!(analysis.snapshot("absent").await.unwrap().is_none());
}

#[tokio::test]
async fn profile_uses_the_same_single_snapshot_read_and_preserves_unsupported_cards() {
    let (db, _, version) = setup().await;
    let analysis = Analysis::new(db.clone());
    let before = db.metrics().await.unwrap().statements;
    let profile = analysis.profile(&version).await.unwrap().unwrap();
    assert_eq!(db.metrics().await.unwrap().statements, before + 1);
    assert!(!profile.can_simulate);
    assert_eq!(profile.counts.main, 7);
    let white = profile.sources.iter().find(|s| s.color == "W").unwrap();
    assert_eq!(white.unrestricted_lands, 2);
}

#[tokio::test]
async fn larger_decks_do_not_add_per_card_fetches_and_resolution_is_indexed() {
    use alkemyst_core::db::Statement;
    let db = Engine::memory(4 * 1024 * 1024).unwrap();
    let decks = Decks::new(db.clone());
    let cards = (0..500)
        .map(|i| DeckCard {
            reference: format!("oracle:missing-{i}"),
            quantity: 2,
            zone: Zone::Main,
        })
        .collect();
    let v = decks
        .create(
            "large-analysis",
            CreateDeck {
                name: "Large missing catalog".into(),
                format: "test".into(),
                cards,
            },
            Origin::User,
        )
        .await
        .unwrap();
    let analysis = Analysis::new(db.clone());
    let before = db.metrics().await.unwrap().statements;
    let input = analysis.snapshot(&v.id).await.unwrap().unwrap();
    assert_eq!(input.cards.len(), 500);
    assert_eq!(input.counts.unresolved_main, 1000);
    assert_eq!(db.metrics().await.unwrap().statements, before + 1);
    let plan = db
        .explain(
            Statement {
                name: "test.analysis.plan",
                sql: concat!(
                    include_str!("../src/statements/analysis/input_head.sql"),
                    include_str!("../src/statements/cards/reference_from.sql"),
                    include_str!("../src/statements/analysis/input_tail.sql")
                ),
                reads: &[],
                cacheable: false,
            },
            vec![serde_json::json!(v.id)],
        )
        .await
        .unwrap()
        .join("\n");
    assert!(plan.contains("card_arena"), "{plan}");
    assert!(plan.contains("SEARCH p USING"), "{plan}");
    assert!(!plan.contains("SCAN candidate"), "{plan}");
}
