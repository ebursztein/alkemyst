mod support;
use alkemyst_core::{
    analysis::Analysis,
    catalog::Catalog,
    db::Engine,
    domain::{CreateDeck, DeckCard, Decks, Origin, Zone},
};
use serde_json::{Value, json};

#[tokio::test]
async fn printed_curves_use_the_front_type_and_exclude_only_front_lands() {
    let db = Engine::memory(4 * 1024 * 1024).unwrap();
    let mut records = support::cards()
        .lines()
        .map(|s| serde_json::from_str::<Value>(s).unwrap())
        .collect::<Vec<_>>();
    let mut land = records[0].clone();
    land["id"] = json!("print-land");
    land["oracle_id"] = json!("oracle-land");
    land["name"] = json!("Plains");
    land["type_line"] = json!("Basic Land — Plains");
    land["cmc"] = json!(0);
    records.push(land);
    Catalog::new(db.clone())
        .import_jsonl(
            std::io::Cursor::new(
                records
                    .iter()
                    .map(Value::to_string)
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            support::source(),
        )
        .await
        .unwrap();
    let v = Decks::new(db.clone())
        .create(
            "front-types",
            CreateDeck {
                name: "Printed curve".into(),
                format: "test".into(),
                cards: vec![
                    DeckCard {
                        reference: "oracle:oracle-modal".into(),
                        quantity: 2,
                        zone: Zone::Main,
                    },
                    DeckCard {
                        reference: "oracle:oracle-land".into(),
                        quantity: 3,
                        zone: Zone::Main,
                    },
                ],
            },
            Origin::User,
        )
        .await
        .unwrap();
    let analysis = Analysis::new(db);
    let comparison = analysis.compare(&v.id, &v.id).await.unwrap().unwrap();
    assert_eq!(comparison.curve.len(), 1);
    assert_eq!(comparison.curve[0].mana_value, Some(3.0));
    assert_eq!(comparison.curve[0].base_quantity, 2);
    let snapshot = analysis.snapshot(&v.id).await.unwrap().unwrap();
    assert_eq!(snapshot.counts.lands, 3);
    assert_eq!(snapshot.curve.len(), 1);
    assert_eq!(snapshot.curve[0].quantity, 2);
}

#[tokio::test]
async fn comparison_resolves_large_inputs_with_one_statement_and_indexed_lookups() {
    use alkemyst_core::db::Statement;
    let db = Engine::memory(4 * 1024 * 1024).unwrap();
    let v = Decks::new(db.clone())
        .create(
            "large-compare",
            CreateDeck {
                name: "Large comparison".into(),
                format: "test".into(),
                cards: (0..500)
                    .map(|i| DeckCard {
                        reference: format!("oracle:missing-{i}"),
                        quantity: 2,
                        zone: Zone::Main,
                    })
                    .collect(),
            },
            Origin::User,
        )
        .await
        .unwrap();
    let analysis = Analysis::new(db.clone());
    let before = db.metrics().await.unwrap().statements;
    let result = analysis.compare(&v.id, &v.id).await.unwrap().unwrap();
    assert_eq!(db.metrics().await.unwrap().statements, before + 1);
    assert_eq!(result.base.unresolved_main, 1000);
    assert_eq!(result.curve[0].base_quantity, 1000);
    assert!(result.cards.is_empty());
    let plan = db
        .explain(
            Statement {
                name: "test.comparison.plan",
                sql: concat!(
                    include_str!("../src/statements/comparison/head.sql"),
                    include_str!("../src/statements/cards/reference_from.sql"),
                    include_str!("../src/statements/comparison/tail.sql")
                ),
                reads: &[],
                cacheable: false,
            },
            vec![json!(v.id), json!(v.id)],
        )
        .await
        .unwrap()
        .join("\n");
    assert!(plan.contains("card_arena"), "{plan}");
    assert!(
        plan.contains("SEARCH p USING INTEGER PRIMARY KEY"),
        "{plan}"
    );
    assert!(!plan.contains("SCAN candidate"), "{plan}");
}
