use alkemyst_core::{
    catalog::{Catalog, CatalogSource},
    db::Engine,
    domain::{CreateDeck, DeckCard, Decks, Origin, Zone},
    practice::Practice,
};
use serde_json::{Value, json};
pub async fn setup(db: Engine) -> (Practice, String) {
    let mut rows = include_str!("../fixtures/cards.jsonl")
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .collect::<Vec<_>>();
    let mut land = rows[0].clone();
    land["id"] = json!("print-land");
    land["oracle_id"] = json!("oracle-land");
    land["name"] = json!("Plains");
    land["type_line"] = json!("Basic Land — Plains");
    land["cmc"] = json!(0);
    land["oracle_text"] = json!("{T}: Add {W}.");
    rows.push(land);
    Catalog::new(db.clone())
        .import_jsonl(
            std::io::Cursor::new(
                rows.iter()
                    .map(Value::to_string)
                    .collect::<Vec<_>>()
                    .join("\n"),
            ),
            CatalogSource {
                url: "https://data.scryfall.io/default-cards/fixture.jsonl.gz".into(),
                updated_at: "2026-10-21T21:00:00Z".into(),
            },
        )
        .await
        .unwrap();
    let v = Decks::new(db.clone())
        .create(
            "practice-deck",
            CreateDeck {
                name: "Practice fixture".into(),
                format: "test".into(),
                cards: vec![
                    DeckCard {
                        reference: "oracle:oracle-equipment".into(),
                        quantity: 7,
                        zone: Zone::Main,
                    },
                    DeckCard {
                        reference: "oracle:oracle-land".into(),
                        quantity: 7,
                        zone: Zone::Main,
                    },
                    DeckCard {
                        reference: "oracle:oracle-tutor".into(),
                        quantity: 3,
                        zone: Zone::Sideboard,
                    },
                ],
            },
            Origin::User,
        )
        .await
        .unwrap();
    (Practice::new(db), v.id)
}
