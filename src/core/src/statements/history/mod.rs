use crate::db::Statement;
const TABLES: &[&str] = &[
    "matches",
    "match_capture_order",
    "games",
    "game_capture_order",
    "capture_events",
    "deck_versions",
];
pub const GAMES: Statement = Statement {
    name: "history.games",
    sql: include_str!("games.sql"),
    reads: TABLES,
    cacheable: true,
};
pub const GAME: Statement = Statement {
    name: "history.game",
    sql: "SELECT summary FROM history_game_summaries WHERE id=?1",
    reads: TABLES,
    cacheable: true,
};
pub const TIMELINE: Statement = Statement {
    name: "history.timeline",
    sql: include_str!("timeline.sql"),
    reads: &["games", "capture_events"],
    cacheable: true,
};
pub const STATISTICS: Statement = Statement {
    name: "history.statistics",
    sql: include_str!("statistics.sql"),
    reads: TABLES,
    cacheable: true,
};

pub const MATCHES: Statement = Statement {
    name: "history.matches",
    sql: include_str!("matches.sql"),
    reads: TABLES,
    cacheable: true,
};
pub const MATCH: Statement = Statement {
    name: "history.match",
    sql: "SELECT summary FROM history_match_summaries WHERE id=?1",
    reads: TABLES,
    cacheable: true,
};

pub const TURNS: Statement = Statement {
    name: "history.turns",
    sql: include_str!("turns.sql"),
    reads: &["games", "capture_events"],
    cacheable: true,
};
