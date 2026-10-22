use alkemyst_core::{
    Application,
    api::{AuthConfig, router},
    db::Engine,
};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

const TOKEN: &str = "test-only-credential-with-at-least-32-characters";

#[tokio::test]
async fn authentication_origin_and_host_are_checked_before_operations() {
    let app = Application::new(Engine::memory(1024 * 1024).unwrap());
    let router = router(app, AuthConfig::new(TOKEN.into(), false).unwrap());
    for (token, origin, host, expected) in [
        (None, None, "127.0.0.1", StatusCode::UNAUTHORIZED),
        (Some("wrong"), None, "127.0.0.1", StatusCode::UNAUTHORIZED),
        (
            Some(TOKEN),
            Some("https://evil.example"),
            "127.0.0.1",
            StatusCode::FORBIDDEN,
        ),
        (Some(TOKEN), None, "evil.example", StatusCode::FORBIDDEN),
        (
            Some(TOKEN),
            Some("http://localhost:1420"),
            "127.0.0.1",
            StatusCode::FORBIDDEN,
        ),
        (Some(TOKEN), None, "127.0.0.1", StatusCode::OK),
        (
            Some(TOKEN),
            Some("tauri://localhost"),
            "127.0.0.1",
            StatusCode::OK,
        ),
    ] {
        let mut request = Request::builder().uri("/v1/decks").header("host", host);
        if let Some(token) = token {
            request = request.header("authorization", format!("Bearer {token}"));
        }
        if let Some(origin) = origin {
            request = request.header("origin", origin);
        }
        let response = router
            .clone()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap())
                .unwrap();
        if expected != StatusCode::OK {
            assert!(body["code"].is_string());
        }
    }
}

#[tokio::test]
async fn trusted_preflight_needs_no_token_but_never_exposes_data() {
    let app = Application::new(Engine::memory(4096).unwrap());
    let router = router(app, AuthConfig::new(TOKEN.into(), true).unwrap());
    let request = Request::builder()
        .method("OPTIONS")
        .uri("/v1/decks")
        .header("host", "localhost:47831")
        .header("origin", "http://localhost:1420")
        .header("access-control-request-method", "GET")
        .body(Body::empty())
        .unwrap();
    let response = router.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        response.headers()["access-control-allow-origin"],
        "http://localhost:1420"
    );
}
