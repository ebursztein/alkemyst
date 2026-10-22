use super::{
    extract::ApiJson,
    problem::{ApiError, Problem},
};
use crate::analysis::probabilities::{self, DrawProbability, ExactProbability, JointProbability};
use axum::Json;
/// Exact finite-deck draw probability without replacement. This is not a
/// battlefield, tutor, mulligan or Arena smoothing model.
#[utoipa::path(post,path="/v1/probabilities/draw",operation_id="probabilityDraw",request_body=DrawProbability,responses((status=200,body=ExactProbability),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn draw(
    ApiJson(input): ApiJson<DrawProbability>,
) -> Result<Json<ExactProbability>, ApiError> {
    Ok(Json(probabilities::draw(input)?))
}
/// Exact joint draw probability across mutually exclusive categories covering
/// the whole deck. Split overlapping groups into intersection categories first.
#[utoipa::path(post,path="/v1/probabilities/joint",operation_id="probabilityJoint",request_body=JointProbability,responses((status=200,body=ExactProbability),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn joint(
    ApiJson(input): ApiJson<JointProbability>,
) -> Result<Json<ExactProbability>, ApiError> {
    Ok(Json(probabilities::joint(input)?))
}
