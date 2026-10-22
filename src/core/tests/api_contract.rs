#[path = "api_support/setup.rs"]
mod api_setup;
mod api_support;
use alkemyst_core::{ApiDoc, api::AuthConfig};
use api_setup::setup;
use api_support::{TOKEN, call};
use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;
use utoipa::OpenApi;

#[tokio::test]
async fn contract_matches_runtime_routes_and_rejects_malformed_payloads() {
    let (_, _, router) = setup();
    let schema = serde_json::to_value(ApiDoc::openapi()).unwrap();
    assert_eq!(schema["openapi"], "3.1.0");
    let discovery = call(&router, "GET", "/v1/discovery", None).await.1;
    assert!(
        discovery["operations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|id| id == "systemDiscovery")
    );
    assert_eq!(discovery["local_only"], true);
    assert_eq!(call(&router, "GET", "/openapi.json", None).await.1, schema);
    assert_eq!(schema["security"][0]["localBearer"], serde_json::json!([]));
    assert_eq!(
        schema["paths"]["/v1/decks"]["post"]["operationId"],
        "decksCreate"
    );
    for (content_type, body, expected) in [
        (
            "application/json",
            "{broken".to_string(),
            StatusCode::BAD_REQUEST,
        ),
        (
            "application/json",
            "{}".to_string(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ),
        (
            "text/plain",
            "{}".to_string(),
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
        ),
        (
            "application/json",
            "x".repeat(1024 * 1024 + 1),
            StatusCode::PAYLOAD_TOO_LARGE,
        ),
    ] {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/v1/decks")
                    .header("host", "localhost")
                    .header("authorization", format!("Bearer {TOKEN}"))
                    .header("content-type", content_type)
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
        let error: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap())
                .unwrap();
        assert_eq!(error["code"], "invalid_request");
    }
    assert!(AuthConfig::new("short".into(), false).is_err());
    assert!(AuthConfig::new("a\n".repeat(40), false).is_err());
}
