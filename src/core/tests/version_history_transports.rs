mod mcp_support;
use alkemyst_core::domain::{CreateDeck, DeckCard, Origin, Zone};
use mcp_support::{ClientHarness, app};
use serde_json::json;
#[tokio::test]
async fn version_history_matches_across_rest_and_mcp_and_reuses_the_engine_cache() {
    let (db, app) = app();
    let v = app
        .decks
        .create(
            "version-transport",
            CreateDeck {
                name: "Version history".into(),
                format: "test".into(),
                cards: vec![DeckCard {
                    reference: "oracle:unknown".into(),
                    quantity: 60,
                    zone: Zone::Main,
                }],
            },
            Origin::User,
        )
        .await
        .unwrap();
    let h = ClientHarness::start(app).await;
    let rest = h
        .rest(&format!("/v1/decks/{}/versions?limit=2", v.deck_id))
        .await;
    assert_eq!(rest["items"][0]["id"], v.id);
    let before = db.metrics().await.unwrap().cache_hits;
    let result = h
        .tool("deckVersionsList", json!({"id":v.deck_id,"limit":2}))
        .await;
    assert_eq!(
        result["result"]["structuredContent"]["data"], rest,
        "{result}"
    );
    assert!(db.metrics().await.unwrap().cache_hits > before);
    assert_eq!(
        h.tool("deckVersionsList", json!({"id":"missing"})).await["result"]["structuredContent"]["code"],
        "not_found"
    );
    h.server.stop().await.unwrap();
}
