use super::{
    AppState,
    extract::{ApiJson, ApiPath, ApiQuery},
    problem::{ApiError, Problem},
};
use crate::{
    analysis::DeckSnapshot,
    domain::Origin,
    practice::{NewPractice, PracticeAction, PracticePage, PracticeQuery, PracticeView},
};
use axum::{Json, extract::State};
use serde::Deserialize;
use utoipa::ToSchema;
#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreatePracticeCommand {
    pub idempotency_key: String,
    pub input: NewPractice,
}
#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(crate) struct PracticeActionCommand {
    pub idempotency_key: String,
    #[schema(maximum = 1000)]
    pub expected_revision: u32,
    pub action: PracticeAction,
}
/// Start a manual sandbox using frozen main-deck card metadata and a reproducible
/// shuffle. Mana payments, abilities, combat and state-based actions are manual.
#[utoipa::path(post,path="/v1/practice",operation_id="practiceCreate",request_body=CreatePracticeCommand,
    responses((status=200,body=PracticeView),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn create(
    State(s): State<AppState>,
    ApiJson(c): ApiJson<CreatePracticeCommand>,
) -> Result<Json<PracticeView>, ApiError> {
    Ok(Json(
        s.app
            .practice
            .create(&c.idempotency_key, c.input, Origin::User)
            .await?,
    ))
}
/// List local practice sessions in stable creation order, independently of match history.
#[utoipa::path(get,path="/v1/practice",operation_id="practiceList",params(PracticeQuery),
    responses((status=200,body=PracticePage),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn list(
    State(s): State<AppState>,
    ApiQuery(q): ApiQuery<PracticeQuery>,
) -> Result<Json<PracticePage>, ApiError> {
    Ok(Json(s.app.practice.list(q).await?))
}
/// Read the latest saved step without revealing the ordered library.
#[utoipa::path(get,path="/v1/practice/{id}",operation_id="practiceGet",params(("id"=String,Path)),
    responses((status=200,body=PracticeView),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn get(
    State(s): State<AppState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<PracticeView>, ApiError> {
    Ok(Json(
        s.app
            .practice
            .get(&id)
            .await?
            .ok_or_else(ApiError::missing)?,
    ))
}
/// Read frozen card metadata. This is the original deck roster, not its shuffle order.
#[utoipa::path(get,path="/v1/practice/{id}/inputs",operation_id="practiceInputsGet",params(("id"=String,Path)),
    responses((status=200,body=DeckSnapshot),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn inputs(
    State(s): State<AppState>,
    ApiPath(id): ApiPath<String>,
) -> Result<Json<DeckSnapshot>, ApiError> {
    Ok(Json(
        s.app
            .practice
            .inputs(&id)
            .await?
            .ok_or_else(ApiError::missing)?,
    ))
}
/// Append one immutable practice step at the expected revision. Reusing the key
/// returns the original step; a competing action returns conflict without advancing.
#[utoipa::path(post,path="/v1/practice/{id}/actions",operation_id="practiceAct",params(("id"=String,Path)),request_body=PracticeActionCommand,
    responses((status=200,body=PracticeView),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn act(
    State(s): State<AppState>,
    ApiPath(id): ApiPath<String>,
    ApiJson(c): ApiJson<PracticeActionCommand>,
) -> Result<Json<PracticeView>, ApiError> {
    Ok(Json(
        s.app
            .practice
            .act(
                &c.idempotency_key,
                &id,
                c.expected_revision,
                c.action,
                Origin::User,
            )
            .await?,
    ))
}
