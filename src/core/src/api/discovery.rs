use super::problem::Problem;
use axum::Json;
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

#[derive(Serialize, ToSchema)]
pub struct Discovery {
    pub name: &'static str,
    pub version: &'static str,
    pub local_only: bool,
    pub capabilities: Vec<&'static str>,
    pub operations: Vec<String>,
}

/// Discover implemented local capabilities and stable operation identifiers.
#[utoipa::path(get,path="/v1/discovery",operation_id="systemDiscovery",
    responses((status=200,body=Discovery),(status="4XX",body=Problem),(status="5XX",body=Problem)))]
pub(super) async fn get() -> Json<Discovery> {
    Json(discover())
}

pub(crate) fn discover() -> Discovery {
    let schema = serde_json::to_value(super::ApiDoc::openapi()).expect("OpenAPI serialization");
    let mut operations = schema["paths"]
        .as_object()
        .into_iter()
        .flat_map(|paths| paths.values())
        .filter_map(|path| path.as_object())
        .flat_map(|path| path.values())
        .filter_map(|operation| operation["operationId"].as_str().map(str::to_owned))
        .collect::<Vec<_>>();
    operations.sort();
    Discovery {
        name: "Alkemyst",
        version: env!("CARGO_PKG_VERSION"),
        local_only: true,
        capabilities: vec![
            "cards",
            "analysis",
            "simulations",
            "exact_probabilities",
            "decks",
            "versions",
            "notes",
            "games",
            "matches",
            "timelines",
            "statistics",
            "capture",
        ],
        operations,
    }
}
