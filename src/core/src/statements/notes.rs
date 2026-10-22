use crate::db::{Mutation, Statement};

pub const CREATE: Mutation = Mutation {
    sql: "INSERT INTO notes(id, deck_id, version_id, body, origin, created_at)
        SELECT ?1, ?2, ?3, ?4, ?5, ?6 WHERE EXISTS(
            SELECT 1 FROM mutation_receipts WHERE key=?7 AND result_id=?1)",
    writes: &["notes"],
};
pub const GET: Statement = Statement {
    name: "notes.getCreated",
    sql: "SELECT json_object('id',n.id,'deck_id',n.deck_id,'version_id',n.version_id,
        'body',n.body,'origin',n.origin,'created_at',n.created_at)
        FROM notes n JOIN mutation_receipts r ON r.result_id=n.id WHERE r.key=?1",
    reads: &["notes", "mutation_receipts"],
    cacheable: true,
};
pub const LIST: Statement = Statement {
    name: "notes.list",
    sql: "SELECT json_object('id',id,'deck_id',deck_id,'version_id',version_id,
        'body',body,'origin',origin,'created_at',created_at)
        FROM notes WHERE deck_id=?1 AND (?2 IS NULL OR id>?2) ORDER BY id LIMIT ?3",
    reads: &["notes"],
    cacheable: true,
};
