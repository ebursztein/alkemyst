mod support;
use alkemyst_core::{
    catalog::Catalog,
    db::Engine,
    domain::{CreateDeck, DeckCard, Decks, Origin, Zone},
    practice::{NewPractice, Practice, PracticeAction},
};
#[tokio::test]
async fn manual_land_play_selects_the_land_face_of_a_double_faced_card() {
    let db = Engine::memory(4 * 1024 * 1024).unwrap();
    Catalog::new(db.clone())
        .import_jsonl(std::io::Cursor::new(support::cards()), support::source())
        .await
        .unwrap();
    let v = Decks::new(db.clone())
        .create(
            "modal-practice",
            CreateDeck {
                name: "Modal practice".into(),
                format: "test".into(),
                cards: vec![DeckCard {
                    reference: "oracle:oracle-modal".into(),
                    quantity: 7,
                    zone: Zone::Main,
                }],
            },
            Origin::User,
        )
        .await
        .unwrap();
    let p = Practice::new(db);
    let first = p
        .create(
            "modal-session",
            NewPractice {
                version_id: v.id,
                on_the_play: true,
                seed: Some("0000000000000001".into()),
            },
            Origin::User,
        )
        .await
        .unwrap();
    let card = first.hand[0].id;
    assert_eq!(first.hand[0].faces[0].type_line.as_deref(), Some("Sorcery"));
    assert_eq!(first.hand[0].faces[1].type_line.as_deref(), Some("Land"));
    let kept = p
        .act(
            "keep-modal",
            &first.id,
            0,
            PracticeAction::Keep { bottom: vec![] },
            Origin::User,
        )
        .await
        .unwrap();
    assert!(
        p.act(
            "wrong-face",
            &first.id,
            kept.revision,
            PracticeAction::PlayLand { card, face: 0 },
            Origin::User
        )
        .await
        .is_err()
    );
    let played = p
        .act(
            "land-face",
            &first.id,
            kept.revision,
            PracticeAction::PlayLand { card, face: 1 },
            Origin::User,
        )
        .await
        .unwrap();
    assert_eq!(played.battlefield[0].face, 1);
    assert!(played.land_played);
}
