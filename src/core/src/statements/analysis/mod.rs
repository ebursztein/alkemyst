use crate::db::Statement;
pub const SNAPSHOT: Statement = Statement {
    name: "analysis.snapshot",
    sql: concat!(
        include_str!("input_head.sql"),
        include_str!("../cards/reference_from.sql"),
        include_str!("input_tail.sql")
    ),
    reads: &[
        "deck_versions",
        "deck_cards",
        "catalog_state",
        "catalog_snapshots",
        "cards",
        "card_printings",
        "arena_card_aliases",
    ],
    cacheable: true,
};
