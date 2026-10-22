//! Serialized SQLite access shared by every application adapter.
mod cache;
mod engine;
mod snapshot;
mod worker;

pub use engine::Engine;
use serde_json::Value;

#[derive(Clone, Copy, Debug)]
pub struct Statement {
    pub name: &'static str,
    pub sql: &'static str,
    pub reads: &'static [&'static str],
    pub cacheable: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Mutation {
    pub sql: &'static str,
    pub writes: &'static [&'static str],
}

#[derive(Debug)]
pub struct Write {
    pub statement: Mutation,
    pub params: Vec<Value>,
    pub expected_rows: Option<usize>,
}

#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Metrics {
    pub read_calls: u64,
    pub source_reads: u64,
    pub statements: u64,
    pub cache_hits: u64,
    pub failed_transactions: u64,
    pub cache_bytes: usize,
    pub query_microseconds: u64,
    pub slow_queries: u64,
    pub table_revisions: std::collections::BTreeMap<String, i64>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database worker stopped")]
    Closed,
    #[error("database error: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("invalid JSON result: {0}")]
    Json(#[from] serde_json::Error),
    #[error("parameters must be JSON scalar values within SQLite numeric bounds")]
    Binding,
    #[error("write affected {actual} rows; expected {expected}")]
    RowCount { expected: usize, actual: usize },
    #[error("database schema is newer than this application")]
    FutureSchema,
    #[error("read statement must be read-only")]
    NotReadOnly,
    #[error("cannot start database worker: {0}")]
    Startup(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;
