use alkemyst_core::{
    db::Engine,
    domain::{CreateDeck, Decks, NewNote, Origin},
};

#[tokio::test]
async fn notes_are_idempotent_and_do_not_rewrite_versions() {
    let decks = Decks::new(Engine::memory(1024 * 1024).unwrap());
    let version = decks
        .create(
            "deck",
            CreateDeck {
                name: "Boros".into(),
                format: "standard".into(),
                cards: vec![],
            },
            Origin::User,
        )
        .await
        .unwrap();
    let request = NewNote {
        deck_id: version.deck_id.clone(),
        version_id: Some(version.id.clone()),
        body: "Try another land.".into(),
    };
    let note = decks
        .create_note("note", request.clone(), Origin::Agent)
        .await
        .unwrap();
    let again = decks
        .create_note("note", request, Origin::Agent)
        .await
        .unwrap();
    assert_eq!(note, again);
    assert_eq!(note.origin, Origin::Agent);
    assert_eq!(decks.version(&version.id).await.unwrap(), Some(version));
    let page = decks.notes(&note.deck_id, 10, None).await.unwrap();
    assert_eq!(page.items.len(), 1);
    assert!(page.next_cursor.is_none());
    let bad = NewNote {
        deck_id: note.deck_id.clone(),
        version_id: None,
        body: "Different".into(),
    };
    assert!(decks.create_note("note", bad, Origin::Agent).await.is_err());
    let missing = NewNote {
        deck_id: "missing".into(),
        version_id: None,
        body: "Missing deck".into(),
    };
    assert!(
        decks
            .create_note("missing", missing, Origin::User)
            .await
            .is_err()
    );
    for body in [" ".into(), "x".repeat(20001)] {
        assert!(
            decks
                .create_note(
                    "invalid",
                    NewNote {
                        deck_id: note.deck_id.clone(),
                        version_id: None,
                        body
                    },
                    Origin::User
                )
                .await
                .is_err()
        );
    }
}
