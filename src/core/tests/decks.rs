use alkemyst_core::{
    db::Engine,
    domain::{CreateDeck, DeckCard, Decks, NewVersion, Origin, Zone},
};

fn input() -> CreateDeck {
    CreateDeck {
        name: "Boros Equipment".into(),
        format: "standard".into(),
        cards: vec![
            DeckCard {
                reference: "arena:106533".into(),
                quantity: 4,
                zone: Zone::Main,
            },
            DeckCard {
                reference: "oracle:equipment".into(),
                quantity: 4,
                zone: Zone::Main,
            },
        ],
    }
}

#[tokio::test]
async fn retries_return_one_immutable_version_and_edits_append_versions() {
    let db = Engine::memory(1024 * 1024).unwrap();
    let decks = Decks::new(db.clone());
    let first = decks
        .create("create-1", input(), Origin::User)
        .await
        .unwrap();
    assert_eq!(first.ordinal, 1);
    assert_eq!(first.cards.len(), 2);
    let retry = decks
        .create("create-1", input(), Origin::User)
        .await
        .unwrap();
    assert_eq!(first, retry);
    let mut conflict = input();
    conflict.name = "Different".into();
    assert!(
        decks
            .create("create-1", conflict, Origin::User)
            .await
            .is_err()
    );
    let mut cards = first.cards.clone();
    cards[0].quantity = 3;
    let change = NewVersion {
        deck_id: first.deck_id.clone(),
        parent_version: Some(first.id.clone()),
        name: first.name.clone(),
        format: first.format.clone(),
        cards,
    };
    let next = decks
        .create_version("version-2", change, Origin::Agent)
        .await
        .unwrap();
    assert_eq!(next.ordinal, 2);
    assert_eq!(next.origin, Origin::Agent);
    assert_eq!(next.parent_version, Some(first.id.clone()));
    assert_eq!(decks.version(&first.id).await.unwrap(), Some(first));
    let before = db.metrics().await.unwrap().statements;
    let list = decks.list(10, None).await.unwrap();
    assert_eq!(db.metrics().await.unwrap().statements - before, 1);
    assert_eq!(list.items.len(), 1);
    assert_eq!(list.items[0].latest_version, next.id);
    assert_eq!(list.items[0].version_count, 2);
}

#[tokio::test]
async fn normalized_card_order_and_duplicate_lines_are_idempotent() {
    let decks = Decks::new(Engine::memory(1024 * 1024).unwrap());
    let mut first = input();
    first.cards[0].quantity = 2;
    first.cards.push(first.cards[0].clone());
    let version = decks
        .create("canonical", first, Origin::User)
        .await
        .unwrap();
    let mut reordered = input();
    reordered.cards.reverse();
    let again = decks
        .create("canonical", reordered, Origin::User)
        .await
        .unwrap();
    assert_eq!(version, again);
}

#[tokio::test]
async fn invalid_parent_or_deck_cannot_leave_partial_creations() {
    let decks = Decks::new(Engine::memory(1024 * 1024).unwrap());
    let first = decks.create("first", input(), Origin::User).await.unwrap();
    let other = decks.create("second", input(), Origin::User).await.unwrap();
    let change = NewVersion {
        deck_id: other.deck_id.clone(),
        parent_version: Some(first.id.clone()),
        name: "Invalid".into(),
        format: "standard".into(),
        cards: input().cards,
    };
    assert!(
        decks
            .create_version("bad-parent", change, Origin::Agent)
            .await
            .is_err()
    );
    assert_eq!(decks.list(10, None).await.unwrap().items.len(), 2);
    assert!(decks.version("missing").await.unwrap().is_none());
}

#[tokio::test]
async fn concurrent_retries_share_one_creation() {
    let decks = Decks::new(Engine::memory(1024 * 1024).unwrap());
    let other = decks.clone();
    let (first, second) = tokio::join!(
        decks.create("racing", input(), Origin::User),
        other.create("racing", input(), Origin::User)
    );
    assert_eq!(first.unwrap(), second.unwrap());
    assert_eq!(decks.list(10, None).await.unwrap().items.len(), 1);
}
