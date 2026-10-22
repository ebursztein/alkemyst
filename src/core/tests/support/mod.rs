use alkemyst_core::catalog::CatalogSource;
use serde_json::json;

pub fn source() -> CatalogSource {
    CatalogSource {
        url: "https://data.scryfall.io/default-cards/fixture.jsonl.gz".into(),
        updated_at: "2026-10-21T21:00:00Z".into(),
    }
}

pub fn cards() -> String {
    let base = json!({
        "object":"card", "id":"print-equipment", "oracle_id":"oracle-equipment",
        "arena_id":42, "name":"Forgeblade", "layout":"normal", "cmc":2,
        "mana_cost":"{1}{W}", "type_line":"Legendary Artifact — Equipment",
        "oracle_text":"Equipped creature gets +1/+1. Equip {1}", "colors":["W"],
        "color_identity":["W"], "lang":"en", "set":"tst", "collector_number":"1",
        "released_at":"2026-01-01", "legalities":{"standard":"legal","modern":"legal"},
        "image_uris":{"normal":"https://cards.scryfall.io/normal/fixture.jpg"},
        "custom_public_field":{"preserved":true}
    });
    let mut reprint = base.clone();
    reprint["id"] = json!("print-equipment-old");
    reprint["arena_id"] = json!(null);
    reprint["released_at"] = json!("2025-01-01");
    let tutor = json!({
        "object":"card", "id":"print-tutor", "oracle_id":"oracle-tutor", "arena_id":43,
        "name":"Find the Blade", "layout":"normal", "cmc":1, "mana_cost":"{W}",
        "type_line":"Sorcery", "oracle_text":"Search your library for an Equipment card.",
        "colors":["W"], "color_identity":["W"], "lang":"en", "set":"tst", "collector_number":"2",
        "released_at":"2026-01-01", "legalities":{"standard":"not_legal","modern":"legal"}
    });
    let modal = json!({
        "object":"card", "id":"print-modal", "oracle_id":"oracle-modal", "name":"Ember // Meadow",
        "layout":"modal_dfc", "cmc":3, "type_line":"Sorcery // Land", "color_identity":["R","W"],
        "lang":"en", "set":"tst", "collector_number":"3", "released_at":"2026-01-01",
        "legalities":{"standard":"legal"},
        "card_faces":[{"name":"Ember","mana_cost":"{2}{R}","oracle_text":"Deal 3 damage."},
                      {"name":"Meadow","mana_cost":"","oracle_text":"This land enters tapped."}]
    });
    [base, reprint, tutor, modal]
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}
