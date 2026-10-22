use super::Result;
use rusqlite::Connection;
use std::{io, path::Path};

pub(super) fn write(connection: &Connection, destination: &Path) -> Result<()> {
    // Check early to avoid copying a large catalog for a destination already in use.
    // persist_noclobber repeats this check atomically at publication, closing the race.
    match destination.symlink_metadata() {
        Ok(_) => return Err(io::Error::from(io::ErrorKind::AlreadyExists).into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let parent = destination
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let temporary = tempfile::NamedTempFile::new_in(parent)?;
    // SQLite's online backup reads committed WAL pages as well as the main file.
    // All operations run on the SQL worker, between complete application writes.
    connection.backup(rusqlite::MAIN_DB, temporary.path(), None)?;
    temporary.as_file().sync_all()?;
    temporary
        .persist_noclobber(destination)
        .map_err(|error| error.error)?;
    Ok(())
}
