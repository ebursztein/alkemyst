use alkemyst_core::{
    db::{Engine, Mutation, Write},
    domain::{CreateDeck, DeckCard, Decks, Origin, Zone},
};
use serde_json::json;

fn input() -> CreateDeck {
    CreateDeck {
        name: "Fixture".into(),
        format: "standard".into(),
        cards: vec![DeckCard {
            reference: "arena:1".into(),
            quantity: 60,
            zone: Zone::Main,
        }],
    }
}

#[tokio::test]
async fn sealed_versions_reject_all_history_changes_in_sqlite() {
    let db = Engine::memory(1024 * 1024).unwrap();
    let decks = Decks::new(db.clone());
    let version = decks
        .create("immutable", input(), Origin::Arena)
        .await
        .unwrap();
    assert_eq!(
        decks.version(&version.id).await.unwrap().as_ref(),
        Some(&version)
    );
    let before = db.metrics().await.unwrap().cache_hits;
    for sql in [
        "UPDATE deck_versions SET name='Edited' WHERE id=?1",
        "UPDATE deck_versions SET sealed=0 WHERE id=?1",
        "DELETE FROM deck_versions WHERE id=?1",
        "INSERT INTO deck_cards VALUES (?1, 'arena:2', 1, 'main')",
        "UPDATE deck_cards SET quantity=1 WHERE version_id=?1",
        "DELETE FROM deck_cards WHERE version_id=?1",
    ] {
        assert!(
            db.write(vec![Write {
                statement: Mutation {
                    sql,
                    writes: &["deck_versions", "deck_cards"]
                },
                params: vec![json!(version.id)],
                expected_rows: None,
            }])
            .await
            .is_err()
        );
        assert_eq!(
            decks.version(&version.id).await.unwrap().as_ref(),
            Some(&version)
        );
    }
    assert_eq!(db.metrics().await.unwrap().cache_hits - before, 6);
}

#[tokio::test]
async fn malformed_inputs_fail_before_any_write_and_pagination_is_bounded() {
    let db = Engine::memory(1024 * 1024).unwrap();
    let decks = Decks::new(db.clone());
    let mut cases = Vec::new();
    let mut invalid = input();
    invalid.name = " ".into();
    cases.push(invalid);
    let mut invalid = input();
    invalid.name = "x".repeat(201);
    cases.push(invalid);
    let mut invalid = input();
    invalid.format = String::new();
    cases.push(invalid);
    let mut invalid = input();
    invalid.cards[0].quantity = 0;
    cases.push(invalid);
    let mut invalid = input();
    invalid.cards[0].quantity = 251;
    cases.push(invalid);
    for reference in [
        "unknown",
        "oracle:",
        "raw:1",
        "arena:0",
        "arena:no",
        "scryfall:bad/path",
    ] {
        let mut invalid = input();
        invalid.cards[0].reference = reference.into();
        cases.push(invalid);
    }
    let mut invalid = input();
    invalid.cards = vec![invalid.cards[0].clone(); 501];
    cases.push(invalid);
    let mut invalid = input();
    invalid.cards = vec![invalid.cards[0].clone(); 5];
    cases.push(invalid);
    for case in cases {
        assert!(decks.create("invalid", case, Origin::User).await.is_err());
    }
    for key in ["".to_string(), "x".repeat(129), "bad\nkey".into()] {
        assert!(decks.create(&key, input(), Origin::User).await.is_err());
    }
    assert!(
        !db.metrics()
            .await
            .unwrap()
            .table_revisions
            .contains_key("decks")
    );
    assert!(decks.list(0, None).await.is_err());
    assert!(decks.list(101, None).await.is_err());
    assert!(decks.list(1, Some("x".repeat(129))).await.is_err());
    for i in 0..3 {
        decks
            .create(&format!("new-{i}"), input(), Origin::User)
            .await
            .unwrap();
    }
    let first = decks.list(2, None).await.unwrap();
    let second = decks.list(2, first.next_cursor).await.unwrap();
    assert_eq!(first.items.len() + second.items.len(), 3);
    assert!(second.next_cursor.is_none());
}
