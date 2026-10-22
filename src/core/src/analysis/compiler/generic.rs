use crate::analysis::turns::SpellSpec;
use serde_json::Value;
const KEYWORDS: &[&str] = &[
    "flying",
    "haste",
    "first strike",
    "double strike",
    "trample",
    "vigilance",
    "lifelink",
    "deathtouch",
    "reach",
    "menace",
];
pub(super) fn creature(types: &str, front: &Value) -> Option<SpellSpec> {
    if !types.contains("Creature") {
        return None;
    }
    let text = front["oracle_text"].as_str()?;
    if text.is_empty() && front["keywords"].as_array().is_some_and(|a| !a.is_empty()) {
        return None;
    }
    if !text.is_empty()
        && !text
            .split([',', '\n'])
            .all(|s| KEYWORDS.contains(&s.trim().to_ascii_lowercase().as_str()))
    {
        return None;
    }
    if front["keywords"].as_array().is_some_and(|a| {
        a.iter().any(|v| {
            v.as_str()
                .is_none_or(|s| !KEYWORDS.contains(&s.to_ascii_lowercase().as_str()))
        })
    }) {
        return None;
    }
    Some(SpellSpec {
        permanent: true,
        creature: true,
        artifact: types.contains("Artifact"),
        haste: text
            .split([',', '\n'])
            .any(|s| s.trim().eq_ignore_ascii_case("haste")),
        ..Default::default()
    })
}
