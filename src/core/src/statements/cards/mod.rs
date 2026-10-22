use crate::db::{Mutation, Statement};

pub const STATUS: Statement = Statement {
    name: "cards.status",
    sql: include_str!("status.sql"),
    reads: &[
        "catalog_state",
        "catalog_snapshots",
        "cards",
        "card_printings",
    ],
    cacheable: true,
};
pub const SEARCH: Statement = Statement {
    name: "cards.search",
    sql: include_str!("search.sql"),
    reads: &["catalog_state", "cards", "cards_fts"],
    cacheable: true,
};
pub const DETAIL: Statement = Statement {
    name: "cards.detail",
    sql: "SELECT c.data FROM cards c JOIN catalog_state s ON c.snapshot_id=s.active_snapshot WHERE c.oracle_id=?1",
    reads: &["cards", "catalog_state"],
    cacheable: true,
};
pub const RESOLVE: Statement = Statement {
    name: "cards.resolveArena",
    sql: include_str!("resolve.sql"),
    reads: &["card_printings", "catalog_state", "arena_card_aliases"],
    cacheable: true,
};
pub const BEGIN: Mutation = Mutation {
    sql: "INSERT INTO catalog_snapshots VALUES (?1, ?2, ?3, 'staging')",
    writes: &["catalog_snapshots"],
};
pub const STAGE: Mutation = Mutation {
    sql: include_str!("stage.sql"),
    writes: &["card_printings"],
};
pub const BUILD: Mutation = Mutation {
    sql: include_str!("build.sql"),
    writes: &["cards", "cards_fts"],
};
pub const RETIRE: Mutation = Mutation {
    sql: "UPDATE catalog_snapshots SET state='retired' WHERE state='active'",
    writes: &["catalog_snapshots"],
};
pub const ACTIVATE: Mutation = Mutation {
    sql: "UPDATE catalog_snapshots SET state='active' WHERE id=?1 AND state='staging' AND EXISTS(SELECT 1 FROM cards WHERE snapshot_id=?1)",
    writes: &["catalog_snapshots"],
};
pub const PUBLISH: Mutation = Mutation {
    sql: "INSERT INTO catalog_state VALUES (1, ?1) ON CONFLICT(singleton) DO UPDATE SET active_snapshot=excluded.active_snapshot",
    writes: &["catalog_state"],
};
pub const DISCARD: Mutation = Mutation {
    sql: "DELETE FROM catalog_snapshots WHERE id=?1 AND state='staging'",
    writes: &["catalog_snapshots", "cards", "cards_fts", "card_printings"],
};
pub const CLEAN: Mutation = Mutation {
    sql: "DELETE FROM catalog_snapshots WHERE state!='active' AND id!=?1",
    writes: &["catalog_snapshots", "cards", "cards_fts", "card_printings"],
};

pub const ARENA_SOURCE: Statement = Statement {
    name: "cards.arenaAsset",
    sql: include_str!("arena_source.sql"),
    reads: &[],
    cacheable: false,
};
pub const ALIASES: Mutation = Mutation {
    sql: "INSERT INTO arena_card_aliases(arena_id, name, set_code, collector_number)
        SELECT json_extract(value, '$.arena_id'), json_extract(value, '$.name'),
            json_extract(value, '$.set_code'), json_extract(value, '$.collector_number')
        FROM json_each(?1) WHERE 1
        ON CONFLICT(arena_id) DO UPDATE SET name=excluded.name,
            set_code=excluded.set_code, collector_number=excluded.collector_number",
    writes: &["arena_card_aliases"],
};
