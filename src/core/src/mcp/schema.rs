use crate::ApiDoc;
use rmcp::model::{Tool, ToolAnnotations};
use serde_json::{Value, json};
use utoipa::OpenApi;

// Adding a REST endpoint never grants an agent a new capability implicitly.
pub const ALLOWLIST: &[&str] = &[
    "gamesList",
    "gameGet",
    "gameTimeline",
    "gameTurns",
    "matchesList",
    "matchGet",
    "statisticsGet",
    "captureStatusGet",
    "systemDiscovery",
    "cardsSearch",
    "cardsGet",
    "cardsResolveArena",
    "cardsResolveReferences",
    "catalogStatusGet",
    "catalogRefresh",
    "decksList",
    "decksCreate",
    "deckVersionGet",
    "deckVersionCreate",
    "deckArenaImport",
    "deckArenaExport",
    "notesList",
    "notesCreate",
];

pub fn tools() -> Vec<Tool> {
    let document = serde_json::to_value(ApiDoc::openapi()).expect("OpenAPI serialization");
    from_openapi(&document)
}

pub fn from_openapi(document: &Value) -> Vec<Tool> {
    let mut tools = vec![];
    for path in document["paths"]
        .as_object()
        .expect("OpenAPI paths")
        .values()
    {
        for (method, operation) in path.as_object().expect("OpenAPI path item") {
            let Some(id) = operation["operationId"].as_str() else {
                continue;
            };
            if !ALLOWLIST.contains(&id) {
                continue;
            }
            let mut input =
                operation["requestBody"]["content"]["application/json"]["schema"].clone();
            if input.is_null() {
                let mut properties = serde_json::Map::new();
                let mut required = vec![];
                for param in operation["parameters"].as_array().into_iter().flatten() {
                    let name = param["name"].as_str().expect("OpenAPI parameter name");
                    properties.insert(name.into(), param["schema"].clone());
                    if param["required"] == true {
                        required.push(name);
                    }
                }
                input = json!({"type":"object", "properties":properties, "required":required, "additionalProperties":false});
            }
            // Tool roots must be objects, so resolve the request's top-level reference.
            if let Some(reference) = input["$ref"].as_str() {
                input = document
                    .pointer(reference.strip_prefix('#').expect("local ref"))
                    .expect("OpenAPI schema")
                    .clone();
            }
            localize(&mut input, document);
            let mut tool = Tool::new(
                id.to_owned(),
                operation["description"]
                    .as_str()
                    .or(operation["summary"].as_str())
                    .unwrap_or(id)
                    .to_owned(),
                input.as_object().expect("object schema").clone(),
            );
            let mut output = json!({"type":"object","properties":{"data":operation["responses"]["200"]["content"]["application/json"]["schema"]},"required":["data"],"additionalProperties":false});
            localize(&mut output, document);
            tool.output_schema = Some(std::sync::Arc::new(output.as_object().unwrap().clone()));
            let read_only = method == "get"
                || ["cardsSearch", "cardsResolveArena", "cardsResolveReferences"].contains(&id);
            let mut annotations = ToolAnnotations::default();
            annotations.read_only_hint = Some(read_only);
            annotations.destructive_hint = Some(false);
            annotations.idempotent_hint = Some(true);
            annotations.open_world_hint = Some(id == "catalogRefresh");
            tool.annotations = Some(annotations);
            tools.push(tool);
        }
    }
    tools.sort_by(|a, b| a.name.cmp(&b.name));
    assert_eq!(
        tools.len(),
        ALLOWLIST.len(),
        "Every allowed tool must have one OpenAPI operation"
    );
    tools
}

fn rewrite_refs(value: &mut Value) {
    match value {
        Value::Object(object) => {
            if let Some(Value::String(reference)) = object.get_mut("$ref") {
                *reference = reference.replace("#/components/schemas/", "#/$defs/");
            }
            for value in object.values_mut() {
                rewrite_refs(value);
            }
        }
        Value::Array(values) => {
            for value in values {
                rewrite_refs(value);
            }
        }
        _ => {}
    }
}

fn localize(schema: &mut Value, document: &Value) {
    let mut definitions = serde_json::Map::new();
    collect_definitions(schema, document, &mut definitions);
    schema["$defs"] = Value::Object(definitions);
    rewrite_refs(schema);
}

fn collect_definitions(
    value: &Value,
    document: &Value,
    definitions: &mut serde_json::Map<String, Value>,
) {
    match value {
        Value::Object(object) => {
            if let Some(reference) = object.get("$ref").and_then(Value::as_str) {
                let name = reference
                    .strip_prefix("#/components/schemas/")
                    .expect("Only local OpenAPI references are supported");
                if !definitions.contains_key(name) {
                    let definition = document
                        .pointer(reference.strip_prefix('#').unwrap())
                        .expect("Referenced OpenAPI schema")
                        .clone();
                    // Insert before walking to terminate recursive types.
                    definitions.insert(name.into(), definition.clone());
                    collect_definitions(&definition, document, definitions);
                }
            }
            for child in object.values() {
                collect_definitions(child, document, definitions);
            }
        }
        Value::Array(values) => {
            for child in values {
                collect_definitions(child, document, definitions);
            }
        }
        _ => {}
    }
}
