use alkemyst_core::{
    ApiDoc,
    mcp::schema::{ALLOWLIST, from_openapi, tools},
};
use serde_json::{Value, json};
use utoipa::OpenApi;

#[test]
fn tools_derive_referenced_types_limits_and_descriptions_from_openapi() {
    let mut document = serde_json::to_value(ApiDoc::openapi()).unwrap();
    document["paths"]["/not-allowed"] = json!({"get":{"operationId":"arbitrarySQL"}});
    let derived = from_openapi(&document);
    assert_eq!(derived.len(), ALLOWLIST.len());
    assert!(!derived.iter().any(|tool| tool.name == "arbitrarySQL"));
    for tool in &derived {
        let schema = Value::Object(tool.input_schema.as_ref().clone());
        jsonschema::validator_for(&schema).unwrap();
        assert!(!schema.to_string().contains("#/components/schemas/"));
        assert!(
            tool.description
                .as_ref()
                .is_some_and(|description| !description.is_empty())
        );
    }
    assert!(derived.iter().all(|tool| {
        !tool.input_schema["$defs"]
            .as_object()
            .unwrap()
            .contains_key("CatalogStatus")
    }));
    let page = derived
        .iter()
        .find(|tool| tool.name == "decksList")
        .unwrap();
    let validator =
        jsonschema::validator_for(&Value::Object(page.input_schema.as_ref().clone())).unwrap();
    assert!(validator.is_valid(&json!({})));
    for invalid in [
        json!({"limit":0}),
        json!({"limit":101}),
        json!({"limit":"40"}),
        json!({"sql":"SELECT 1"}),
    ] {
        assert!(!validator.is_valid(&invalid));
    }
    document["components"]["schemas"]["CreateDeck"]["properties"]["name"]["maxLength"] = json!(3);
    let changed = from_openapi(&document);
    let create = changed
        .iter()
        .find(|tool| tool.name == "decksCreate")
        .unwrap();
    let validator =
        jsonschema::validator_for(&Value::Object(create.input_schema.as_ref().clone())).unwrap();
    assert!(!validator.is_valid(&json!({"idempotency_key":"new","input":{"name":"Long name","format":"standard","cards":[]}})));
    assert_eq!(tools().len(), ALLOWLIST.len());
}
