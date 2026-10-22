mod mcp_support;
use mcp_support::{ClientHarness, app};
use serde_json::json;

#[tokio::test]
async fn rest_and_mcp_share_cache_versions_and_receipts() {
    let (db, app) = app();
    let h = ClientHarness::start(app).await;
    let warm = h.rest("/v1/decks").await;
    assert_eq!(warm["items"], json!([]));
    let before = db.metrics().await.unwrap();
    let warm_mcp = h.tool("decksList", json!({})).await;
    assert_eq!(warm_mcp["result"]["structuredContent"]["data"], warm);
    assert_eq!(
        db.metrics().await.unwrap().cache_hits,
        before.cache_hits + 1
    );
    let command = json!({"idempotency_key":"shared","input":{"name":"Equipment","format":"standard","cards":[]}});
    let created = h.tool("decksCreate", command.clone()).await;
    let first = created["result"]["structuredContent"]["data"].clone();
    assert_eq!(first["origin"], "agent");
    let retry: serde_json::Value = h
        .client
        .post(format!("{}/v1/decks", h.base))
        .bearer_auth(&h.token)
        .json(&command)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(retry, first);
    assert_eq!(
        h.rest("/v1/decks").await["items"].as_array().unwrap().len(),
        1
    );
    let version=h.tool("deckVersionCreate",json!({"idempotency_key":"edit","input":{"deck_id":first["deck_id"],"parent_version":first["id"],"name":"Equipment v2","format":"standard","cards":[]}})).await;
    assert_eq!(version["result"]["structuredContent"]["data"]["ordinal"], 2);
    assert_eq!(
        h.rest(&format!(
            "/v1/deck-versions/{}",
            first["id"].as_str().unwrap()
        ))
        .await,
        first
    );
    let read = h.tool("deckVersionGet", json!({"id":first["id"]})).await;
    assert_eq!(read["result"]["structuredContent"]["data"], first);
    let note=h.tool("notesCreate",json!({"idempotency_key":"note","input":{"deck_id":first["deck_id"],"version_id":first["id"],"body":"More tutors?"}})).await;
    assert_eq!(
        note["result"]["structuredContent"]["data"]["origin"],
        "agent"
    );
    let notes = h
        .tool("notesList", json!({"deck_id":first["deck_id"]}))
        .await;
    assert_eq!(
        notes["result"]["structuredContent"]["data"]["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let discovery = h.tool("systemDiscovery", json!({})).await;
    assert_eq!(
        discovery["result"]["structuredContent"]["data"]["local_only"],
        true
    );
    for (tool, args) in [
        ("decksList", json!({"sql":"DELETE FROM decks"})),
        ("deckVersionGet", json!({})),
        ("decksCreate", json!({"input":{}})),
    ] {
        let error = h.tool(tool, args).await;
        assert_eq!(error["result"]["isError"], true, "{error}");
        assert_eq!(
            error["result"]["structuredContent"]["code"],
            "invalid_request"
        );
    }
    assert!(h.tool("sql", json!({})).await["error"].is_object());
    let missing = h.tool("deckVersionGet", json!({"id":"absent"})).await;
    assert_eq!(missing["result"]["structuredContent"]["code"], "not_found");
    h.server.stop().await.unwrap();
}

#[tokio::test]
async fn card_tools_use_offline_catalog_and_match_rest_metadata() {
    let (_, app) = app();
    app.catalog
        .import_jsonl(
            std::io::Cursor::new(include_str!("fixtures/cards.jsonl")),
            alkemyst_core::catalog::CatalogSource {
                url: "https://data.scryfall.io/default-cards/fixture.jsonl.gz".into(),
                updated_at: "2026-10-21T21:00:00Z".into(),
            },
        )
        .await
        .unwrap();
    let h = ClientHarness::start(app).await;
    let tools = alkemyst_core::mcp::schema::tools();
    for (name, args) in [
        ("cardsSearch", json!({"text":"forge"})),
        ("cardsGet", json!({"oracle_id":"oracle-modal"})),
        ("cardsResolveArena", json!({"ids":[42,999]})),
        (
            "cardsResolveReferences",
            json!({"references":["oracle:oracle-tutor"]}),
        ),
        ("catalogStatusGet", json!({})),
    ] {
        let result = h.tool(name, args).await;
        assert_eq!(result["result"]["isError"], false, "{result}");
        let schema = tools
            .iter()
            .find(|tool| tool.name == name)
            .unwrap()
            .output_schema
            .as_ref()
            .unwrap();
        let validator =
            jsonschema::validator_for(&serde_json::Value::Object(schema.as_ref().clone())).unwrap();
        assert!(
            validator.is_valid(&result["result"]["structuredContent"]),
            "{result}"
        );
        if name == "cardsGet" {
            assert_eq!(
                result["result"]["structuredContent"]["data"],
                h.rest("/v1/cards/oracle-modal").await
            );
        }
    }
    let imported=h.tool("deckArenaImport",json!({"idempotency_key":"import","input":{"name":"Boros","format":"standard","text":"Deck\n4 Forgeblade"}})).await;
    let version = &imported["result"]["structuredContent"]["data"];
    assert_eq!(version["origin"], "agent");
    let exported = h.tool("deckArenaExport", json!({"id":version["id"]})).await;
    assert!(
        exported["result"]["structuredContent"]["data"]["text"]
            .as_str()
            .unwrap()
            .contains("4 Forgeblade")
    );
    let unknown=h.tool("deckArenaImport",json!({"idempotency_key":"unknown","input":{"name":"Boros","format":"standard","text":"Deck\n4 Missing"}})).await;
    assert_eq!(
        unknown["result"]["structuredContent"]["code"],
        "unresolved_cards"
    );
    h.server.stop().await.unwrap();
}
