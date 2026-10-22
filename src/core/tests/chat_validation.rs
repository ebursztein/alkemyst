use alkemyst_core::{
    Application,
    chat::{ChatQuery, NewConversation, NewMessage},
    db::Engine,
};

fn conversation(title: &str) -> NewConversation {
    NewConversation {
        title: title.into(),
        version_id: None,
        share_deck: false,
        share_notes: false,
        share_history: false,
    }
}

#[tokio::test]
async fn invalid_inputs_do_not_consume_keys_or_change_conversations() {
    let app = Application::new(Engine::memory(4096).unwrap());
    for title in ["", " ", &"x".repeat(201)] {
        assert!(app.chat.create("valid", conversation(title)).await.is_err());
    }
    for key in ["", "bad\nkey", &"x".repeat(129)] {
        assert!(app.chat.create(key, conversation("Valid")).await.is_err());
    }
    let mut unshared = conversation("Pinned");
    unshared.share_deck = true;
    assert!(app.chat.create("valid", unshared).await.is_err());
    let created = app
        .chat
        .create("valid", conversation("Valid"))
        .await
        .unwrap();
    for body in ["", " ", &"x".repeat(16385)] {
        assert!(
            app.chat
                .append_user(
                    &created.id,
                    "valid",
                    NewMessage {
                        body: body.into(),
                        expected_revision: 0
                    }
                )
                .await
                .is_err()
        );
    }
    for query in [
        ChatQuery {
            limit: 0,
            cursor: None,
        },
        ChatQuery {
            limit: 101,
            cursor: None,
        },
        ChatQuery {
            limit: 1,
            cursor: Some("bogus".into()),
        },
    ] {
        assert!(app.chat.list(query.clone()).await.is_err());
        assert!(app.chat.messages(&created.id, query).await.is_err());
    }
    assert!(app.chat.get("").await.is_err());
    assert!(app.chat.delete("").await.is_err());
    assert!(app.chat.get("missing").await.unwrap().is_none());
    assert!(
        app.chat
            .messages("missing", ChatQuery::default())
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        app.chat
            .append_user(
                "missing",
                "valid",
                NewMessage {
                    body: "Question".into(),
                    expected_revision: 0
                }
            )
            .await
            .is_err()
    );
    assert_eq!(
        app.chat.get(&created.id).await.unwrap().unwrap().revision,
        0
    );
}
