use alkemyst_core::{
    Application,
    api::{AuthConfig, RunningServer},
    db::Engine,
};
use reqwest::Client;
use serde_json::{Value, json};

pub struct ClientHarness {
    pub server: RunningServer,
    pub client: Client,
    pub token: String,
    pub base: String,
}
impl ClientHarness {
    pub async fn start(app: Application) -> Self {
        let auth = AuthConfig::generate();
        let token = auth.credential().to_owned();
        let server = RunningServer::start(app, auth, "127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();
        let base = format!("http://{}", server.address());
        Self {
            server,
            client: Client::new(),
            token,
            base,
        }
    }
    pub async fn rpc(&self, method: &str, params: Value) -> Value {
        self.client
            .post(format!("{}/mcp", self.base))
            .bearer_auth(&self.token)
            .header("accept", "application/json, text/event-stream")
            .header("mcp-protocol-version", "2025-03-26")
            .json(&json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
    pub async fn tool(&self, name: &str, arguments: Value) -> Value {
        self.rpc("tools/call", json!({"name":name,"arguments":arguments}))
            .await
    }
    pub async fn rest(&self, path: &str) -> Value {
        self.client
            .get(format!("{}{path}", self.base))
            .bearer_auth(&self.token)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap()
    }
}
pub fn app() -> (Engine, Application) {
    let db = Engine::memory(4 * 1024 * 1024).unwrap();
    (db.clone(), Application::new(db))
}
