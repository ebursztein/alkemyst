use alkemyst_core::{
    Application,
    api::{AuthConfig, RunningServer},
    db::Engine,
};
use serde_json::{Value, json};

#[tokio::test]
async fn mcp_is_authenticated_and_lists_only_contract_backed_tools() {
    let auth = AuthConfig::generate();
    let token = auth.credential().to_owned();
    let server = RunningServer::start(
        Application::new(Engine::memory(4096).unwrap()),
        auth,
        "127.0.0.1:0".parse().unwrap(),
    )
    .await
    .unwrap();
    let url = format!("http://{}/mcp", server.address());
    let client = reqwest::Client::new();
    let request = json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"alkemyst-test","version":"1"}}});
    let unauthorized = client.post(&url).json(&request).send().await.unwrap();
    assert_eq!(unauthorized.status(), 401);
    for (header, value) in [
        ("origin", "https://untrusted.example"),
        ("host", "untrusted.example"),
    ] {
        let denied = client
            .post(&url)
            .bearer_auth(&token)
            .header(header, value)
            .json(&request)
            .send()
            .await
            .unwrap();
        assert_eq!(denied.status(), 403);
    }
    let preflight = client
        .request(reqwest::Method::OPTIONS, &url)
        .header("origin", "tauri://localhost")
        .header("access-control-request-method", "POST")
        .header(
            "access-control-request-headers",
            "authorization,mcp-protocol-version",
        )
        .send()
        .await
        .unwrap();
    assert_eq!(preflight.status(), 204);
    assert!(
        preflight.headers()["access-control-allow-headers"]
            .to_str()
            .unwrap()
            .contains("MCP-Protocol-Version")
    );
    let initialized: Value = client
        .post(&url)
        .bearer_auth(&token)
        .header("accept", "application/json, text/event-stream")
        .json(&request)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(initialized["result"]["serverInfo"]["name"], "Alkemyst");
    let listed: Value = client
        .post(&url)
        .bearer_auth(&token)
        .header("accept", "application/json, text/event-stream")
        .header("mcp-protocol-version", "2025-03-26")
        .json(&json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let tools = listed["result"]["tools"].as_array().unwrap();
    assert!(tools.iter().any(|tool| tool["name"] == "decksCreate"));
    assert!(!tools.iter().any(|tool| tool["name"] == "sql"));
    for tool in tools {
        assert_eq!(tool["inputSchema"]["type"], "object");
    }
    let oversized = client
        .post(&url)
        .bearer_auth(&token)
        .header("accept", "application/json, text/event-stream")
        .header("content-type", "application/json")
        .body(" ".repeat(1024 * 1024 + 1))
        .send()
        .await
        .unwrap();
    assert_eq!(oversized.status(), 413);
    server.stop().await.unwrap();
}
