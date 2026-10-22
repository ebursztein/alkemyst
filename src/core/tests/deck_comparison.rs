mod support;
use alkemyst_core::{
    analysis::Analysis,
    catalog::Catalog,
    db::Engine,
    domain::{CreateDeck, DeckCard, Decks, NewVersion, Origin, Zone},
};
fn card(reference: &str, quantity: u16, zone: Zone) -> DeckCard {
    DeckCard {
        reference: reference.into(),
        quantity,
        zone,
    }
}
#[tokio::test]
async fn comparison_normalizes_printings_and_keeps_zones_missing_cards_and_curves_explicit() {
    let db = Engine::memory(4 * 1024 * 1024).unwrap();
    let decks = Decks::new(db.clone());
    let base = decks
        .create(
            "compare-base",
            CreateDeck {
                name: "Comparison".into(),
                format: "standard".into(),
                cards: vec![
                    card("oracle:oracle-equipment", 4, Zone::Main),
                    card("arena:999999", 1, Zone::Main),
                    card("oracle:oracle-tutor", 2, Zone::Sideboard),
                ],
            },
            Origin::User,
        )
        .await
        .unwrap();
    let candidate = decks
        .create_version(
            "compare-next",
            NewVersion {
                deck_id: base.deck_id.clone(),
                parent_version: Some(base.id.clone()),
                name: "Candidate".into(),
                format: "standard".into(),
                cards: vec![
                    card("scryfall:print-equipment-old", 3, Zone::Main),
                    card("arena:42", 1, Zone::Main),
                    card("oracle:oracle-tutor", 2, Zone::Main),
                    card("arena:999999", 1, Zone::Main),
                ],
            },
            Origin::User,
        )
        .await
        .unwrap();
    let analysis = Analysis::new(db.clone());
    let missing = analysis
        .compare(&base.id, &candidate.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(missing.base.unresolved_main, 5);
    assert_eq!(missing.candidate.unresolved_main, 7);
    Catalog::new(db.clone())
        .import_jsonl(std::io::Cursor::new(support::cards()), support::source())
        .await
        .unwrap();
    let before = db.metrics().await.unwrap();
    let result = analysis
        .compare(&base.id, &candidate.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        db.metrics().await.unwrap().statements,
        before.statements + 1
    );
    assert_eq!((result.base.main, result.candidate.main), (5, 7));
    assert_eq!(
        (
            result.base.unresolved_main,
            result.candidate.unresolved_main
        ),
        (1, 1)
    );
    assert!(result.catalog_snapshot.is_some());
    assert_eq!(result.cards.len(), 2);
    assert!(
        result
            .cards
            .iter()
            .all(|c| c.identity == "oracle:oracle-tutor")
    );
    let main = result.cards.iter().find(|c| c.zone == Zone::Main).unwrap();
    assert_eq!(
        (main.base_quantity, main.candidate_quantity, main.delta),
        (0, 2, 2)
    );
    let side = result
        .cards
        .iter()
        .find(|c| c.zone == Zone::Sideboard)
        .unwrap();
    assert_eq!(side.delta, -2);
    let equipment = result
        .curve
        .iter()
        .find(|c| c.mana_value == Some(2.0))
        .unwrap();
    assert_eq!(
        (
            equipment.base_quantity,
            equipment.candidate_quantity,
            equipment.delta
        ),
        (4, 4, 0)
    );
    assert_eq!(
        result
            .curve
            .iter()
            .find(|c| c.mana_value.is_none())
            .unwrap()
            .base_quantity,
        1
    );
    let warm = db.metrics().await.unwrap();
    assert_eq!(
        analysis
            .compare(&base.id, &candidate.id)
            .await
            .unwrap()
            .unwrap(),
        result
    );
    assert_eq!(db.metrics().await.unwrap().statements, warm.statements);
    assert!(
        analysis
            .compare(&base.id, "missing")
            .await
            .unwrap()
            .is_none()
    );
    let same = analysis.compare(&base.id, &base.id).await.unwrap().unwrap();
    assert!(same.cards.is_empty());
    assert!(same.curve.iter().all(|c| c.delta == 0));
    for bad in [String::new(), "v".repeat(257)] {
        assert!(analysis.compare(&bad, &base.id).await.is_err());
        assert!(analysis.compare(&base.id, &bad).await.is_err());
    }
}
