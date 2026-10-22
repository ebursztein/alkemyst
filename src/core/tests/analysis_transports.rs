mod mcp_support;
use alkemyst_core::{
    catalog::CatalogSource,
    domain::{CreateDeck, DeckCard, Origin, Zone},
};
use mcp_support::{ClientHarness, app};
use serde_json::{Value, json};

#[tokio::test]
async fn rest_and_mcp_profiles_share_sql_cache_and_catalog_invalidation() {
    let (db, app) = app();
    let catalog = app.catalog.clone();
    let source = CatalogSource {
        url: "https://data.scryfall.io/default-cards/fixture.jsonl.gz".into(),
        updated_at: "2026-10-21T21:00:00Z".into(),
    };
    catalog
        .import_jsonl(
            std::io::Cursor::new(include_str!("fixtures/cards.jsonl").to_owned()),
            source.clone(),
        )
        .await
        .unwrap();
    let v = app
        .decks
        .create(
            "profile-transport",
            CreateDeck {
                name: "Test profile".into(),
                format: "standard".into(),
                cards: vec![DeckCard {
                    reference: "oracle:oracle-equipment".into(),
                    quantity: 4,
                    zone: Zone::Main,
                }],
            },
            Origin::User,
        )
        .await
        .unwrap();
    let h = ClientHarness::start(app).await;
    let path = format!("/v1/deck-versions/{}/analysis", v.id);
    let profile = h.rest(&path).await;
    assert_eq!(profile["counts"]["equipment"], 4);
    let before = db.metrics().await.unwrap().cache_hits;
    let rpc = h.tool("analysisProfileGet", json!({"id":v.id})).await;
    assert_eq!(rpc["result"]["structuredContent"]["data"], profile);
    assert_eq!(db.metrics().await.unwrap().cache_hits, before + 1);
    let updated = include_str!("fixtures/cards.jsonl")
        .lines()
        .map(|line| {
            let mut value: Value = serde_json::from_str(line).unwrap();
            if value["oracle_id"] == "oracle-equipment" {
                value["cmc"] = json!(3);
            }
            value.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    catalog
        .import_jsonl(
            std::io::Cursor::new(updated),
            CatalogSource {
                updated_at: "2026-10-22T00:00:00Z".into(),
                ..source
            },
        )
        .await
        .unwrap();
    let after=h.tool("analysisProfileGet",json!({"id":v.id})).await["result"]["structuredContent"]["data"].clone();
    assert_ne!(after["catalog_snapshot"], profile["catalog_snapshot"]);
    assert_eq!(after["curve"][0]["mana_value"], 3.0);
    assert_eq!(h.rest(&path).await, after);
    assert_eq!(
        h.rest("/v1/deck-versions/absent/analysis").await["code"],
        "not_found"
    );
    assert_eq!(
        h.tool("analysisProfileGet", json!({"id":"absent"})).await["result"]["structuredContent"]["code"],
        "not_found"
    );
    let tools = h.rpc("tools/list", Value::Null).await;
    assert!(
        tools["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool["name"] == "analysisProfileGet")
    );
    db.close().await.unwrap();
    assert_eq!(h.rest(&path).await["code"], "busy");
    assert_eq!(
        h.tool("analysisProfileGet", json!({"id":v.id})).await["result"]["structuredContent"]["code"],
        "busy"
    );
    h.server.stop().await.unwrap();
}
