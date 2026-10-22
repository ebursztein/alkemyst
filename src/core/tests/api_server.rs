use alkemyst_core::{
    Application,
    api::{AuthConfig, RunningServer},
    db::Engine,
};

#[tokio::test]
async fn listener_only_binds_loopback_and_shuts_down_cleanly() {
    let app = Application::new(Engine::memory(4096).unwrap());
    let auth = AuthConfig::generate();
    let token = auth.credential().to_string();
    assert!(
        RunningServer::start(app.clone(), auth.clone(), "0.0.0.0:0".parse().unwrap())
            .await
            .is_err()
    );
    let server = RunningServer::start(app, auth, "127.0.0.1:0".parse().unwrap())
        .await
        .unwrap();
    let base = format!("http://{}", server.address());
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{base}/v1/decks"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());
    assert_eq!(
        client
            .get(format!("{base}/v1/health"))
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap()["name"],
        "Alkemyst"
    );
    server.stop().await.unwrap();
    assert!(
        client
            .get(format!("{base}/v1/health"))
            .send()
            .await
            .is_err()
    );
}
