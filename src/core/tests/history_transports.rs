mod mcp_support;
use mcp_support::{ClientHarness, app};
use serde_json::{Value, json};
use std::io::Write as _;
#[tokio::test]
async fn rest_and_mcp_read_the_same_history_and_share_invalidation() {
    let (db, app) = app();
    let ingest = app.ingest.clone();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("Player.log");
    std::fs::write(&path, include_str!("fixtures/arena-game.txt")).unwrap();
    ingest.import_file(&path).await.unwrap();
    let h = ClientHarness::start(app).await;
    let games = h.rest("/v1/games").await;
    assert_eq!(games["items"].as_array().unwrap().len(), 1);
    let game = &games["items"][0];
    let id = game["id"].as_str().unwrap();
    let match_id = game["match_id"].as_str().unwrap();
    let before = db.metrics().await.unwrap().cache_hits;
    assert_eq!(
        h.tool("gamesList", json!({})).await["result"]["structuredContent"]["data"],
        games
    );
    assert_eq!(db.metrics().await.unwrap().cache_hits, before + 1);
    for (tool, args, path) in [
        ("gameGet", json!({"id":id}), format!("/v1/games/{id}")),
        (
            "gameTimeline",
            json!({"id":id,"limit":3}),
            format!("/v1/games/{id}/events?limit=3"),
        ),
        (
            "gameTurns",
            json!({"id":id,"limit":2}),
            format!("/v1/games/{id}/turns?limit=2"),
        ),
        ("matchesList", json!({}), "/v1/matches".into()),
        (
            "matchGet",
            json!({"id":match_id}),
            format!("/v1/matches/{match_id}"),
        ),
        ("statisticsGet", json!({}), "/v1/statistics".into()),
        ("captureStatusGet", json!({}), "/v1/capture".into()),
    ] {
        let rest = h.rest(&path).await;
        let rpc = h.tool(tool, args).await;
        assert_eq!(rpc["result"]["structuredContent"]["data"], rest, "{rpc}");
    }
    let before = h.rest("/v1/statistics").await;
    let new = format!(
        "[UnityCrossThreadLogger] {}\n",
        json!({"greToClientEvent":{"greToClientMessages":[{"type":"GREMessageType_GameStateMessage","msgId":1,"gameStateId":1,"systemSeatIds":[1],"gameStateMessage":{"type":"GameStateType_Full","gameInfo":{"matchID":"new-captured-game","gameNumber":1}}}]}})
    );
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap();
    file.write_all(new.as_bytes()).unwrap();
    ingest.poll_file(&path).await.unwrap();
    let after = h.tool("statisticsGet", json!({})).await;
    assert_eq!(after["result"]["structuredContent"]["data"]["games"], 2);
    assert_eq!(before["games"], 1);
    assert_eq!(
        h.rest("/v1/statistics").await,
        after["result"]["structuredContent"]["data"]
    );
    for (tool, args) in [
        ("gameGet", json!({"id":"missing"})),
        ("gameTimeline", json!({"id":"missing"})),
        ("matchGet", json!({"id":"missing"})),
    ] {
        assert_eq!(
            h.tool(tool, args).await["result"]["structuredContent"]["code"],
            "not_found"
        );
    }
    for path in [
        "/v1/games?limit=0",
        "/v1/games?cursor=bad",
        "/v1/games?mode=invalid",
        "/v1/statistics?complete_only=invalid",
    ] {
        let response = h
            .client
            .get(format!("{}{path}", h.base))
            .bearer_auth(&h.token)
            .send()
            .await
            .unwrap();
        assert!(response.status().is_client_error());
        let body: Value = response.json().await.unwrap();
        assert_eq!(body["code"], "invalid_request");
    }
    let discovery = h.rpc("tools/list", Value::Null).await;
    assert!(
        discovery["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|tool| tool["name"] == "gameTimeline")
    );
    db.close().await.unwrap();
    let closed = h
        .client
        .get(format!("{}/v1/capture", h.base))
        .bearer_auth(&h.token)
        .send()
        .await
        .unwrap();
    assert_eq!(closed.status(), 503);
    let problem: Value = closed.json().await.unwrap();
    assert_eq!(
        h.tool("captureStatusGet", json!({})).await["result"]["structuredContent"],
        problem
    );
    h.server.stop().await.unwrap();
}
