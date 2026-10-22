mod auth;
mod cards;
mod decks;
mod discovery;
mod extract;
mod models;
mod notes;
mod openapi;
mod server;
pub use openapi::ApiDoc;
pub use server::RunningServer;
pub mod problem;

use crate::Application;
pub use auth::AuthConfig;
use axum::{
    Json, Router,
    extract::DefaultBodyLimit,
    http::StatusCode,
    middleware,
    routing::{get, post},
};
pub use models::*;
use problem::{ApiError, ErrorCode};
use utoipa::OpenApi;

#[derive(Clone)]
struct AppState {
    app: Application,
    auth: AuthConfig,
}

pub fn router(app: Application, auth: AuthConfig) -> Router {
    let state = AppState { app, auth };
    let protected = Router::new()
        .route("/v1/discovery", get(discovery::get))
        .route("/v1/decks", get(decks::list).post(decks::create))
        .route("/v1/deck-versions", post(decks::create_version))
        .route("/v1/deck-versions/{id}", get(decks::version))
        .route("/v1/deck-versions/{id}/arena", get(decks::export))
        .route("/v1/decks/import-arena", post(decks::import))
        .route("/v1/notes", get(notes::list).post(notes::create))
        .route("/v1/cards/search", post(cards::search))
        .route("/v1/cards/{oracle_id}", get(cards::get))
        .route("/v1/cards/resolve-arena", post(cards::arena))
        .route("/v1/cards/resolve-references", post(cards::references))
        .route("/v1/catalog", get(cards::status))
        .route("/v1/catalog/refresh", post(cards::refresh))
        .route(
            "/openapi.json",
            get(|| async { Json(crate::ApiDoc::openapi()) }),
        )
        .fallback(|| async { ApiError::missing() })
        .method_not_allowed_fallback(|| async {
            ApiError::new(
                StatusCode::METHOD_NOT_ALLOWED,
                ErrorCode::InvalidRequest,
                "Method not allowed",
            )
        })
        .layer(DefaultBodyLimit::max(1024 * 1024))
        .layer(middleware::from_fn_with_state(state.clone(), auth::guard))
        .with_state(state);
    Router::new()
        .route("/v1/health", get(|| async { Json(crate::build_info()) }))
        .merge(protected)
}
