#[path = "api_support/setup.rs"]
mod api_setup;
mod api_support;
use api_setup::setup;
mod support;
use api_support::call;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn rest_card_search_resolution_and_import_export_are_offline() {
    let (_, app, router) = setup();
    app.catalog
        .import_jsonl(std::io::Cursor::new(support::cards()), support::source())
        .await
        .unwrap();
    let (status, page) = call(
        &router,
        "POST",
        "/v1/cards/search",
        Some(json!({"text":"forge"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(page["items"][0]["name"], "Forgeblade");
    assert_eq!(
        call(&router, "GET", "/v1/catalog", None).await.1["cards"],
        3
    );
    assert_eq!(
        call(&router, "GET", "/v1/cards/oracle-modal", None).await.1["card_faces"][1]["name"],
        "Meadow"
    );
    let ids = call(
        &router,
        "POST",
        "/v1/cards/resolve-arena",
        Some(json!({"ids":[42,999]})),
    )
    .await;
    assert_eq!(ids.1[0]["card"]["name"], "Forgeblade");
    assert!(ids.1[1]["card"].is_null());
    let refs = call(
        &router,
        "POST",
        "/v1/cards/resolve-references",
        Some(json!({"references":["oracle:oracle-tutor"]})),
    )
    .await;
    assert_eq!(refs.1[0]["card"]["name"], "Find the Blade");
    let imported=call(&router,"POST","/v1/decks/import-arena",Some(json!({"idempotency_key":"arena-text","input":{"name":"Imported","format":"standard","text":"Deck\n4 Forgeblade"}}))).await;
    assert_eq!(imported.0, StatusCode::OK);
    let path = format!(
        "/v1/deck-versions/{}/arena",
        imported.1["id"].as_str().unwrap()
    );
    assert!(
        call(&router, "GET", &path, None).await.1["text"]
            .as_str()
            .unwrap()
            .contains("4 Forgeblade")
    );
    let missing=call(&router,"POST","/v1/decks/import-arena",Some(json!({"idempotency_key":"unknown","input":{"name":"Unknown","format":"standard","text":"Deck\n4 Missing"}}))).await;
    assert_eq!(missing.0, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(missing.1["unresolved"], json!(["Missing"]));
}
