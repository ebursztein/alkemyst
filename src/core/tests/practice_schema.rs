use alkemyst_core::{ApiDoc, mcp::schema::tools};
use serde_json::{Value, json};
use utoipa::OpenApi;
#[test]
fn mcp_combines_path_parameters_with_typed_request_bodies_without_mutating_openapi() {
    let tools = tools();
    let tool = tools.iter().find(|t| t.name == "practiceAct").unwrap();
    let schema = Value::Object(tool.input_schema.as_ref().clone());
    let validator = jsonschema::validator_for(&schema).unwrap();
    let action = json!({"id":"session","idempotency_key":"step","expected_revision":1,"action":{"kind":"next_turn"}});
    assert!(validator.is_valid(&action), "{schema}");
    let mut missing = action.clone();
    missing.as_object_mut().unwrap().remove("id");
    assert!(!validator.is_valid(&missing));
    let mut extra = action;
    extra["unexpected"] = json!(true);
    assert!(!validator.is_valid(&extra));
    let document = serde_json::to_value(ApiDoc::openapi()).unwrap();
    assert!(
        document["components"]["schemas"]["PracticeActionCommand"]["properties"]
            .get("id")
            .is_none()
    );
    assert_eq!(
        tool.annotations.as_ref().unwrap().read_only_hint,
        Some(false)
    );
}
