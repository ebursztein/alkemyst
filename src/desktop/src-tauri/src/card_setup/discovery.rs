use super::{CardSources, locations};
use std::{io, path::PathBuf, time::SystemTime};

#[derive(Clone, PartialEq, Eq)]
pub(super) struct Asset {
    pub path: PathBuf,
    modified: Option<SystemTime>,
    length: u64,
}
impl Asset {
    fn at(path: PathBuf) -> io::Result<Self> {
        let path = path.canonicalize()?;
        let metadata = path.metadata()?;
        if !metadata.is_file() {
            return Err(io::ErrorKind::InvalidInput.into());
        }
        Ok(Self {
            path,
            modified: metadata.modified().ok(),
            length: metadata.len(),
        })
    }
}

pub(super) fn find(sources: CardSources) -> io::Result<Vec<Asset>> {
    let directories = match sources {
        CardSources::Discover => locations::directories(),
        CardSources::Directories(directories) => directories,
        CardSources::Disabled => return Ok(vec![]),
        CardSources::File(path) => {
            return match Asset::at(path) {
                Ok(asset) => Ok(vec![asset]),
                Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(vec![]),
                Err(error) => Err(error),
            };
        }
    };
    let mut assets = vec![];
    let mut failure = None;
    for directory in directories {
        let entries = match std::fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => {
                failure = Some(error);
                continue;
            }
        };
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    failure = Some(error);
                    continue;
                }
            };
            let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
            if !name.starts_with("raw_carddatabase_") || !name.ends_with(".mtga") {
                continue;
            }
            match Asset::at(entry.path()) {
                Ok(asset) => assets.push(asset),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => failure = Some(error),
            }
        }
    }
    assets.sort_by(|a, b| {
        b.modified
            .cmp(&a.modified)
            .then_with(|| a.path.cmp(&b.path))
    });
    assets.dedup_by(|a, b| a.path == b.path);
    assets.truncate(64);
    if assets.is_empty()
        && let Some(error) = failure
    {
        return Err(error);
    }
    Ok(assets)
}
