use super::{
    AppState,
    extract::{ApiPath, ApiQuery},
    problem::{ApiError, Problem},
};
use crate::history::{
    GamePage, GameQuery, GameSummary, MatchPage, MatchSummary, Statistics, StatisticsQuery,
    TimelinePage, TimelineQuery, TurnPage, TurnQuery,
};
use axum::{Json, extract::State};

/// List captured games with immutable deck attribution, samples and missing-data flags.
#[utoipa::path(get,path="/v1/games",operation_id="gamesList",params(GameQuery),responses((status=200,body=GamePage),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn games(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<GameQuery>,
) -> Result<Json<GamePage>, ApiError> {
    Ok(Json(state.app.history.games(query).await?))
}
/// Read one game; known outcomes do not imply a complete timeline.
#[utoipa::path(get,path="/v1/games/{id}",operation_id="gameGet",params(("id"=String,Path)),responses((status=200,body=GameSummary),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn game(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<GameSummary>, ApiError> {
    Ok(Json(
        state
            .app
            .history
            .game(&id)
            .await?
            .ok_or_else(ApiError::missing)?,
    ))
}
/// Read a stable, bounded event page. Keep returned cursors opaque; later captures do not alter this pagination snapshot.
#[utoipa::path(get,path="/v1/games/{id}/events",operation_id="gameTimeline",params(("id"=String,Path),TimelineQuery),responses((status=200,body=TimelinePage),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn timeline(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<String>,
    ApiQuery(query): ApiQuery<TimelineQuery>,
) -> Result<Json<TimelinePage>, ApiError> {
    Ok(Json(
        state
            .app
            .history
            .timeline(&id, query)
            .await?
            .ok_or_else(ApiError::missing)?,
    ))
}
/// List matches separately from their games. Deck/version filters select matches containing at least one matching game.
#[utoipa::path(get,path="/v1/matches",operation_id="matchesList",params(GameQuery),responses((status=200,body=MatchPage),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn matches(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<GameQuery>,
) -> Result<Json<MatchPage>, ApiError> {
    Ok(Json(state.app.history.matches(query).await?))
}
/// Read a match and its games. Match outcomes require an explicit match-scoped result.
#[utoipa::path(get,path="/v1/matches/{id}",operation_id="matchGet",params(("id"=String,Path)),responses((status=200,body=MatchSummary),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn match_detail(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<MatchSummary>, ApiError> {
    Ok(Json(
        state
            .app
            .history
            .match_summary(&id)
            .await?
            .ok_or_else(ApiError::missing)?,
    ))
}
/// Observe game-level performance with explicit denominators. Unknowns are excluded from decisive win rate; recorded land counters cover only sampled local turns.
#[utoipa::path(get,path="/v1/statistics",operation_id="statisticsGet",params(StatisticsQuery),responses((status=200,body=Statistics),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn statistics(
    State(state): State<AppState>,
    ApiQuery(query): ApiQuery<StatisticsQuery>,
) -> Result<Json<Statistics>, ApiError> {
    Ok(Json(state.app.history.statistics(query).await?))
}
/// Read durable capture counts. Unknown results include unfinished and ambiguous games.
#[utoipa::path(get,path="/v1/capture",operation_id="captureStatusGet",responses((status=200,body=crate::ingest::CaptureStatus),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn capture(
    State(state): State<AppState>,
) -> Result<Json<crate::ingest::CaptureStatus>, ApiError> {
    Ok(Json(state.app.ingest.status().await?))
}

/// Group observed events into turns without inferring absent actions. Pagination freezes capture evidence; unassigned events are counted separately.
#[utoipa::path(get,path="/v1/games/{id}/turns",operation_id="gameTurns",params(("id"=String,Path),TurnQuery),responses((status=200,body=TurnPage),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn turns(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<String>,
    ApiQuery(query): ApiQuery<TurnQuery>,
) -> Result<Json<TurnPage>, ApiError> {
    Ok(Json(
        state
            .app
            .history
            .turns(&id, query)
            .await?
            .ok_or_else(ApiError::missing)?,
    ))
}
