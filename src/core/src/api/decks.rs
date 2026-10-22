use super::{
    AppState,
    extract::{ApiJson, ApiPath, ApiQuery},
    models::*,
    problem::{ApiError, Problem},
};
use crate::domain::{DeckPage, DeckVersion, Origin};
use axum::{Json, extract::State};

/// List local decks with stable cursor pagination.
#[utoipa::path(get,path="/v1/decks",operation_id="decksList",params(PageQuery),
    responses((status=200,body=DeckPage),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn list(
    State(state): State<AppState>,
    ApiQuery(page): ApiQuery<PageQuery>,
) -> Result<Json<DeckPage>, ApiError> {
    Ok(Json(state.app.decks.list(page.limit, page.cursor).await?))
}

/// Create a deck and its first immutable version.
#[utoipa::path(post,path="/v1/decks",operation_id="decksCreate",request_body=CreateDeckCommand,
    responses((status=200,body=DeckVersion),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn create(
    State(state): State<AppState>,
    ApiJson(command): ApiJson<CreateDeckCommand>,
) -> Result<Json<DeckVersion>, ApiError> {
    Ok(Json(
        state
            .app
            .decks
            .create(&command.idempotency_key, command.input, Origin::User)
            .await?,
    ))
}

/// Read an immutable deck version and its original card references.
#[utoipa::path(get,path="/v1/deck-versions/{id}",operation_id="deckVersionGet",params(("id"=String,Path)),
    responses((status=200,body=DeckVersion),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn version(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<DeckVersion>, ApiError> {
    Ok(Json(
        state
            .app
            .decks
            .version(&id)
            .await?
            .ok_or_else(ApiError::missing)?,
    ))
}

/// Append a deck version; earlier versions stay immutable.
#[utoipa::path(post,path="/v1/deck-versions",operation_id="deckVersionCreate",request_body=CreateVersionCommand,
    responses((status=200,body=DeckVersion),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn create_version(
    State(state): State<AppState>,
    ApiJson(command): ApiJson<CreateVersionCommand>,
) -> Result<Json<DeckVersion>, ApiError> {
    Ok(Json(
        state
            .app
            .decks
            .create_version(&command.idempotency_key, command.input, Origin::User)
            .await?,
    ))
}

/// Import Arena text, reporting every unresolved card instead of dropping lines.
#[utoipa::path(post,path="/v1/decks/import-arena",operation_id="deckArenaImport",request_body=ImportDeckCommand,
    responses((status=200,body=DeckVersion),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn import(
    State(state): State<AppState>,
    ApiJson(command): ApiJson<ImportDeckCommand>,
) -> Result<Json<DeckVersion>, ApiError> {
    Ok(Json(
        state
            .app
            .decks
            .import_arena(&command.idempotency_key, command.input, Origin::Import)
            .await?,
    ))
}

/// Export a saved version for pasting into Arena.
#[utoipa::path(get,path="/v1/deck-versions/{id}/arena",operation_id="deckArenaExport",params(("id"=String,Path)),
    responses((status=200,body=ArenaExport),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn export(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<ArenaExport>, ApiError> {
    Ok(Json(ArenaExport {
        text: state.app.decks.export_arena(&id).await?,
    }))
}

/// List sealed immutable versions in descending creation order. Metadata and
/// card counts are aggregated by SQL; cursors remain opaque to callers.
#[utoipa::path(get,path="/v1/decks/{id}/versions",operation_id="deckVersionsList",
    params(("id"=String,Path),PageQuery),
    responses((status=200,body=crate::domain::VersionPage),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn versions(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<String>,
    ApiQuery(page): ApiQuery<PageQuery>,
) -> Result<Json<crate::domain::VersionPage>, ApiError> {
    Ok(Json(
        state
            .app
            .decks
            .versions(&id, page.limit, page.cursor)
            .await?
            .ok_or_else(ApiError::missing)?,
    ))
}
