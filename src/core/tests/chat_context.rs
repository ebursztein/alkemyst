use alkemyst_core::{
    Application,
    chat::{ChatQuery, NewConversation, NewMessage},
    db::{Engine, Mutation, Statement, Write},
    domain::{CreateDeck, NewVersion, Origin},
};
use serde_json::json;

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
