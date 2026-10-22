use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct BuildInfo {
    pub name: &'static str,
    pub version: &'static str,
}

/// Read the local service build identity.
#[utoipa::path(
    get,
    path = "/v1/health",
    operation_id = "healthGet",
    responses((status = 200, description = "Build identity", body = BuildInfo)),
    security(())
)]
pub fn build_info() -> BuildInfo {
    BuildInfo {
        name: "Alkemyst",
        version: env!("CARGO_PKG_VERSION"),
    }
}
