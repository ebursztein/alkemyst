mod practice_support;
use alkemyst_core::{
    db::Engine,
    domain::Origin,
    practice::{NewPractice, PracticeQuery},
};
#[tokio::test]
async fn simultaneous_creation_retries_never_leave_orphan_sessions() {
    let (p, version) = practice_support::setup(Engine::memory(8 * 1024 * 1024).unwrap()).await;
    for i in 0..8 {
        let input = NewPractice {
            version_id: version.clone(),
            on_the_play: true,
            seed: None,
        };
        let key = format!("concurrent-create-{i}");
        let (a, b) = tokio::join!(
            p.create(&key, input.clone(), Origin::User),
            p.create(&key, input, Origin::Agent)
        );
        assert_eq!(a.unwrap(), b.unwrap());
    }
    let page = p
        .list(PracticeQuery {
            limit: 100,
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(
        page.items.len(),
        8,
        "each idempotency key must own exactly one session"
    );
}
