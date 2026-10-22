use serde_json::Value;

#[test]
fn sanctioned_boros_fixture_preserves_real_card_rules_without_capture_identity() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/boros-equipment.json");
    let value: Value = serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
    let cards = value["cards"].as_array().unwrap();
    assert_eq!(
        cards
            .iter()
            .map(|c| c["quantity"].as_u64().unwrap())
            .sum::<u64>(),
        60
    );
    let cloud = cards
        .iter()
        .find(|c| c["card"]["name"] == "Cloud, Midgar Mercenary")
        .unwrap();
    assert_eq!(cloud["card"]["mana_cost"], "{W}{W}");
    let kellan = cards
        .iter()
        .find(|c| c["card"]["name"] == "Kellan, the Fae-Blooded // Birthright Boon")
        .unwrap();
    assert_eq!(kellan["card"]["card_faces"][1]["mana_cost"], "{1}{W}");
    for entry in cards {
        assert_eq!(entry.as_object().unwrap().len(), 2);
        for key in entry["card"].as_object().unwrap().keys() {
            assert!(
                [
                    "oracle_id",
                    "name",
                    "mana_cost",
                    "cmc",
                    "type_line",
                    "oracle_text",
                    "colors",
                    "keywords",
                    "card_faces"
                ]
                .contains(&key.as_str()),
                "unexpected captured field {key}"
            );
        }
    }
    assert!(value.get("source_path").is_none());
    assert!(value.get("deck_id").is_none());
}
