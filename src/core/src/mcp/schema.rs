//! MCP representation of the shared, explicitly allowed application contracts.
use crate::agent;
pub use crate::agent::ALLOWLIST;
use rmcp::model::{Tool, ToolAnnotations};
use serde_json::Value;

pub fn tools() -> Vec<Tool> {
    agent::tools().iter().map(to_mcp).collect()
}

pub fn from_openapi(document: &Value) -> Vec<Tool> {
    agent::schema::from_openapi(document)
        .iter()
        .map(to_mcp)
        .collect()
}

fn to_mcp(contract: &agent::Tool) -> Tool {
    let mut tool = Tool::new(
        contract.name.clone(),
        contract.description.clone(),
        contract.input_schema.clone(),
    );
    tool.output_schema = Some(std::sync::Arc::new(contract.output_schema.clone()));
    let mut annotations = ToolAnnotations::default();
    annotations.read_only_hint = Some(contract.read_only);
    annotations.destructive_hint = Some(false);
    annotations.idempotent_hint = Some(true);
    annotations.open_world_hint = Some(contract.open_world);
    tool.annotations = Some(annotations);
    tool
}
