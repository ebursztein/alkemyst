mod mcp_support;
use alkemyst_core::{
    domain::{CreateDeck, DeckCard, Origin, Zone},
    practice::{NewPractice, PracticeAction},
};
use mcp_support::{ClientHarness, app};
use serde_json::{Value, json};
#[tokio::test]
async fn rest_and_mcp_share_practice_steps_cache_and_agent_provenance() {
    let (db, app) = app();
    let v = app
        .decks
        .create(
            "practice-transport",
            CreateDeck {
                name: "Manual sandbox".into(),
                format: "test".into(),
                cards: vec![DeckCard {
                    reference: "oracle:unresolved".into(),
                    quantity: 60,
                    zone: Zone::Main,
                }],
            },
            Origin::User,
        )
        .await
        .unwrap();
    let h = ClientHarness::start(app).await;
    let created=h.tool("practiceCreate",json!({"idempotency_key":"agent-practice","input":NewPractice{version_id:v.id,on_the_play:true,seed:Some("123456789abcdef0".into())}})).await;
    let initial = &created["result"]["structuredContent"]["data"];
    let id = initial["id"].as_str().unwrap();
    assert_eq!(initial["origin"], "agent");
    assert_eq!(initial["rules_mode"], "manual_sandbox");
    assert!(initial.get("library").is_none());
    assert_eq!(initial["library_count"], 53);
    let before = db.metrics().await.unwrap();
    assert_eq!(h.rest(&format!("/v1/practice/{id}")).await, *initial);
    assert!(db.metrics().await.unwrap().statements <= before.statements + 1);
    let response=h.client.post(format!("{}/v1/practice/{id}/actions",h.base)).bearer_auth(&h.token)
        .json(&json!({"idempotency_key":"rest-keep","expected_revision":0,"action":PracticeAction::Keep{bottom:vec![]}})).send().await.unwrap();
    assert_eq!(response.status(), 200);
    let kept: Value = response.json().await.unwrap();
    assert_eq!(kept["last_action_origin"], "user");
    assert_eq!(kept["revision"], 1);
    assert_eq!(h.rest(&format!("/v1/practice/{id}")).await, kept);
    let before = db.metrics().await.unwrap().cache_hits;
    assert_eq!(
        h.tool("practiceGet", json!({"id":id})).await["result"]["structuredContent"]["data"],
        kept
    );
    assert!(db.metrics().await.unwrap().cache_hits > before);
    let next=h.tool("practiceAct",json!({"id":id,"idempotency_key":"agent-next","expected_revision":1,"action":{"kind":"next_turn"}})).await;
    assert_eq!(
        next["result"]["structuredContent"]["data"]["turn"], 2,
        "{next}"
    );
    assert_eq!(
        next["result"]["structuredContent"]["data"]["last_action_origin"],
        "agent"
    );
    assert_eq!(
        h.tool("practiceInputsGet", json!({"id":id})).await["result"]["structuredContent"]["data"],
        h.rest(&format!("/v1/practice/{id}/inputs")).await
    );
    assert_eq!(
        h.tool("practiceList", json!({"limit":2})).await["result"]["structuredContent"]["data"],
        h.rest("/v1/practice?limit=2").await
    );
    h.server.stop().await.unwrap();
}
