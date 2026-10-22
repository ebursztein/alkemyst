use serde_json::{Value, json};
use sha2::{Digest, Sha256};
fn face(card: &Value) -> Value {
    json!({"name":card["name"],"mana_cost":card["mana_cost"],"type_line":card["type_line"],"oracle_text":card["oracle_text"],"colors":card["colors"],"keywords":card["keywords"]})
}
/// Only rules-relevant fields are signed. Printing IDs, artwork, price and
/// unrelated metadata can change without invalidating a reviewed model.
pub(super) fn fingerprint(card: &Value) -> String {
    let mut canonical = face(card);
    canonical["oracle_id"] = card["oracle_id"].clone();
    canonical["cmc"] = json!(card["cmc"].as_f64());
    canonical["card_faces"] = card["card_faces"]
        .as_array()
        .map(|faces| faces.iter().map(face).collect::<Vec<_>>())
        .map_or(Value::Null, |v| json!(v));
    Sha256::digest(canonical.to_string().as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
