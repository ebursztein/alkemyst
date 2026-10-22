use alkemyst_core::analysis::{simulation::BattlefieldRunner, turns::Goal};
use alkemyst_core::{
    analysis::{
        Analysis,
        compiler::{IssueCode, compile},
    },
    catalog::Catalog,
    db::Engine,
    domain::{CreateDeck, DeckCard, Decks, Origin, Zone},
};
use landlord::London;
use serde_json::{Value, json};

async fn snapshot() -> alkemyst_core::analysis::DeckSnapshot {
    let db = Engine::memory(4 * 1024 * 1024).unwrap();
    let catalog = Catalog::new(db.clone());
    let fixture: Value =
        serde_json::from_str(include_str!("fixtures/boros-equipment.json")).unwrap();
    let records = fixture["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            let mut card = entry["card"].clone();
            card["id"] = json!(format!("printing-{}", card["oracle_id"].as_str().unwrap()));
            card["object"] = json!("card");
            card["lang"] = json!("en");
            card["released_at"] = json!("2026-01-01");
            card.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    catalog
        .import_jsonl(
            std::io::Cursor::new(records),
            alkemyst_core::catalog::CatalogSource {
                url: "https://data.scryfall.io/default-cards/fixture.jsonl.gz".into(),
                updated_at: "2026-10-21T21:00:00Z".into(),
            },
        )
        .await
        .unwrap();
    let cards = fixture["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| DeckCard {
            reference: format!("oracle:{}", e["card"]["oracle_id"].as_str().unwrap()),
            quantity: e["quantity"].as_u64().unwrap() as u16,
            zone: Zone::Main,
        })
        .collect();
    let v = Decks::new(db.clone())
        .create(
            "actual-model",
            CreateDeck {
                name: "Sanitized Boros benchmark".into(),
                format: "historic".into(),
                cards,
            },
            Origin::Import,
        )
        .await
        .unwrap();
    Analysis::new(db).snapshot(&v.id).await.unwrap().unwrap()
}

#[tokio::test]
async fn actual_boros_deck_compiles_and_simulates_with_explicit_exclusions() {
    let raw = snapshot().await;
    let compiled = compile(raw);
    assert!(
        compiled.profile.can_simulate,
        "{:?}",
        compiled.profile.cards
    );
    assert_eq!(
        (
            compiled.profile.counts.main,
            compiled.profile.counts.lands,
            compiled.profile.counts.equipment
        ),
        (60, 23, 10)
    );
    let kellan = compiled
        .profile
        .cards
        .iter()
        .find(|c| c.name.as_deref() == Some("Kellan, the Fae-Blooded // Birthright Boon"))
        .unwrap();
    assert_eq!(kellan.cast_cost.as_deref(), Some("{2}{R}"));
    assert_eq!(kellan.adventure_cost.as_deref(), Some("{1}{W}"));
    assert!(
        compiled
            .scenario(
                8,
                true,
                London::never(),
                Goal {
                    equipment: 2,
                    attached: 1,
                    attack_ready: 1
                },
                false
            )
            .is_err()
    );
    let scenario = compiled
        .scenario(
            8,
            true,
            London::never(),
            Goal {
                equipment: 2,
                attached: 1,
                attack_ready: 1,
            },
            true,
        )
        .unwrap();
    let mut runner = BattlefieldRunner::new(scenario, 200, 2026).unwrap();
    while !runner.finished() {
        runner.advance(50).unwrap();
    }
    let report = runner.report().unwrap();
    assert_eq!(report.completed_runs, 200);
    assert!(report.turns[7].mean_equipment >= report.turns[0].mean_equipment);
    assert!(report.turns[7].story_samples > 0);
}

#[tokio::test]
async fn a_changed_registered_rule_blocks_instead_of_reusing_a_stale_model() {
    let mut raw = snapshot().await;
    let cloud = raw
        .cards
        .iter_mut()
        .find(|c| c.data.as_ref().unwrap()["name"] == "Cloud, Midgar Mercenary")
        .unwrap();
    cloud.data.as_mut().unwrap()["oracle_text"] = json!("When this enters, draw twenty cards.");
    let compiled = compile(raw);
    assert!(!compiled.profile.can_simulate);
    assert!(
        compiled
            .profile
            .cards
            .iter()
            .any(|c| c.issues.iter().any(|i| i.code == IssueCode::RulesChanged))
    );
}

#[tokio::test]
async fn unknown_resource_effects_missing_cards_and_unmodeled_costs_never_become_plain_creatures() {
    for (field, value, expected) in [
        (
            "oracle_text",
            json!("{T}: Draw seven cards."),
            IssueCode::UnknownRules,
        ),
        ("mana_cost", json!("{X}{R}"), IssueCode::UnsupportedCost),
    ] {
        let mut raw = snapshot().await;
        let card = raw.cards[0].data.as_mut().unwrap();
        card.as_object_mut().unwrap().remove("card_faces");
        card["oracle_id"] = json!("custom-card");
        card["name"] = json!("Unreviewed creature");
        card["type_line"] = json!("Creature — Dwarf");
        card["oracle_text"] = json!("");
        card["mana_cost"] = json!("{R}");
        card[field] = value;
        let compiled = compile(raw);
        assert!(!compiled.profile.can_simulate);
        assert!(
            compiled
                .profile
                .cards
                .iter()
                .any(|c| c.issues.iter().any(|i| i.code == expected))
        );
    }
    let mut raw = snapshot().await;
    raw.cards[0].data = None;
    let compiled = compile(raw);
    assert!(!compiled.profile.can_simulate);
    assert!(
        compiled.profile.cards[0]
            .issues
            .iter()
            .any(|i| i.code == IssueCode::MissingCard)
    );
}

#[tokio::test]
async fn absence_of_a_cast_cost_is_not_a_zero_mana_spell() {
    let mut raw = snapshot().await;
    let card = raw.cards[0].data.as_mut().unwrap();
    card.as_object_mut().unwrap().remove("card_faces");
    card["oracle_id"] = json!("no-cost-creature");
    card["name"] = json!("No printed cost");
    card["type_line"] = json!("Creature");
    card["oracle_text"] = json!("");
    card["mana_cost"] = json!("");
    card["keywords"] = json!([]);
    let result = compile(raw);
    assert!(!result.profile.can_simulate);
    assert!(
        result.profile.cards[0]
            .issues
            .iter()
            .any(|i| i.code == IssueCode::UnsupportedCost)
    );
}

#[tokio::test]
async fn unknown_keywords_are_not_discarded_when_rules_text_is_empty() {
    let mut raw = snapshot().await;
    let card = raw.cards[0].data.as_mut().unwrap();
    card.as_object_mut().unwrap().remove("card_faces");
    card["oracle_id"] = json!("custom-keyword");
    card["name"] = json!("Unknown keyword creature");
    card["type_line"] = json!("Creature");
    card["mana_cost"] = json!("{R}");
    card["oracle_text"] = json!("");
    card["keywords"] = json!(["Landfall"]);
    assert!(!compile(raw).profile.can_simulate);
}
#[path = "analysis_compiler/cases.rs"]
mod cases;
