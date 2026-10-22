use crate::db::{Mutation, Statement};
pub const CHECKPOINT: Statement = Statement {
    name: "capture.checkpoint",
    sql: "SELECT checkpoint FROM capture_sources WHERE id=?1",
    reads: &["capture_sources"],
    cacheable: true,
};
pub const SAVE_CHECKPOINT: Mutation = Mutation {
    sql: "INSERT INTO capture_sources VALUES (?1, ?3+1, json_set(?2, '$.sequence', ?3+1)) ON CONFLICT(id) DO UPDATE SET sequence=excluded.sequence, checkpoint=excluded.checkpoint WHERE capture_sources.sequence=?3",
    writes: &["capture_sources"],
};
pub const DECK: Statement = Statement {
    name: "capture.deck",
    sql: "SELECT json_object('deck_id', (SELECT deck_id FROM arena_decks WHERE alias=?1), 'version_id', (SELECT result_id FROM mutation_receipts WHERE key=?2))",
    reads: &["arena_decks", "mutation_receipts"],
    cacheable: true,
};
pub const ALIAS: Mutation = Mutation {
    sql: "INSERT INTO arena_decks VALUES (?1,?2) ON CONFLICT(alias) DO NOTHING",
    writes: &["arena_decks"],
};
pub const MATCHES: Mutation = Mutation {
    sql: "INSERT INTO matches SELECT DISTINCT json_extract(value,'$.match_id') FROM json_each(?1) WHERE json_extract(value,'$.match_id') IS NOT NULL ON CONFLICT(id) DO NOTHING",
    writes: &["matches"],
};
pub const GAMES: Mutation = Mutation {
    sql: include_str!("games.sql"),
    writes: &["games"],
};
pub const EVENTS: Mutation = Mutation {
    sql: include_str!("events.sql"),
    writes: &["capture_events"],
};
pub const STATUS: Statement = Statement {
    name: "capture.status",
    sql: include_str!("status.sql"),
    reads: &[
        "capture_sources",
        "matches",
        "games",
        "capture_events",
        "deck_versions",
    ],
    cacheable: true,
};
