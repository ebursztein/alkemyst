mod mcp_support;
use alkemyst_core::domain::{CreateDeck, DeckCard, Origin, Zone};
use mcp_support::{ClientHarness, app};
use serde_json::json;

#[tokio::test]
async fn comparison_is_shared_by_rest_and_allowlisted_mcp() {
    let (db, app) = app();
    let version = app
        .decks
        .create(
            "transport-compare",
            CreateDeck {
                name: "Local comparison".into(),
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
    let path = format!("/v1/deck-versions/{}/compare/{}", version.id, version.id);
    let rest = h.rest(&path).await;
    assert_eq!(rest["base"]["unresolved_main"], 60);
    assert_eq!(rest["cards"], json!([]));
    let before = db.metrics().await.unwrap();
    let result = h
        .tool(
            "deckVersionsCompare",
            json!({"id":version.id,"candidate_id":version.id}),
        )
        .await;
    assert_eq!(result["result"]["structuredContent"]["data"], rest);
    assert!(db.metrics().await.unwrap().cache_hits > before.cache_hits);
    assert_eq!(
        h.tool(
            "deckVersionsCompare",
            json!({"id":version.id,"candidate_id":"missing"})
        )
        .await["result"]["structuredContent"]["code"],
        "not_found"
    );
    h.server.stop().await.unwrap();
}
