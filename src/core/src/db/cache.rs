use super::Statement;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

struct Entry {
    rows: Vec<Value>,
    dependencies: &'static [&'static str],
    bytes: usize,
    touched: u64,
}

pub(super) struct Cache {
    entries: HashMap<String, Entry>,
    budget: usize,
    pub bytes: usize,
    tick: u64,
}

impl Cache {
    pub fn new(budget: usize) -> Self {
        Self {
            entries: HashMap::new(),
            budget,
            bytes: 0,
            tick: 0,
        }
    }

    pub fn key(statement: Statement, params: &[Value]) -> String {
        // Include the definition, not just its human-readable name.
        serde_json::json!([statement.name, statement.sql, statement.reads, params]).to_string()
    }

    pub fn get(&mut self, key: &str) -> Option<Vec<Value>> {
        self.tick += 1;
        let entry = self.entries.get_mut(key)?;
        entry.touched = self.tick;
        Some(entry.rows.clone())
    }

    pub fn insert(&mut self, key: String, statement: Statement, rows: &[Value]) {
        let bytes = key.len() + serde_json::to_vec(rows).map_or(0, |value| value.len());
        if bytes > self.budget {
            return;
        }
        while self.bytes + bytes > self.budget || self.entries.len() >= 1024 {
            let oldest = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.touched)
                .map(|(key, _)| key.clone());
            if let Some(oldest) = oldest {
                self.remove(&oldest);
            } else {
                break;
            }
        }
        self.tick += 1;
        self.bytes += bytes;
        self.entries.insert(
            key,
            Entry {
                rows: rows.to_vec(),
                dependencies: statement.reads,
                bytes,
                touched: self.tick,
            },
        );
    }

    fn remove(&mut self, key: &str) {
        if let Some(entry) = self.entries.remove(key) {
            self.bytes -= entry.bytes;
        }
    }

    pub fn invalidate(&mut self, tables: &HashSet<&str>) {
        self.entries.retain(|_, entry| {
            let affected = entry
                .dependencies
                .iter()
                .any(|table| tables.contains(table));
            if affected {
                self.bytes -= entry.bytes;
            }
            !affected
        });
    }
}
