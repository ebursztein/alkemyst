use alkemyst_core::{
    Application,
    chat::{ChatQuery, NewConversation, NewMessage},
    db::{Engine, Mutation, Statement, Write},
    domain::{CreateDeck, NewVersion, Origin},
};
use serde_json::json;

#[tokio::test]
async fn creation_retries_preserve_the_versioned_fingerprint_contract() {
    let db = Engine::memory(4096).unwrap();
    // SHA-256 of the frozen v1 tuple [1,"Equipment questions",null,false,false,false].
    db.write(vec![Write {
        statement: Mutation {
            sql: "INSERT INTO chat_conversations(id,create_key,fingerprint,created_at,title,share_deck,share_notes,share_history)
                VALUES ('fixture-id','fixture-key','03d88186488f2ccfb781ee83162d02c40fb5af69b421624ffdee8fc94487c9d8',1,'Equipment questions',0,0,0)",
            writes: &["chat_conversations"],
        },
        params: vec![], expected_rows: Some(1),
    }]).await.unwrap();
    let result = Application::new(db.clone())
        .chat
        .create(
            "fixture-key",
            NewConversation {
                title: "Equipment questions".into(),
                version_id: None,
                share_deck: false,
                share_notes: false,
                share_history: false,
            },
        )
        .await
        .unwrap();
    assert_eq!(result.id, "fixture-id");
    assert_eq!(result.created_at, 1);
    db.close().await.unwrap();
}

#[tokio::test]
async fn context_pins_immutable_versions_and_deletion_removes_only_chat_content() {
    let db = Engine::memory(1024 * 1024).unwrap();
    let app = Application::new(db.clone());
    let first = app
        .decks
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
    let mut input = NewConversation {
        title: "Private local title".into(),
        version_id: Some("missing".into()),
        share_deck: true,
        share_notes: true,
        share_history: true,
    };
    assert!(app.chat.create("chat", input.clone()).await.is_err());
    input.version_id = Some(first.id.clone());
    let conversation = app.chat.create("chat", input.clone()).await.unwrap();
    let second = app
        .decks
        .create_version(
            "edit",
            NewVersion {
                deck_id: first.deck_id.clone(),
                parent_version: Some(first.id.clone()),
                name: "Boros revised".into(),
                format: "standard".into(),
                cards: vec![],
            },
            Origin::Agent,
        )
        .await
        .unwrap();
    assert_eq!(
        app.chat
            .get(&conversation.id)
            .await
            .unwrap()
            .unwrap()
            .version_id,
        Some(first.id.clone())
    );
    let private_text = "Private prompt retained only in this conversation";
    let oversized = db
        .write(vec![Write {
            statement: Mutation {
                sql: "INSERT INTO chat_messages VALUES ('oversized',?1,1,'oversized','user',?2,1)",
                writes: &["chat_messages"],
            },
            params: vec![json!(conversation.id), json!("😀".repeat(5000))],
            expected_rows: Some(1),
        }])
        .await;
    assert!(oversized.is_err());
    assert_eq!(
        app.chat
            .get(&conversation.id)
            .await
            .unwrap()
            .unwrap()
            .revision,
        0
    );
    app.chat
        .append_user(
            &conversation.id,
            "message",
            NewMessage {
                body: private_text.into(),
                expected_revision: 0,
            },
        )
        .await
        .unwrap();
    assert_eq!(app.chat.create("chat", input).await.unwrap(), conversation);
    for sql in [
        "UPDATE chat_conversations SET title='changed' WHERE id=?1",
        "UPDATE chat_messages SET body='changed' WHERE conversation_id=?1",
    ] {
        assert!(
            db.write(vec![Write {
                statement: Mutation {
                    sql,
                    writes: &["chat_conversations", "chat_messages"]
                },
                params: vec![json!(conversation.id)],
                expected_rows: Some(1)
            }])
            .await
            .is_err()
        );
    }
    app.chat.delete(&conversation.id).await.unwrap();
    let cleared=db.read(Statement {
        name:"test.chat.cleared",
        sql:"SELECT json_object('title',c.title,'version_id',c.version_id,'deleted',c.deleted,
            'messages',(SELECT count(*) FROM chat_messages WHERE conversation_id=c.id),
            'prompt_receipts',(SELECT count(*) FROM mutation_receipts WHERE instr(body,?2)>0))
            FROM chat_conversations c WHERE c.id=?1",
        reads:&["chat_conversations","chat_messages","mutation_receipts"],cacheable:false,
    },vec![json!(conversation.id),json!(private_text)]).await.unwrap();
    assert_eq!(
        cleared[0],
        json!({"title":null,"version_id":null,"deleted":1,"messages":0,"prompt_receipts":0})
    );
    assert_eq!(app.decks.version(&first.id).await.unwrap().unwrap(), first);
    assert_eq!(
        app.decks.version(&second.id).await.unwrap().unwrap(),
        second
    );
    assert!(
        app.chat
            .list(ChatQuery::default())
            .await
            .unwrap()
            .items
            .is_empty()
    );
    db.close().await.unwrap();
}
