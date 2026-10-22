mod support;
use alkemyst_core::{
    catalog::Catalog,
    db::Engine,
    domain::{Decks, ImportDeck, Origin, Zone},
};
use support::{cards, source};

#[tokio::test]
async fn arena_text_import_export_and_retry_preserve_card_counts_and_zones() {
    let db = Engine::memory(1024 * 1024).unwrap();
    Catalog::new(db.clone())
        .import_jsonl(std::io::Cursor::new(cards()), source())
        .await
        .unwrap();
    let decks = Decks::new(db);
    let input=ImportDeck {name:"Boros".into(),format:"standard".into(),text:"About\nName Boros\n\nDeck\n4 Forgeblade (TST) 1\n2 Ember\n\nSideboard\n1 Find the Blade\n".into()};
    let version = decks
        .import_arena("text", input.clone(), Origin::Import)
        .await
        .unwrap();
    assert_eq!(version.cards.len(), 3);
    assert_eq!(
        version
            .cards
            .iter()
            .filter(|c| c.zone == Zone::Main)
            .map(|c| c.quantity)
            .sum::<u16>(),
        6
    );
    assert_eq!(
        decks
            .import_arena("text", input, Origin::Import)
            .await
            .unwrap(),
        version
    );
    let exported = decks.export_arena(&version.id).await.unwrap();
    assert!(exported.contains("4 Forgeblade (TST) 1"));
    assert!(exported.contains("Sideboard\n1 Find the Blade (TST) 2"));
    let again = decks
        .import_arena(
            "roundtrip",
            ImportDeck {
                name: "Copy".into(),
                format: "standard".into(),
                text: exported,
            },
            Origin::Import,
        )
        .await
        .unwrap();
    assert_eq!(again.cards, version.cards);
    assert!(decks.export_arena("missing").await.is_err());
}

#[tokio::test]
async fn unknown_or_malformed_card_lines_never_create_partial_decks() {
    let decks = Decks::new(Engine::memory(1024 * 1024).unwrap());
    for text in [
        "",
        "Deck\n4 Missing Card",
        "Deck\n0 Forgeblade",
        "Deck\nnot a card line",
        "Deck\n4 ",
    ] {
        assert!(
            decks
                .import_arena(
                    "invalid",
                    ImportDeck {
                        name: "Bad".into(),
                        format: "standard".into(),
                        text: text.into()
                    },
                    Origin::Import
                )
                .await
                .is_err()
        );
    }
    assert!(decks.list(10, None).await.unwrap().items.is_empty());
}
