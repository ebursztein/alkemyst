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
async fn history_is_ordered_idempotent_cached_and_deleted_without_resurrection() {
    let db = Engine::memory(1024 * 1024).unwrap();
    let app = Application::new(db.clone());
    let input = conversation("Equipment questions");
    let created = app
        .chat
        .create("conversation", input.clone())
        .await
        .unwrap();
    assert_eq!(
        app.chat.create("conversation", input).await.unwrap(),
        created
    );
    assert!(
        app.chat
            .create("conversation", conversation("Different"))
            .await
            .is_err()
    );
    assert_eq!(created.revision, 0);
    let empty = app
        .chat
        .messages(&created.id, ChatQuery::default())
        .await
        .unwrap()
        .unwrap();
    assert!(empty.items.is_empty());
    let before = db.metrics().await.unwrap();
    app.chat
        .messages(&created.id, ChatQuery::default())
        .await
        .unwrap();
    assert_eq!(
        db.metrics().await.unwrap().cache_hits,
        before.cache_hits + 1
    );
    let input = NewMessage {
        body: "How many equipment?".into(),
        expected_revision: 0,
    };
    let first = app
        .chat
        .append_user(&created.id, "message", input.clone())
        .await
        .unwrap();
    assert_eq!(
        app.chat
            .append_user(&created.id, "message", input)
            .await
            .unwrap(),
        first
    );
    assert!(
        app.chat
            .append_user(
                &created.id,
                "message",
                NewMessage {
                    body: "Changed".into(),
                    expected_revision: 0,
                }
            )
            .await
            .is_err()
    );
    app.chat
        .append_user(
            &created.id,
            "second",
            NewMessage {
                body: "With tutors?".into(),
                expected_revision: 1,
            },
        )
        .await
        .unwrap();
    let page = app
        .chat
        .messages(
            &created.id,
            ChatQuery {
                limit: 1,
                cursor: None,
            },
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(page.items, vec![first]);
    let next = app
        .chat
        .messages(
            &created.id,
            ChatQuery {
                limit: 1,
                cursor: page.next_cursor,
            },
        )
        .await
        .unwrap()
        .unwrap();
    assert_eq!(next.items[0].body, "With tutors?");
    assert!(next.next_cursor.is_none());
    assert_eq!(
        app.chat.get(&created.id).await.unwrap().unwrap().revision,
        2
    );
    assert_eq!(
        app.chat
            .list(ChatQuery::default())
            .await
            .unwrap()
            .items
            .len(),
        1
    );
    app.chat.delete(&created.id).await.unwrap();
    app.chat.delete(&created.id).await.unwrap();
    assert!(app.chat.get(&created.id).await.unwrap().is_none());
    assert!(
        app.chat
            .messages(&created.id, ChatQuery::default())
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        app.chat
            .list(ChatQuery::default())
            .await
            .unwrap()
            .items
            .is_empty()
    );
    assert!(
        app.chat
            .create("conversation", conversation("Equipment questions"))
            .await
            .is_err()
    );
    assert!(
        app.chat
            .append_user(
                &created.id,
                "late",
                NewMessage {
                    body: "Delayed retry".into(),
                    expected_revision: 0,
                }
            )
            .await
            .is_err()
    );
    db.close().await.unwrap();
}

#[tokio::test]
async fn competing_appends_have_one_winner_and_history_survives_reopen_and_backup() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("chat.sqlite3");
    let backup = dir.path().join("backup.sqlite3");
    let db = Engine::open(&path, 4096).unwrap();
    let app = Application::new(db.clone());
    let c = app
        .chat
        .create("create", conversation("Local"))
        .await
        .unwrap();
    let left = app.chat.append_user(
        &c.id,
        "left",
        NewMessage {
            body: "Left".into(),
            expected_revision: 0,
        },
    );
    let right = app.chat.append_user(
        &c.id,
        "right",
        NewMessage {
            body: "Right".into(),
            expected_revision: 0,
        },
    );
    let (left, right) = tokio::join!(left, right);
    assert_ne!(left.is_ok(), right.is_ok());
    let messages = app
        .chat
        .messages(&c.id, ChatQuery::default())
        .await
        .unwrap();
    db.backup_to(&backup).await.unwrap();
    db.close().await.unwrap();
    let reopened = Engine::open(&path, 4096).unwrap();
    assert_eq!(
        Application::new(reopened.clone())
            .chat
            .messages(&c.id, ChatQuery::default())
            .await
            .unwrap(),
        messages
    );
    let restored = Engine::memory(4096).unwrap();
    restored.restore_from(&backup).await.unwrap();
    assert_eq!(
        Application::new(restored.clone())
            .chat
            .messages(&c.id, ChatQuery::default())
            .await
            .unwrap(),
        messages
    );
    reopened.close().await.unwrap();
    restored.close().await.unwrap();
}
