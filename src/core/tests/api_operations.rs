#[path = "api_support/setup.rs"]
mod api_setup;
mod api_support;
use api_setup::setup;
use api_support::call;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn rest_decks_versions_notes_and_errors_use_shared_operations() {
    let (db, app, router) = setup();
    let command = json!({"idempotency_key":"api-create","input":{"name":"Boros","format":"standard","cards":[{"reference":"arena:42","quantity":4,"zone":"main"}]}});
    let (status, first) = call(&router, "POST", "/v1/decks", Some(command.clone())).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        call(&router, "POST", "/v1/decks", Some(command)).await.1,
        first
    );
    let id = first["id"].as_str().unwrap();
    let path = format!("/v1/deck-versions/{id}");
    assert_eq!(call(&router, "GET", &path, None).await.1, first);
    let before = db.metrics().await.unwrap().cache_hits;
    assert_eq!(call(&router, "GET", &path, None).await.1, first);
    assert_eq!(db.metrics().await.unwrap().cache_hits, before + 1);
    let version = json!({"idempotency_key":"api-version","input":{"deck_id":first["deck_id"],"parent_version":id,"name":"Boros v2","format":"standard","cards":[]}});
    let (status, next) = call(&router, "POST", "/v1/deck-versions", Some(version)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(next["ordinal"], 2);
    assert_eq!(app.decks.version(id).await.unwrap().unwrap().name, "Boros");
    let note = json!({"idempotency_key":"api-note","input":{"deck_id":first["deck_id"],"version_id":id,"body":"Test another tutor."}});
    assert_eq!(
        call(&router, "POST", "/v1/notes", Some(note)).await.0,
        StatusCode::OK
    );
    let note_path = format!("/v1/notes?deck_id={}", first["deck_id"].as_str().unwrap());
    assert_eq!(
        call(&router, "GET", &note_path, None).await.1["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let duplicate = json!({"idempotency_key":"api-create","input":{"name":"Changed","format":"standard","cards":[]}});
    let conflict = call(&router, "POST", "/v1/decks", Some(duplicate)).await;
    assert_eq!(conflict.0, StatusCode::CONFLICT);
    assert_eq!(conflict.1["code"], "conflict");
    for path in ["/v1/deck-versions/missing", "/v1/cards/missing", "/missing"] {
        assert_eq!(
            call(&router, "GET", path, None).await.0,
            StatusCode::NOT_FOUND
        );
    }
    assert_eq!(
        call(&router, "DELETE", "/v1/decks", None).await.0,
        StatusCode::METHOD_NOT_ALLOWED
    );
    assert_eq!(
        call(&router, "GET", "/v1/decks?limit=0", None).await.0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        call(&router, "GET", "/v1/decks?limit=no", None).await.0,
        StatusCode::BAD_REQUEST
    );
}
