mod practice_support;
use alkemyst_core::{
    db::Engine,
    domain::Origin,
    practice::{NewPractice, PracticeAction, PracticePhase},
};
#[tokio::test]
async fn london_practice_keeps_physical_copies_and_ordered_bottoms_through_drawing() {
    let db = Engine::memory(4 * 1024 * 1024).unwrap();
    let (p, version) = practice_support::setup(db.clone()).await;
    let input = NewPractice {
        version_id: version,
        on_the_play: true,
        seed: Some("ffffffffffffffff".into()),
    };
    let first = p
        .create("practice-once", input.clone(), Origin::User)
        .await
        .unwrap();
    assert_eq!(first.hand.len(), 7);
    assert_eq!(first.library_count, 7);
    assert_eq!(first.phase, PracticePhase::Opening);
    assert_eq!(
        p.create("practice-once", input.clone(), Origin::Agent)
            .await
            .unwrap(),
        first
    );
    let again = p
        .create("practice-replay", input, Origin::User)
        .await
        .unwrap();
    assert_eq!(first.hand, again.hand);
    let one = p
        .act(
            "mull-1",
            &first.id,
            0,
            PracticeAction::Mulligan,
            Origin::User,
        )
        .await
        .unwrap();
    let two = p
        .act(
            "mull-2",
            &first.id,
            1,
            PracticeAction::Mulligan,
            Origin::User,
        )
        .await
        .unwrap();
    assert_eq!(one.hand.len(), 7);
    assert_eq!(two.bottom_count, 2);
    let bottom = vec![two.hand[2].id, two.hand[0].id];
    let kept = p
        .act(
            "keep",
            &first.id,
            2,
            PracticeAction::Keep {
                bottom: bottom.clone(),
            },
            Origin::User,
        )
        .await
        .unwrap();
    assert_eq!(kept.turn, 1);
    assert_eq!(kept.hand.len(), 5);
    assert_eq!(kept.library_count, 9);
    assert_eq!(
        p.act(
            "keep",
            &first.id,
            2,
            PracticeAction::Keep {
                bottom: bottom.clone()
            },
            Origin::Agent
        )
        .await
        .unwrap(),
        kept
    );
    let mut seen = kept.hand.iter().map(|c| c.id).collect::<Vec<_>>();
    let mut drawn = vec![];
    let mut current = kept;
    while current.library_count > 0 {
        let old = current
            .hand
            .iter()
            .map(|c| c.id)
            .collect::<std::collections::BTreeSet<_>>();
        current = p
            .act(
                &format!("draw-{}", current.revision),
                &first.id,
                current.revision,
                PracticeAction::Draw,
                Origin::User,
            )
            .await
            .unwrap();
        let id = current
            .hand
            .iter()
            .find(|c| !old.contains(&c.id))
            .unwrap()
            .id;
        drawn.push(id);
        seen.push(id);
    }
    assert_eq!(&drawn[drawn.len() - 2..], bottom.as_slice());
    seen.sort_unstable();
    assert_eq!(seen, (0..14).collect::<Vec<_>>());
    let decked = p
        .act(
            "decked",
            &first.id,
            current.revision,
            PracticeAction::Draw,
            Origin::User,
        )
        .await
        .unwrap();
    assert!(decked.decked);
    assert_eq!(decked.phase, PracticePhase::Finished);
    let before = db.metrics().await.unwrap();
    assert_eq!(p.get(&first.id).await.unwrap().unwrap(), decked);
    let warm = db.metrics().await.unwrap();
    assert!(warm.statements <= before.statements + 1);
    assert_eq!(p.get(&first.id).await.unwrap().unwrap(), decked);
    assert_eq!(db.metrics().await.unwrap().statements, warm.statements);
}
