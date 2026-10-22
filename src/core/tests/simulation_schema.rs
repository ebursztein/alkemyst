use alkemyst_core::mcp::schema::tools;
use serde_json::Value;
#[test]
fn async_creation_and_exact_math_tools_keep_typed_output_and_read_only_hints() {
    let tools = tools();
    let create = tools
        .iter()
        .find(|t| t.name == "simulationsCreate")
        .unwrap();
    let output = Value::Object(create.output_schema.as_ref().unwrap().as_ref().clone());
    jsonschema::validator_for(&output).unwrap();
    assert_eq!(
        output["properties"]["data"]["$ref"],
        "#/$defs/SimulationJob"
    );
    for name in ["probabilityDraw", "probabilityJoint"] {
        let tool = tools.iter().find(|t| t.name == name).unwrap();
        assert_eq!(
            tool.annotations.as_ref().unwrap().read_only_hint,
            Some(true)
        );
    }
}
