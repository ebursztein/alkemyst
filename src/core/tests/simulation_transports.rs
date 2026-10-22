#[path = "simulation_jobs/support.rs"]
mod data;
#[path = "mcp_support/client.rs"]
mod mcp_client;
use alkemyst_core::Application;
use mcp_client::ClientHarness;
use serde_json::{Value, json};
#[tokio::test]
async fn rest_sdk_contract_and_mcp_share_persisted_jobs_and_exact_math() {
    let (db, _, version) = data::setup().await;
    let h = ClientHarness::start(Application::new(db.clone())).await;
    let input = data::parameters(&version);
    let command = json!({"idempotency_key":"job-transport","input":input});
    let response = h
        .client
        .post(format!("{}/v1/simulations", h.base))
        .bearer_auth(&h.token)
        .json(&command)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 202);
    let created: Value = response.json().await.unwrap();
    let id = created["id"].as_str().unwrap();
    let repeated = h.tool("simulationsCreate", command).await;
    assert_eq!(repeated["result"]["structuredContent"]["data"]["id"], id);
    let path = format!("/v1/simulations/{id}");
    tokio::time::timeout(std::time::Duration::from_secs(15), async {
        loop {
            if h.rest(&path).await["status"] == "completed" {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    })
    .await
    .unwrap();
    let job = h.rest(&path).await;
    assert_eq!(job["report"]["seed"], "ffffffffffffffff");
    let before = db.metrics().await.unwrap().cache_hits;
    assert_eq!(
        h.tool("simulationGet", json!({"id":id})).await["result"]["structuredContent"]["data"],
        job
    );
    assert!(db.metrics().await.unwrap().cache_hits > before);
    assert_eq!(
        h.tool("simulationInputsGet", json!({"id":id})).await["result"]["structuredContent"]["data"],
        h.rest(&format!("{path}/inputs")).await
    );
    assert_eq!(
        h.tool("simulationsList", json!({"limit":2})).await["result"]["structuredContent"]["data"],
        h.rest("/v1/simulations?limit=2").await
    );
    let cancelled = h.tool("simulationCancel", json!({"id":id})).await;
    assert_eq!(
        cancelled["result"]["structuredContent"]["data"]["status"],
        "completed"
    );
    let exact = json!({"population":60,"successes":4,"draws":7,"min":1,"max":4});
    let rest: Value = h
        .client
        .post(format!("{}/v1/probabilities/draw", h.base))
        .bearer_auth(&h.token)
        .json(&exact)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        h.tool("probabilityDraw", exact).await["result"]["structuredContent"]["data"],
        rest
    );
    assert!((rest["probability"].as_f64().unwrap() - 0.3994996257442573).abs() < 1e-12);
    assert_eq!(
        h.tool("simulationGet", json!({"id":"missing"})).await["result"]["structuredContent"]["code"],
        "not_found"
    );
    h.server.stop().await.unwrap();
}
