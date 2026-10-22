use alkemyst_core::{
    analysis::{
        Analysis,
        jobs::{ModelScope, MulliganParameters, SimulationParameters},
    },
    catalog::{Catalog, CatalogSource},
    db::Engine,
    domain::{CreateDeck, DeckCard, Decks, Origin, Zone},
};
use serde_json::{Value, json};
pub async fn setup() -> (Engine, Analysis, String) {
    populate(Engine::memory(8 * 1024 * 1024).unwrap()).await
}
pub async fn populate(db: Engine) -> (Engine, Analysis, String) {
    let catalog = Catalog::new(db.clone());
    let fixture: Value =
        serde_json::from_str(include_str!("../fixtures/boros-equipment.json")).unwrap();
    let rows = fixture["cards"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            let mut c = e["card"].clone();
            c["id"] = json!(format!("print-{}", c["oracle_id"].as_str().unwrap()));
            c["object"] = json!("card");
            c["lang"] = json!("en");
            c["released_at"] = json!("2026-01-01");
            c.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    catalog
        .import_jsonl(
            std::io::Cursor::new(rows),
            CatalogSource {
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
    let version = Decks::new(db.clone())
        .create(
            "jobs-deck",
            CreateDeck {
                name: "Sanitized job test".into(),
                format: "historic".into(),
                cards,
            },
            Origin::Import,
        )
        .await
        .unwrap();
    (db.clone(), Analysis::new(db), version.id)
}
pub fn parameters(version: &str) -> SimulationParameters {
    SimulationParameters {
        version_id: version.into(),
        runs: 100,
        turns: 5,
        on_the_play: true,
        seed: Some("ffffffffffffffff".into()),
        scope: ModelScope::PrecombatGoldfish,
        goal: Default::default(),
        mulligan: MulliganParameters::default(),
    }
}
