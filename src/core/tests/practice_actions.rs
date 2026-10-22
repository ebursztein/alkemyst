mod practice_support;
use alkemyst_core::{
    db::Engine,
    domain::Origin,
    practice::{NewPractice, PracticeAction, PracticePhase, PracticeQuery, PracticeZone},
};

#[tokio::test]
async fn basic_turns_track_land_drops_taps_visible_zones_and_first_draw() {
    let (p, version) = practice_support::setup(Engine::memory(4 * 1024 * 1024).unwrap()).await;
    let initial = p
        .create(
            "actions",
            NewPractice {
                version_id: version,
                on_the_play: false,
                seed: Some("0000000000000001".into()),
            },
            Origin::User,
        )
        .await
        .unwrap();
    assert!(
        p.act(
            "early-draw",
            &initial.id,
            0,
            PracticeAction::Draw,
            Origin::User
        )
        .await
        .is_err()
    );
    let mut current = p
        .act(
            "keep",
            &initial.id,
            0,
            PracticeAction::Keep { bottom: vec![] },
            Origin::User,
        )
        .await
        .unwrap();
    assert_eq!(current.hand.len(), 8);
    assert_eq!(current.library_count, 6);
    let land = current
        .hand
        .iter()
        .find(|c| c.name.as_deref() == Some("Plains"))
        .unwrap()
        .id;
    assert!(
        p.act(
            "land-as-spell",
            &initial.id,
            current.revision,
            PracticeAction::Play {
                card: land,
                face: 0,
                destination: PracticeZone::Battlefield
            },
            Origin::User
        )
        .await
        .is_err()
    );
    current = p
        .act(
            "land",
            &initial.id,
            current.revision,
            PracticeAction::PlayLand {
                card: land,
                face: 0,
            },
            Origin::User,
        )
        .await
        .unwrap();
    assert!(current.land_played);
    let second = current
        .hand
        .iter()
        .find(|c| c.name.as_deref() == Some("Plains"))
        .unwrap()
        .id;
    assert!(
        p.act(
            "second-land",
            &initial.id,
            current.revision,
            PracticeAction::PlayLand {
                card: second,
                face: 0
            },
            Origin::User
        )
        .await
        .is_err()
    );
    current = p
        .act(
            "tap",
            &initial.id,
            current.revision,
            PracticeAction::Tap {
                card: land,
                tapped: true,
            },
            Origin::Agent,
        )
        .await
        .unwrap();
    assert!(current.battlefield[0].tapped);
    assert_eq!(current.last_action_origin, Origin::Agent);
    current = p
        .act(
            "next",
            &initial.id,
            current.revision,
            PracticeAction::NextTurn,
            Origin::User,
        )
        .await
        .unwrap();
    assert_eq!(current.turn, 2);
    assert!(!current.land_played);
    assert!(!current.battlefield[0].tapped);
    let spell = current
        .hand
        .iter()
        .find(|c| c.name.as_deref() == Some("Forgeblade"))
        .unwrap()
        .id;
    current = p
        .act(
            "cast",
            &initial.id,
            current.revision,
            PracticeAction::Play {
                card: spell,
                face: 0,
                destination: PracticeZone::Battlefield,
            },
            Origin::User,
        )
        .await
        .unwrap();
    current = p
        .act(
            "grave",
            &initial.id,
            current.revision,
            PracticeAction::Move {
                card: spell,
                destination: PracticeZone::Graveyard,
            },
            Origin::User,
        )
        .await
        .unwrap();
    current = p
        .act(
            "exile",
            &initial.id,
            current.revision,
            PracticeAction::Move {
                card: spell,
                destination: PracticeZone::Exile,
            },
            Origin::User,
        )
        .await
        .unwrap();
    current = p
        .act(
            "return",
            &initial.id,
            current.revision,
            PracticeAction::Move {
                card: spell,
                destination: PracticeZone::Hand,
            },
            Origin::User,
        )
        .await
        .unwrap();
    current = p
        .act(
            "resolve",
            &initial.id,
            current.revision,
            PracticeAction::Play {
                card: spell,
                face: 0,
                destination: PracticeZone::Graveyard,
            },
            Origin::User,
        )
        .await
        .unwrap();
    assert_eq!(current.graveyard[0].id, spell);
    current = p
        .act(
            "manual-battlefield",
            &initial.id,
            current.revision,
            PracticeAction::Move {
                card: spell,
                destination: PracticeZone::Battlefield,
            },
            Origin::User,
        )
        .await
        .unwrap();
    assert_eq!(current.battlefield.len(), 2);
    let hidden = (0..14)
        .find(|id| {
            !current.hand.iter().any(|c| c.id == *id)
                && !current.battlefield.iter().any(|c| c.card.id == *id)
        })
        .unwrap();
    assert!(
        p.act(
            "hidden",
            &initial.id,
            current.revision,
            PracticeAction::Move {
                card: hidden,
                destination: PracticeZone::Hand
            },
            Origin::User
        )
        .await
        .is_err()
    );
    let finished = p
        .act(
            "finish",
            &initial.id,
            current.revision,
            PracticeAction::Finish,
            Origin::User,
        )
        .await
        .unwrap();
    assert_eq!(finished.phase, PracticePhase::Finished);
    assert!(
        p.act(
            "after-finish",
            &initial.id,
            finished.revision,
            PracticeAction::Draw,
            Origin::User
        )
        .await
        .is_err()
    );
    let page = p.list(PracticeQuery::default()).await.unwrap();
    assert_eq!(page.items[0].phase, PracticePhase::Finished);
    assert!(!page.items[0].decked);
}
