use alkemyst_core::{Application, agent, db::Engine};
use serde_json::{Value, json};

#[tokio::test]
async fn direct_agent_calls_share_cache_receipts_and_immutable_versions() {
    let db = Engine::memory(1024 * 1024).unwrap();
    let app = Application::new(db.clone());
    let first_read = app.invoke_agent("decksList", json!({})).await.unwrap();
    let before = db.metrics().await.unwrap();
    assert_eq!(
        app.invoke_agent("decksList", json!({})).await.unwrap(),
        first_read
    );
    assert_eq!(
        db.metrics().await.unwrap().cache_hits,
        before.cache_hits + 1
    );
    let command = json!({"idempotency_key":"agent-create","input":{
        "name":"Equipment", "format":"standard", "cards":[]
    }});
    let created = app
        .invoke_agent("decksCreate", command.clone())
        .await
        .unwrap();
    assert_eq!(created["data"]["origin"], "agent");
    assert_eq!(
        app.invoke_agent("decksCreate", command).await.unwrap(),
        created
    );
    let original = &created["data"];
    let edited = app
        .invoke_agent(
            "deckVersionCreate",
            json!({
                "idempotency_key":"agent-edit", "input":{
                    "deck_id":original["deck_id"], "parent_version":original["id"],
                    "name":"Equipment revised", "format":"standard", "cards":[]
                }
            }),
        )
        .await
        .unwrap();
    assert_eq!(edited["data"]["ordinal"], 2);
    assert_eq!(
        app.invoke_agent("deckVersionGet", json!({"id":original["id"]}))
            .await
            .unwrap(),
        created
    );
    let listed = app.invoke_agent("decksList", json!({})).await.unwrap();
    assert_eq!(listed["data"]["items"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn invalid_or_unlisted_calls_never_reach_storage() {
    let db = Engine::memory(4096).unwrap();
    let app = Application::new(db.clone());
    let before = db.metrics().await.unwrap();
    for (name, args, expected) in [
        ("arbitrarySQL", json!({}), "not_found"),
        ("practiceAct", Value::Null, "invalid_request"),
        (
            "decksList",
            json!({"sql":"DELETE FROM decks"}),
            "invalid_request",
        ),
        ("decksList", json!({"limit":101}), "invalid_request"),
        ("decksCreate", json!({"input":{}}), "invalid_request"),
    ] {
        let error = app.invoke_agent(name, args).await.err().unwrap();
        assert_eq!(
            serde_json::to_value(error.problem).unwrap()["code"],
            expected
        );
    }
    assert_eq!(db.metrics().await.unwrap().statements, before.statements);
}

#[test]
fn mcp_uses_the_same_agent_contracts_and_capability_hints() {
    let mcp = alkemyst_core::mcp::schema::tools();
    let shared = agent::tools();
    assert_eq!(mcp.len(), shared.len());
    for (mcp, shared) in mcp.iter().zip(shared) {
        assert_eq!(mcp.name, shared.name);
        assert_eq!(
            mcp.description.as_deref(),
            Some(shared.description.as_str())
        );
        assert_eq!(mcp.input_schema.as_ref(), &shared.input_schema);
        assert_eq!(
            mcp.output_schema.as_ref().unwrap().as_ref(),
            &shared.output_schema
        );
        let hints = mcp.annotations.as_ref().unwrap();
        assert_eq!(hints.read_only_hint, Some(shared.read_only));
        assert_eq!(hints.open_world_hint, Some(shared.open_world));
    }
}
