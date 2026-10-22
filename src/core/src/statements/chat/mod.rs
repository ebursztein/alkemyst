use crate::db::{Mutation, Statement};
const READS: &[&str] = &["chat_conversations", "chat_messages"];
pub const GET: Statement = Statement {
    name: "chat.get",
    sql: "SELECT data FROM chat_conversation_details WHERE id=?1 AND deleted=0",
    reads: READS,
    cacheable: true,
};
pub const RECEIPT: Statement = Statement {
    name: "chat.creationReceipt",
    sql: "SELECT json_object('fingerprint',fingerprint,'value',json(CASE deleted WHEN 0 THEN json_set(data,'$.revision',0) ELSE NULL END)) FROM chat_conversation_details WHERE create_key=?1",
    reads: READS,
    cacheable: true,
};
pub const MESSAGE_RECEIPT: Statement = Statement {
    name: "chat.messageReceipt",
    sql: "SELECT json_object('value',json(m.data)) FROM chat_conversations c LEFT JOIN chat_message_details m ON m.conversation_id=c.id AND m.idempotency_key=?2 WHERE c.id=?1 AND c.deleted=0",
    reads: READS,
    cacheable: true,
};
pub const CREATE: Mutation = Mutation {
    sql: "INSERT INTO chat_conversations(id,create_key,fingerprint,created_at,title,version_id,share_deck,share_notes,share_history)
        SELECT ?1,?2,?3,unixepoch(),?4,?5,?6,?7,?8
        WHERE ?5 IS NULL OR EXISTS(SELECT 1 FROM deck_versions WHERE id=?5 AND sealed=1)
        ON CONFLICT(create_key) DO NOTHING",
    writes: &["chat_conversations"],
};
pub const APPEND_USER: Mutation = Mutation {
    sql:
        "INSERT INTO chat_messages(id,conversation_id,sequence,idempotency_key,role,body,created_at)
        SELECT ?1,c.id,?2+1,?3,'user',?4,unixepoch() FROM chat_conversations c
        WHERE c.id=?5 AND c.deleted=0 AND ?2<2000
            AND coalesce((SELECT max(sequence) FROM chat_messages WHERE conversation_id=c.id),0)=?2
        ON CONFLICT(conversation_id,idempotency_key) DO NOTHING",
    writes: &["chat_messages"],
};
pub const CLEAR_MESSAGES: Mutation = Mutation {
    sql: "DELETE FROM chat_messages WHERE conversation_id=?1",
    writes: &["chat_messages"],
};
pub const DELETE: Mutation = Mutation {
    sql: "UPDATE chat_conversations SET deleted=1,title=NULL,version_id=NULL,share_deck=0,share_notes=0,share_history=0 WHERE id=?1 AND deleted=0",
    writes: &["chat_conversations"],
};
pub const LIST: Statement = Statement {
    name: "chat.list",
    sql: include_str!("list.sql"),
    reads: READS,
    cacheable: true,
};
pub const MESSAGES: Statement = Statement {
    name: "chat.messages",
    sql: include_str!("messages.sql"),
    reads: READS,
    cacheable: true,
};
