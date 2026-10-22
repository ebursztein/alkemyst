use super::{Error, Metrics, Result, Statement, Write, cache::Cache};
use crate::statements::system;
use rusqlite::{Connection, params_from_iter, types::Value as Binding};
use serde_json::Value;
use std::{
    collections::HashSet,
    path::Path,
    time::{Duration, Instant},
};

pub(super) struct Worker {
    connection: Connection,
    cache: Cache,
    metrics: Metrics,
}

impl Worker {
    pub fn open(path: Option<&Path>, budget: usize) -> Result<Self> {
        let mut connection = match path {
            Some(path) => Connection::open(path)?,
            None => Connection::open_in_memory()?,
        };
        connection.busy_timeout(Duration::from_secs(5))?;
        connection.execute_batch(system::CONFIGURE)?;
        let version: i64 = connection.query_row(system::SCHEMA_VERSION, [], |row| row.get(0))?;
        if version > 7 {
            return Err(Error::FutureSchema);
        }
        if version == 0 {
            let transaction = connection.transaction()?;
            transaction.execute_batch(system::INITIAL)?;
            transaction.commit()?;
        }
        if version < 2 {
            let transaction = connection.transaction()?;
            transaction.execute_batch(system::CATALOG)?;
            transaction.commit()?;
        }
        if version < 3 {
            let transaction = connection.transaction()?;
            transaction.execute_batch(system::ARENA_ALIASES)?;
            transaction.commit()?;
        }
        if version < 4 {
            let transaction = connection.transaction()?;
            transaction.execute_batch(system::DECKS)?;
            transaction.commit()?;
        }
        if version < 5 {
            let transaction = connection.transaction()?;
            transaction.execute_batch(system::RECEIPT_ORIGINS)?;
            transaction.commit()?;
        }
        if version < 6 {
            let transaction = connection.transaction()?;
            transaction.execute_batch(system::CAPTURE)?;
            transaction.commit()?;
        }
        if version < 7 {
            let transaction = connection.transaction()?;
            transaction.execute_batch(system::HISTORY_ORDER)?;
            transaction.execute_batch(system::HISTORY_MATCH_FACTS)?;
            transaction.execute_batch(system::HISTORY_FACTS)?;
            transaction.execute_batch(system::HISTORY_SUMMARIES)?;
            transaction.execute_batch(system::HISTORY_MATCHES)?;
            transaction.commit()?;
        }
        let table_revisions = connection
            .prepare_cached(system::REVISIONS)?
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?
            .collect::<std::result::Result<_, _>>()?;
        Ok(Self {
            connection,
            cache: Cache::new(budget),
            metrics: Metrics {
                table_revisions,
                ..Metrics::default()
            },
        })
    }

    pub fn read(&mut self, query: Statement, params: Vec<Value>) -> Result<Vec<Value>> {
        self.metrics.read_calls += 1;
        let bindings = bindings(params.iter())?;
        let key = Cache::key(query, &params);
        if query.cacheable
            && let Some(rows) = self.cache.get(&key)
        {
            self.metrics.cache_hits += 1;
            return Ok(rows);
        }
        let start = Instant::now();
        let rows = query_rows(&self.connection, query, bindings);
        self.metrics.statements += 1;
        self.metrics.query_microseconds += start.elapsed().as_micros().min(u64::MAX as u128) as u64;
        if start.elapsed() >= Duration::from_millis(50) {
            self.metrics.slow_queries += 1;
        }
        let rows = rows?;
        if query.cacheable {
            self.cache.insert(key, query, &rows);
        }
        Ok(rows)
    }

    pub fn read_source(&mut self, path: &Path, statement: Statement) -> Result<Vec<Value>> {
        self.metrics.source_reads += 1;
        let connection = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        query_rows(&connection, statement, vec![])
    }

    pub fn write(&mut self, writes: Vec<Write>) -> Result<()> {
        let result = self.transaction(writes);
        match result {
            Ok(changed) => {
                self.cache.invalidate(&changed);
                for table in changed {
                    *self
                        .metrics
                        .table_revisions
                        .entry(table.to_string())
                        .or_default() += 1;
                }
                Ok(())
            }
            Err(error) => {
                self.metrics.failed_transactions += 1;
                Err(error)
            }
        }
    }

    fn transaction(&mut self, writes: Vec<Write>) -> Result<HashSet<&'static str>> {
        let transaction = self.connection.transaction()?;
        let mut changed = HashSet::new();
        for write in writes {
            let bindings = bindings(write.params.iter())?;
            let actual = transaction
                .prepare_cached(write.statement.sql)?
                .execute(params_from_iter(bindings))?;
            if let Some(expected) = write.expected_rows
                && actual != expected
            {
                return Err(Error::RowCount { expected, actual });
            }
            if actual > 0 {
                changed.extend(write.statement.writes);
            }
        }
        for table in &changed {
            transaction
                .prepare_cached(system::BUMP_REVISION)?
                .execute([table])?;
        }
        transaction.commit()?;
        Ok(changed)
    }

    pub fn explain(&mut self, query: Statement, params: Vec<Value>) -> Result<Vec<String>> {
        let sql = format!("EXPLAIN QUERY PLAN {}", query.sql);
        let bindings = bindings(params.iter())?;
        let mut prepared = self.connection.prepare_cached(&sql)?;
        let rows = prepared.query_map(params_from_iter(bindings), |row| row.get(3))?;
        Ok(rows.collect::<std::result::Result<Vec<String>, _>>()?)
    }

    pub fn metrics(&self) -> Metrics {
        Metrics {
            cache_bytes: self.cache.bytes,
            ..self.metrics.clone()
        }
    }
}

fn bindings<'a>(params: impl Iterator<Item = &'a Value>) -> Result<Vec<Binding>> {
    params
        .map(|value| match value {
            Value::Null => Ok(Binding::Null),
            Value::Bool(value) => Ok(Binding::Integer(i64::from(*value))),
            Value::String(value) => Ok(Binding::Text(value.clone())),
            Value::Number(value) => {
                if let Some(value) = value.as_i64() {
                    Ok(Binding::Integer(value))
                } else if value.is_f64() {
                    value.as_f64().map(Binding::Real).ok_or(Error::Binding)
                } else {
                    Err(Error::Binding)
                }
            }
            _ => Err(Error::Binding),
        })
        .collect()
}

fn query_rows(
    connection: &Connection,
    query: Statement,
    bindings: Vec<Binding>,
) -> Result<Vec<Value>> {
    let mut prepared = connection.prepare_cached(query.sql)?;
    if !prepared.readonly() {
        return Err(Error::NotReadOnly);
    }
    let mut cursor = prepared.query(params_from_iter(bindings))?;
    let mut rows = Vec::new();
    while let Some(row) = cursor.next()? {
        let encoded: String = row.get(0)?;
        rows.push(serde_json::from_str(&encoded)?);
    }
    Ok(rows)
}
