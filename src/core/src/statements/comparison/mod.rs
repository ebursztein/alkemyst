use crate::db::Statement;
pub const COMPARE: Statement = Statement {
    name: "analysis.compare",
    sql: concat!(
        include_str!("head.sql"),
        include_str!("../cards/reference_from.sql"),
        include_str!("tail.sql")
    ),
    reads: super::analysis::SNAPSHOT.reads,
    cacheable: true,
};
