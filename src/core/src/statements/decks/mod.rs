use crate::db::{Mutation, Statement};

pub const RECEIPT: Mutation = Mutation {
    sql: "INSERT INTO mutation_receipts(key, operation, body, result_id) VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(key) DO UPDATE SET key=excluded.key
        WHERE operation=excluded.operation AND body=excluded.body",
    writes: &["mutation_receipts"],
};
pub const CREATE: Mutation = Mutation {
    sql: "INSERT INTO decks(id, name, format, created_at) SELECT ?1, ?2, ?3, ?4
        WHERE EXISTS(SELECT 1 FROM mutation_receipts WHERE key=?5 AND result_id=?6)",
    writes: &["decks"],
};
pub const VERSION: Mutation = Mutation {
    sql: "INSERT INTO deck_versions(id, deck_id, ordinal, parent_version, name, format, origin, created_at)
        SELECT ?1, ?2, coalesce((SELECT max(ordinal)+1 FROM deck_versions WHERE deck_id=?2),1), ?3, ?4, ?5, ?6, ?7
        WHERE EXISTS(SELECT 1 FROM mutation_receipts WHERE key=?8 AND result_id=?1)",
    writes: &["deck_versions"],
};
pub const CARDS: Mutation = Mutation {
    sql: "INSERT INTO deck_cards(version_id, reference, quantity, zone)
        SELECT ?1, json_extract(value, '$.reference'), json_extract(value, '$.quantity'), json_extract(value, '$.zone')
        FROM json_each(?2) WHERE EXISTS(SELECT 1 FROM mutation_receipts WHERE key=?3 AND result_id=?1)",
    writes: &["deck_cards"],
};
pub const SEAL: Mutation = Mutation {
    sql: "UPDATE deck_versions SET sealed=1 WHERE id=?1 AND sealed=0",
    writes: &["deck_versions"],
};
pub const GET: Statement = Statement {
    name: "decks.version",
    sql: include_str!("version.sql"),
    reads: &["deck_versions", "deck_cards", "mutation_receipts"],
    cacheable: true,
};
pub const LIST: Statement = Statement {
    name: "decks.list",
    sql: include_str!("list.sql"),
    reads: &["decks", "deck_versions", "deck_cards"],
    cacheable: true,
};

pub const RECEIPT_GET: Statement = Statement {
    name: "decks.receipt",
    sql: "SELECT json_object('operation',operation,'body',body) FROM mutation_receipts WHERE key=?1",
    reads: &["mutation_receipts"],
    cacheable: true,
};
pub const RESOLVE_NAMES: Statement = Statement {
    name: "decks.resolveNames",
    sql: include_str!("resolve_names.sql"),
    reads: &["catalog_state", "cards", "card_printings"],
    cacheable: true,
};
pub const EXPORT: Statement = Statement {
    name: "decks.exportArena",
    sql: concat!(
        "WITH refs AS (SELECT reference, quantity, zone FROM deck_cards WHERE version_id=?1),
        resolved AS (SELECT r.reference, r.quantity, r.zone, p.data ",
        include_str!("../cards/reference_from.sql"),
        ") SELECT json_object('cards', json((SELECT json_group_array(json_object(
            'reference',reference,'quantity',quantity,'zone',zone,'name',json_extract(data,'$.name'),
            'set_code',json_extract(data,'$.set'),'collector_number',json_extract(data,'$.collector_number')))
            FROM resolved))) FROM deck_versions WHERE id=?1 AND sealed=1"
    ),
    reads: &[
        "deck_versions",
        "deck_cards",
        "catalog_state",
        "card_printings",
        "cards",
        "arena_card_aliases",
    ],
    cacheable: true,
};

pub const VERSIONS: Statement = Statement {
    name: "decks.versions",
    sql: include_str!("versions.sql"),
    reads: &["decks", "deck_versions", "deck_cards"],
    cacheable: true,
};
