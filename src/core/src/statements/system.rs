//! Internal storage maintenance statements, outside domain query accounting.
pub const SCHEMA_VERSION: &str = "PRAGMA user_version";
pub const CONFIGURE: &str =
    "PRAGMA foreign_keys=ON; PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;";
pub const INITIAL: &str = include_str!("../../migrations/001_initial.sql");

pub const REVISIONS: &str = "SELECT table_name, revision FROM table_revisions";
pub const BUMP_REVISION: &str = "INSERT INTO table_revisions(table_name, revision) VALUES (?1, 1)
    ON CONFLICT(table_name) DO UPDATE SET revision = revision + 1";

pub const CATALOG: &str = include_str!("../../migrations/002_catalog.sql");

pub const ARENA_ALIASES: &str = include_str!("../../migrations/003_arena_aliases.sql");

pub const DECKS: &str = include_str!("../../migrations/004_decks.sql");

pub const RECEIPT_ORIGINS: &str = include_str!("../../migrations/005_receipt_origins.sql");

pub const CAPTURE: &str = include_str!("../../migrations/006_capture.sql");

pub const HISTORY_ORDER: &str = include_str!("../../migrations/007_history_order.sql");
pub const HISTORY_FACTS: &str = include_str!("../../migrations/007_history_facts.sql");
pub const HISTORY_SUMMARIES: &str = include_str!("../../migrations/007_history_summaries.sql");

pub const HISTORY_MATCH_FACTS: &str = include_str!("../../migrations/007_history_match_facts.sql");
pub const HISTORY_MATCHES: &str = include_str!("../../migrations/007_history_matches.sql");
