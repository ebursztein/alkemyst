use super::{Error, Result};
use crate::statements::system;
use rusqlite::{Connection, OpenFlags};
use std::{collections::BTreeMap, io, path::Path, time::Duration};

pub(super) fn write(connection: &Connection, destination: &Path) -> Result<()> {
    available(destination)?;
    publish(copy(connection, parent(destination))?, destination)
}

pub(super) fn stage(current: &Connection, source: &Path, destination: &Path) -> Result<()> {
    available(destination)?;
    publish(checked(current, source, parent(destination))?, destination)
}

pub(super) fn restore(current: &mut Connection, source: &Path) -> Result<BTreeMap<String, i64>> {
    let candidate = checked(current, source, &std::env::temp_dir())?;
    let connection = read_only(candidate.path())?;
    let revisions = connection
        .prepare(system::REVISIONS)?
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<std::result::Result<_, _>>()?;
    drop(connection);
    current.flush_prepared_statement_cache();
    // SQLite rolls back the destination transaction if restoration fails.
    // The selected source remains read-only; restore opens only our private candidate.
    current.restore(
        rusqlite::MAIN_DB,
        candidate.path(),
        None::<fn(rusqlite::backup::Progress)>,
    )?;
    Ok(revisions)
}

fn checked(
    current: &Connection,
    source: &Path,
    directory: &Path,
) -> Result<tempfile::NamedTempFile> {
    let original = read_only(source)?;
    let temporary = copy(&original, directory)?;
    drop(original);
    // Validate the copied snapshot, so a concurrently changing source cannot pass
    // validation for one database and then supply different data for restoration.
    // SQLite omits CHECK expressions when loading a read-only B-tree. Open only
    // our private copy read/write, then prohibit writes through query_only so
    // integrity_check also evaluates the persisted CHECK constraints.
    let candidate = Connection::open(temporary.path())?;
    candidate.execute_batch(system::BACKUP_READ_ONLY)?;
    let expected: i64 = current.query_row(system::SCHEMA_VERSION, [], |row| row.get(0))?;
    let actual: i64 = candidate.query_row(system::SCHEMA_VERSION, [], |row| row.get(0))?;
    if actual > expected {
        return Err(Error::FutureSchema);
    }
    if actual != expected || schema(&candidate)? != schema(current)? {
        return Err(Error::InvalidBackup("unsupported database schema"));
    }
    let integrity = candidate
        .prepare(system::BACKUP_INTEGRITY)?
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if integrity != ["ok"] {
        return Err(Error::InvalidBackup("integrity check failed"));
    }
    if candidate
        .prepare(system::BACKUP_FOREIGN_KEYS)?
        .query([])?
        .next()?
        .is_some()
    {
        return Err(Error::InvalidBackup("foreign key check failed"));
    }
    drop(candidate);
    Ok(temporary)
}

fn read_only(path: &Path) -> Result<Connection> {
    let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    connection.busy_timeout(Duration::from_secs(5))?;
    connection.execute_batch(system::BACKUP_READ_ONLY)?;
    Ok(connection)
}

fn schema(connection: &Connection) -> Result<Vec<String>> {
    Ok(connection
        .prepare(system::BACKUP_SCHEMA)?
        .query_map([], |row| row.get(0))?
        .collect::<std::result::Result<_, _>>()?)
}

fn available(destination: &Path) -> Result<()> {
    // Check early to avoid copying a large catalog for a destination already in use.
    // persist_noclobber repeats this check atomically at publication, closing the race.
    match destination.symlink_metadata() {
        Ok(_) => return Err(io::Error::from(io::ErrorKind::AlreadyExists).into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

fn parent(destination: &Path) -> &Path {
    destination
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn copy(connection: &Connection, directory: &Path) -> Result<tempfile::NamedTempFile> {
    let temporary = tempfile::NamedTempFile::new_in(directory)?;
    // SQLite's online backup reads committed WAL pages as well as the main file.
    // All operations run on the SQL worker, between complete application writes.
    connection.backup(rusqlite::MAIN_DB, temporary.path(), None)?;
    temporary.as_file().sync_all()?;
    Ok(temporary)
}

fn publish(temporary: tempfile::NamedTempFile, destination: &Path) -> Result<()> {
    temporary
        .persist_noclobber(destination)
        .map_err(|error| error.error)?;
    Ok(())
}
