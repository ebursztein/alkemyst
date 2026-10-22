use super::{
    AppState,
    extract::ApiPath,
    problem::{ApiError, Problem},
};
use crate::analysis::{DeckComparison, compiler::Profile};
use axum::{Json, extract::State};

/// Analyze an immutable deck version using the current local catalog. Printed
/// curves remain available when missing or unsupported rules block simulation.
#[utoipa::path(get,path="/v1/deck-versions/{id}/analysis",operation_id="analysisProfileGet",params(("id"=String,Path)),responses((status=200,body=Profile),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn profile(
    State(state): State<AppState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<Profile>, ApiError> {
    Ok(Json(
        state
            .app
            .analysis
            .profile(&id)
            .await?
            .ok_or_else(ApiError::missing)?,
    ))
}

/// Compare immutable versions using one current catalog snapshot. Deltas normalize
/// printings to Oracle identities and retain unresolved references and zones.
/// The printed nonland curve is independent of observed outcomes and simulation.
#[utoipa::path(get,path="/v1/deck-versions/{id}/compare/{candidate_id}",operation_id="deckVersionsCompare",
    params(("id"=String,Path),("candidate_id"=String,Path)),
    responses((status=200,body=DeckComparison),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn compare(
    State(state): State<AppState>,
    ApiPath((id, candidate_id)): ApiPath<(String, String)>,
) -> Result<Json<DeckComparison>, ApiError> {
    Ok(Json(
        state
            .app
            .analysis
            .compare(&id, &candidate_id)
            .await?
            .ok_or_else(ApiError::missing)?,
    ))
}
