use alkemyst_core::{
    Application,
    chat::ChatQuery,
    db::{Engine, Mutation, Statement, Write},
};
use serde_json::json;

#[tokio::test]
async fn large_history_uses_indexed_cursor_pages_and_one_statement_per_read() {
    let db = Engine::memory(4 * 1024 * 1024).unwrap();
    let rows = (0..1000)
        .map(|i| json!({"id":format!("conversation-{i}")}))
        .collect::<Vec<_>>();
    db.write(vec![Write {
        statement:Mutation {
            sql:"INSERT INTO chat_conversations(id,create_key,fingerprint,created_at,title,share_deck,share_notes,share_history)
                SELECT json_extract(value,'$.id'),json_extract(value,'$.id'),printf('%064d',0),1,'Question',0,0,0 FROM json_each(?1)",
            writes:&["chat_conversations"],
        }, params:vec![json!(serde_json::to_string(&rows).unwrap())],expected_rows:Some(1000),
    }]).await.unwrap();
    for sequence in 1..=4 {
        db.write(vec![Write {
            statement:Mutation {
                sql:"INSERT INTO chat_messages SELECT id||'-'||?1,id,?1,id||'-'||?1,'user','Question',1 FROM chat_conversations",
                writes:&["chat_messages"],
            },params:vec![json!(sequence)],expected_rows:Some(1000),
        }]).await.unwrap();
    }
    let list = Statement {
        name: "test.chat.listPlan",
        sql: include_str!("../src/statements/chat/list.sql"),
        reads: &[],
        cacheable: false,
    };
    let plan = db
        .explain(list, vec![json!(null), json!(10)])
        .await
        .unwrap()
        .join("\n");
    assert!(plan.contains("chat_conversation_order"), "{plan}");
    let messages = Statement {
        name: "test.chat.messagesPlan",
        sql: include_str!("../src/statements/chat/messages.sql"),
        ..list
    };
    let plan = db
        .explain(messages, vec![json!("conversation-1"), json!(1), json!(10)])
        .await
        .unwrap()
        .join("\n");
    assert!(plan.contains("conversation_id=? AND sequence>?"), "{plan}");
    let app = Application::new(db.clone());
    let before = db.metrics().await.unwrap();
    let page = app
        .chat
        .list(ChatQuery {
            limit: 10,
            cursor: None,
        })
        .await
        .unwrap();
    assert_eq!(page.items.len(), 10);
    let next = app
        .chat
        .list(ChatQuery {
            limit: 10,
            cursor: page.next_cursor,
        })
        .await
        .unwrap();
    assert_eq!(next.items.len(), 10);
    assert!(
        page.items
            .iter()
            .all(|a| next.items.iter().all(|b| a.id != b.id))
    );
    assert_eq!(
        app.chat
            .messages("conversation-1", ChatQuery::default())
            .await
            .unwrap()
            .unwrap()
            .items
            .len(),
        4
    );
    assert_eq!(
        db.metrics().await.unwrap().statements,
        before.statements + 3
    );
    app.chat
        .messages("conversation-1", ChatQuery::default())
        .await
        .unwrap();
    assert_eq!(
        db.metrics().await.unwrap().cache_hits,
        before.cache_hits + 1
    );
    db.close().await.unwrap();
}
