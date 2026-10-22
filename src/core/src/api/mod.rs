mod analysis;
mod auth;
mod cards;
mod decks;
pub(crate) mod discovery;
mod extract;
mod history;
mod models;
mod notes;
mod openapi;
mod probabilities;
mod server;
mod simulations;
pub use openapi::ApiDoc;
pub use server::RunningServer;
pub(crate) use simulations::CreateSimulationCommand;
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
    router_with_mcp(app, auth, Default::default())
}

pub(super) fn router_with_mcp(
    app: Application,
    auth: AuthConfig,
    config: rmcp::transport::streamable_http_server::StreamableHttpServerConfig,
) -> Router {
    let mcp = crate::mcp::service(app.clone(), config);
    let state = AppState { app, auth };
    let protected = Router::new()
        .nest_service("/mcp", mcp)
        .route(
            "/v1/simulations",
            get(simulations::list).post(simulations::create),
        )
        .route("/v1/simulations/{id}", get(simulations::get))
        .route("/v1/simulations/{id}/inputs", get(simulations::inputs))
        .route("/v1/simulations/{id}/cancel", post(simulations::cancel))
        .route("/v1/probabilities/draw", post(probabilities::draw))
        .route("/v1/probabilities/joint", post(probabilities::joint))
        .route("/v1/games", get(history::games))
        .route("/v1/games/{id}", get(history::game))
        .route("/v1/games/{id}/events", get(history::timeline))
        .route("/v1/games/{id}/turns", get(history::turns))
        .route("/v1/matches", get(history::matches))
        .route("/v1/matches/{id}", get(history::match_detail))
        .route("/v1/statistics", get(history::statistics))
        .route("/v1/capture", get(history::capture))
        .route("/v1/discovery", get(discovery::get))
        .route("/v1/decks", get(decks::list).post(decks::create))
        .route("/v1/deck-versions", post(decks::create_version))
        .route("/v1/deck-versions/{id}", get(decks::version))
        .route("/v1/deck-versions/{id}/analysis", get(analysis::profile))
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
