use super::{
    AppState,
    extract::{ApiJson, ApiPath},
    models::{ArenaIds, References},
    problem::{ApiError, Problem},
};
use crate::catalog::{ArenaResolution, CardPage, CatalogStatus, ReferenceResolution, Search};
use axum::{Json, extract::State};
use serde_json::Value;

/// Search the offline catalog by text and structured filters.
#[utoipa::path(post,path="/v1/cards/search",operation_id="cardsSearch",request_body=Search,
    responses((status=200,body=CardPage),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn search(
    State(state): State<AppState>,
    ApiJson(query): ApiJson<Search>,
) -> Result<Json<CardPage>, ApiError> {
    Ok(Json(state.app.catalog.search(query).await?))
}

/// Read complete original Scryfall metadata, including card faces.
#[utoipa::path(get,path="/v1/cards/{oracle_id}",operation_id="cardsGet",params(("oracle_id"=String,Path)),
    responses((status=200,body=std::collections::BTreeMap<String,Value>),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn get(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<Value>, ApiError> {
    Ok(Json(
        state
            .app
            .catalog
            .card(&id)
            .await?
            .ok_or_else(ApiError::missing)?,
    ))
}

/// Resolve Arena IDs in one batch, preserving unknowns and mapping provenance.
#[utoipa::path(post,path="/v1/cards/resolve-arena",operation_id="cardsResolveArena",request_body=ArenaIds,
    responses((status=200,body=Vec<ArenaResolution>),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn arena(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<ArenaIds>,
) -> Result<Json<Vec<ArenaResolution>>, ApiError> {
    Ok(Json(state.app.catalog.resolve_arena(&input.ids).await?))
}

/// Resolve mixed Oracle, Scryfall-printing and Arena references in one batch.
#[utoipa::path(post,path="/v1/cards/resolve-references",operation_id="cardsResolveReferences",request_body=References,
    responses((status=200,body=Vec<ReferenceResolution>),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn references(
    State(state): State<AppState>,
    ApiJson(input): ApiJson<References>,
) -> Result<Json<Vec<ReferenceResolution>>, ApiError> {
    Ok(Json(
        state
            .app
            .catalog
            .resolve_references(&input.references)
            .await?,
    ))
}

/// Read catalog version and offline availability.
#[utoipa::path(get,path="/v1/catalog",operation_id="catalogStatusGet",
    responses((status=200,body=CatalogStatus),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn status(State(state): State<AppState>) -> Result<Json<CatalogStatus>, ApiError> {
    Ok(Json(state.app.catalog.status().await?))
}

/// Refresh public Scryfall metadata while preserving the prior offline catalog on failure.
#[utoipa::path(post,path="/v1/catalog/refresh",operation_id="catalogRefresh",
    responses((status=200,body=CatalogStatus),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn refresh(
    State(state): State<AppState>,
) -> Result<Json<CatalogStatus>, ApiError> {
    Ok(Json(state.app.refresh_catalog().await?))
}
