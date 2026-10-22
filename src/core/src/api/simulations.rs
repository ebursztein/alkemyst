use super::{
    AppState,
    extract::{ApiJson, ApiPath, ApiQuery},
    problem::{ApiError, Problem},
};
use crate::{
    analysis::jobs::{JobPage, JobQuery, SimulationInputs, SimulationJob, SimulationParameters},
    domain::Origin,
};
use axum::{Json, extract::State, http::StatusCode};
use serde::Deserialize;
use utoipa::ToSchema;
#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateSimulationCommand {
    pub idempotency_key: String,
    pub input: SimulationParameters,
}
/// Queue a seeded local simulation. Inputs and model versions are frozen before
/// execution; retries with the same key return the original job.
#[utoipa::path(post,path="/v1/simulations",operation_id="simulationsCreate",request_body=CreateSimulationCommand,responses((status=202,body=SimulationJob),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn create(
    State(state): State<AppState>,
    ApiJson(c): ApiJson<CreateSimulationCommand>,
) -> Result<(StatusCode, Json<SimulationJob>), ApiError> {
    Ok((
        StatusCode::ACCEPTED,
        Json(
            state
                .app
                .analysis
                .submit_simulation(&c.idempotency_key, c.input, Origin::User)
                .await?,
        ),
    ))
}
/// List persisted simulation jobs in stable creation order using opaque cursors.
#[utoipa::path(get,path="/v1/simulations",operation_id="simulationsList",params(JobQuery),responses((status=200,body=JobPage),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn list(
    State(s): State<AppState>,
    ApiQuery(q): ApiQuery<JobQuery>,
) -> Result<Json<JobPage>, ApiError> {
    Ok(Json(s.app.analysis.simulations(q).await?))
}
/// Read durable progress and results. A missing report means no completed trials.
#[utoipa::path(get,path="/v1/simulations/{id}",operation_id="simulationGet",params(("id"=String,Path)),responses((status=200,body=SimulationJob),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn get(
    State(s): State<AppState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<SimulationJob>, ApiError> {
    Ok(Json(
        s.app
            .analysis
            .simulation(&id)
            .await?
            .ok_or_else(ApiError::missing)?,
    ))
}
/// Read the frozen card metadata and parameters used by a job, not today's catalog.
#[utoipa::path(get,path="/v1/simulations/{id}/inputs",operation_id="simulationInputsGet",params(("id"=String,Path)),responses((status=200,body=SimulationInputs),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn inputs(
    State(s): State<AppState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<SimulationInputs>, ApiError> {
    Ok(Json(
        s.app
            .analysis
            .simulation_inputs(&id)
            .await?
            .ok_or_else(ApiError::missing)?,
    ))
}
/// Request cancellation. Bounded batches preserve completed-trial denominators;
/// cancelling an already terminal job is an idempotent read of its final state.
#[utoipa::path(post,path="/v1/simulations/{id}/cancel",operation_id="simulationCancel",params(("id"=String,Path)),responses((status=200,body=SimulationJob),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn cancel(
    State(s): State<AppState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<SimulationJob>, ApiError> {
    Ok(Json(
        s.app.analysis.cancel_simulation(&id, Origin::User).await?,
    ))
}
