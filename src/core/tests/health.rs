use alkemyst_core::{ApiDoc, build_info};
use utoipa::OpenApi;

#[test]
fn build_identity_matches_package_and_contract() {
    let info = build_info();
    assert_eq!(info.name, "Alkemyst");
    assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
    let value = serde_json::to_value(info).unwrap();
    assert_eq!(value["name"], "Alkemyst");
    let api = serde_json::to_value(ApiDoc::openapi()).unwrap();
    assert_eq!(api["openapi"], "3.1.0");
    assert_eq!(
        api["paths"]["/v1/health"]["get"]["operationId"],
        "healthGet"
    );
    assert!(api["components"]["schemas"]["BuildInfo"].is_object());
}
