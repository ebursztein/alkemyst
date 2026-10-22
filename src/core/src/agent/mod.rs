//! Application-owned agent capabilities shared by MCP and local chat.
mod operations;
pub(crate) mod schema;
pub use schema::ALLOWLIST;

use crate::{Application, api::problem::ApiError};
use serde_json::{Map, Value};
use std::{collections::BTreeMap, sync::OnceLock};

/// Provider-neutral contract derived from an explicitly allowed OpenAPI operation.
#[derive(Clone, Debug)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Map<String, Value>,
    pub output_schema: Map<String, Value>,
    pub read_only: bool,
    pub open_world: bool,
}

struct Registry {
    tools: Vec<Tool>,
    validators: BTreeMap<String, jsonschema::Validator>,
}

fn registry() -> &'static Registry {
    static REGISTRY: OnceLock<Registry> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let tools = schema::tools();
        let validators = tools
            .iter()
            .map(|tool| {
                let input = Value::Object(tool.input_schema.clone());
                (
                    tool.name.clone(),
                    jsonschema::validator_for(&input).expect("Valid OpenAPI tool schema"),
                )
            })
            .collect();
        Registry { tools, validators }
    })
}

pub fn tools() -> &'static [Tool] {
    &registry().tools
}

pub(crate) async fn invoke(
    app: &Application,
    name: &str,
    arguments: Value,
) -> Result<Value, ApiError> {
    let validator = registry()
        .validators
        .get(name)
        .ok_or_else(ApiError::missing)?;
    if !validator.is_valid(&arguments) {
        return Err(ApiError::invalid(
            "Arguments do not match the tool input schema",
        ));
    }
    operations::invoke(app, name, arguments).await
}
