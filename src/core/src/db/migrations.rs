use super::{Error, Result};
use crate::statements::system;
use rusqlite::Connection;

const STEPS: &[&[&str]] = &[
    &[system::INITIAL],
    &[system::CATALOG],
    &[system::ARENA_ALIASES],
    &[system::DECKS],
    &[system::RECEIPT_ORIGINS],
    &[system::CAPTURE],
    &[
        system::HISTORY_ORDER,
        system::HISTORY_MATCH_FACTS,
        system::HISTORY_FACTS,
        system::HISTORY_SUMMARIES,
        system::HISTORY_MATCHES,
    ],
    &[system::SIMULATIONS, system::SIMULATION_DETAILS],
    &[system::PRACTICE],
];
pub(super) const CURRENT_VERSION: i64 = STEPS.len() as i64;

pub(super) fn apply_to(connection: &mut Connection, target: i64) -> Result<()> {
    let actual: i64 = connection.query_row(system::SCHEMA_VERSION, [], |row| row.get(0))?;
    if actual > target || target > CURRENT_VERSION {
        return Err(Error::FutureSchema);
    }
    if actual < 0 || target < 1 {
        return Err(Error::InvalidBackup("unsupported database schema"));
    }
    for scripts in STEPS.iter().take(target as usize).skip(actual as usize) {
        let transaction = connection.transaction()?;
        for sql in *scripts {
            transaction.execute_batch(sql)?;
        }
        transaction.commit()?;
    }
    Ok(())
}

pub(super) fn known_schema(version: i64) -> Result<Vec<String>> {
    if !(1..=CURRENT_VERSION).contains(&version) {
        return Err(Error::InvalidBackup("unsupported database schema"));
    }
    let mut reference = Connection::open_in_memory()?;
    reference.execute_batch(system::CONFIGURE)?;
    apply_to(&mut reference, version)?;
    schema(&reference)
}

pub(super) fn schema(connection: &Connection) -> Result<Vec<String>> {
    Ok(connection
        .prepare(system::BACKUP_SCHEMA)?
        .query_map([], |row| row.get(0))?
        .collect::<std::result::Result<_, _>>()?)
}
