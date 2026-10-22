mod practice_support;
use alkemyst_core::{
    db::Engine,
    domain::Origin,
    practice::{NewPractice, PracticeAction, PracticeQuery, PracticeZone},
};

#[tokio::test]
async fn invalid_requests_leave_state_and_idempotency_keys_reusable() {
    let (p, version) = practice_support::setup(Engine::memory(4 * 1024 * 1024).unwrap()).await;
    let input = NewPractice {
        version_id: version,
        on_the_play: true,
        seed: Some("0000000000000001".into()),
    };
    for key in ["", "\n"] {
        assert!(p.create(key, input.clone(), Origin::User).await.is_err());
    }
    for seed in ["", "1", "zzzzzzzzzzzzzzzz"] {
        assert!(
            p.create(
                "bad-seed",
                NewPractice {
                    seed: Some(seed.into()),
                    ..input.clone()
                },
                Origin::User
            )
            .await
            .is_err()
        );
    }
    assert!(
        p.create(
            "bad-id",
            NewPractice {
                version_id: "".into(),
                ..input.clone()
            },
            Origin::User
        )
        .await
        .is_err()
    );
    assert!(
        p.create(
            "missing",
            NewPractice {
                version_id: "missing".into(),
                ..input.clone()
            },
            Origin::User
        )
        .await
        .is_err()
    );
    let first = p.create("valid", input, Origin::User).await.unwrap();
    assert!(p.get("").await.is_err());
    assert!(p.inputs("").await.is_err());
    assert!(p.get("missing").await.unwrap().is_none());
    assert!(p.inputs("missing").await.unwrap().is_none());
    assert!(
        p.act(
            "missing",
            "missing",
            0,
            PracticeAction::Finish,
            Origin::User
        )
        .await
        .is_err()
    );
    assert!(
        p.act(
            "keep",
            &first.id,
            0,
            PracticeAction::Keep {
                bottom: vec![first.hand[0].id]
            },
            Origin::User
        )
        .await
        .is_err()
    );
    let mut current = p
        .act(
            "keep",
            &first.id,
            0,
            PracticeAction::Keep { bottom: vec![] },
            Origin::User,
        )
        .await
        .unwrap();
    let id = current
        .hand
        .iter()
        .find(|c| c.name.as_deref() == Some("Forgeblade"))
        .unwrap()
        .id;
    for action in [
        PracticeAction::Mulligan,
        PracticeAction::Keep { bottom: vec![] },
        PracticeAction::Play {
            card: id,
            face: 9,
            destination: PracticeZone::Battlefield,
        },
        PracticeAction::Play {
            card: id,
            face: 0,
            destination: PracticeZone::Hand,
        },
        PracticeAction::Play {
            card: 999,
            face: 0,
            destination: PracticeZone::Battlefield,
        },
        PracticeAction::PlayLand { card: id, face: 0 },
        PracticeAction::Tap {
            card: id,
            tapped: true,
        },
    ] {
        assert!(
            p.act(
                "invalid-action",
                &first.id,
                current.revision,
                action,
                Origin::User
            )
            .await
            .is_err()
        );
    }
    assert_eq!(p.get(&first.id).await.unwrap().unwrap(), current);
    current = p
        .act(
            "invalid-action",
            &first.id,
            current.revision,
            PracticeAction::Finish,
            Origin::User,
        )
        .await
        .unwrap();
    assert_eq!(p.get(&first.id).await.unwrap().unwrap(), current);
    for q in [
        PracticeQuery {
            limit: 0,
            ..Default::default()
        },
        PracticeQuery {
            limit: 101,
            ..Default::default()
        },
        PracticeQuery {
            version_id: Some("".into()),
            ..Default::default()
        },
    ] {
        assert!(p.list(q).await.is_err());
    }
    for cursor in [
        "bad",
        "p1:0000000000000000",
        "p1:ffffffffffffffff",
        "p1:+000000000000001",
    ] {
        assert!(
            p.list(PracticeQuery {
                cursor: Some(cursor.into()),
                ..Default::default()
            })
            .await
            .is_err()
        );
    }
}
