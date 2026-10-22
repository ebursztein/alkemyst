use serde::Deserialize;
use serde_json::Value;
use std::sync::LazyLock;
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Rule {
    Basilisk,
    BusterSword,
    Chainsaw,
    Cloud,
    Tentacles,
    Dragonfire,
    Mauler,
    Dain,
    Freya,
    Giott,
    Vantage,
    Kellan,
    Kili,
    Mjolnir,
    Mountain,
    Plains,
    Raubahn,
    Foundry,
    Sheltered,
    Verge,
    Sundown,
    Lonely,
    Thor,
    Thorin,
}
#[derive(Deserialize)]
struct Registered {
    oracle_id: String,
    name: String,
    fingerprint: String,
    rule: Rule,
}
static REGISTRY: LazyLock<Vec<Registered>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("registry.json")).expect("reviewed compiler registry")
});
pub(super) fn rule(card: &Value) -> Result<Option<Rule>, ()> {
    let Some(id) = card["oracle_id"].as_str() else {
        return Ok(None);
    };
    let Some(registered) = REGISTRY.iter().find(|r| r.oracle_id == id) else {
        return Ok(None);
    };
    if card["name"] != registered.name
        || super::fingerprint::fingerprint(card) != registered.fingerprint
    {
        return Err(());
    }
    Ok(Some(registered.rule))
}
