use alkemyst_core::{
    db::Engine,
    domain::{CreateDeck, DeckCard, Decks, NewVersion, Origin, Zone},
};
#[tokio::test]
async fn version_history_is_one_cached_read_with_stable_pages_and_creation_invalidation() {
    let db = Engine::memory(4 * 1024 * 1024).unwrap();
    let decks = Decks::new(db.clone());
    let first = decks
        .create(
            "history",
            CreateDeck {
                name: "Original".into(),
                format: "test".into(),
                cards: vec![DeckCard {
                    reference: "oracle:unknown".into(),
                    quantity: 60,
                    zone: Zone::Main,
                }],
            },
            Origin::User,
        )
        .await
        .unwrap();
    let mut parent = first.id.clone();
    for i in 2..=4 {
        parent = decks
            .create_version(
                &format!("v-{i}"),
                NewVersion {
                    deck_id: first.deck_id.clone(),
                    parent_version: Some(parent),
                    name: format!("Version {i}"),
                    format: "test".into(),
                    cards: vec![
                        DeckCard {
                            reference: "oracle:unknown".into(),
                            quantity: 40,
                            zone: Zone::Main,
                        },
                        DeckCard {
                            reference: "oracle:side".into(),
                            quantity: 15,
                            zone: Zone::Sideboard,
                        },
                    ],
                },
                Origin::User,
            )
            .await
            .unwrap()
            .id;
    }
    let before = db.metrics().await.unwrap().statements;
    let page = decks
        .versions(&first.deck_id, 2, None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(db.metrics().await.unwrap().statements, before + 1);
    assert_eq!(
        page.items.iter().map(|v| v.ordinal).collect::<Vec<_>>(),
        vec![4, 3]
    );
    assert_eq!(
        (page.items[0].main_count, page.items[0].sideboard_count),
        (40, 15)
    );
    let warm = db.metrics().await.unwrap().statements;
    assert_eq!(
        decks
            .versions(&first.deck_id, 2, None)
            .await
            .unwrap()
            .unwrap(),
        page
    );
    assert_eq!(db.metrics().await.unwrap().statements, warm);
    let next = decks
        .versions(&first.deck_id, 2, page.next_cursor)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        next.items.iter().map(|v| v.ordinal).collect::<Vec<_>>(),
        vec![2, 1]
    );
    assert!(next.next_cursor.is_none());
    decks
        .create_version(
            "v-5",
            NewVersion {
                deck_id: first.deck_id.clone(),
                parent_version: Some(parent),
                name: "Version 5".into(),
                format: "test".into(),
                cards: first.cards,
            },
            Origin::Agent,
        )
        .await
        .unwrap();
    let refreshed = decks
        .versions(&first.deck_id, 2, None)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(refreshed.items[0].ordinal, 5);
    assert_eq!(refreshed.items[0].origin, Origin::Agent);
    assert!(decks.versions("missing", 2, None).await.unwrap().is_none());
    for cursor in [
        "bad",
        "v1:0000000000000000",
        "v1:ffffffffffffffff",
        "v1:+000000000000001",
    ] {
        assert!(
            decks
                .versions(&first.deck_id, 2, Some(cursor.into()))
                .await
                .is_err()
        );
    }
    assert!(decks.versions("", 2, None).await.is_err());
    assert!(decks.versions(&"v".repeat(257), 2, None).await.is_err());
    for limit in [0, 101] {
        assert!(decks.versions(&first.deck_id, limit, None).await.is_err());
    }
}

#[tokio::test]
async fn version_history_seeks_the_deck_ordinal_and_only_reads_selected_card_rows() {
    use alkemyst_core::db::Statement;
    let db = Engine::memory(4096).unwrap();
    let plan = db
        .explain(
            Statement {
                name: "test.versions.plan",
                sql: include_str!("../src/statements/decks/versions.sql"),
                reads: &[],
                cacheable: false,
            },
            vec![
                serde_json::json!("deck"),
                serde_json::json!(100),
                serde_json::json!(20),
            ],
        )
        .await
        .unwrap()
        .join("\n");
    assert!(plan.contains("deck_id=? AND ordinal<?"), "{plan}");
    assert!(!plan.contains("SCAN deck_versions"), "{plan}");
    assert!(!plan.contains("SCAN c\n"), "{plan}");
}
