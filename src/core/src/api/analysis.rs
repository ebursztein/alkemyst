use super::{
    AppState,
    extract::ApiPath,
    problem::{ApiError, Problem},
};
use crate::analysis::compiler::Profile;
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
