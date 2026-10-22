use super::{Error, Result, models::Checkpoint};
use manasight_parser::{GameEvent, content_hash, log::entry::LineBuffer, router::Router};
use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::Path,
};

const MAX_BATCH: usize = 8 * 1024 * 1024;
pub(super) struct Batch {
    pub checkpoint: Checkpoint,
    pub events: Vec<GameEvent>,
    pub rotated: bool,
    pub caught_up: bool,
    pub awaiting_partial: bool,
    pub start_offset: u64,
}

pub(super) fn read(path: &Path, old: Checkpoint, retained: bool) -> Result<Batch> {
    let mut file = File::open(path)?;
    let metadata = file.metadata()?;
    let file_id = identity(&file)?;
    let changed = old.sequence > 0
        && (old.file_id != file_id
            || old.offset > metadata.len()
            || old.prefix_len as u64 > metadata.len()
            || hash_range(&mut file, 0, old.prefix_len)? != old.prefix_hash
            || anchor(&mut file, old.offset)? != old.anchor_hash);
    let mut checkpoint = if changed {
        Checkpoint {
            sequence: old.sequence,
            ..Default::default()
        }
    } else {
        old
    };
    let start_offset = checkpoint.offset;
    file.seek(SeekFrom::Start(start_offset))?;
    let mut bytes = Vec::new();
    (&mut file).take(MAX_BATCH as u64).read_to_end(&mut bytes)?;
    let caught_up = start_offset + bytes.len() as u64 >= file.metadata()?.len();
    let length = if retained && caught_up {
        bytes.len()
    } else {
        bytes
            .iter()
            .rposition(|b| *b == b'\n')
            .map_or(0, |index| index + 1)
    };
    let text = std::str::from_utf8(&bytes[..length])
        .map_err(|_| Error::Invalid("Arena log is not valid UTF-8"))?;
    let mut buffer = LineBuffer::new();
    let router = Router::new();
    let mut events = vec![];
    let mut line_start = start_offset;
    for line in text.split_inclusive('\n') {
        let line_end = line_start + line.len() as u64;
        let entries = buffer.push_line(line.trim_end_matches(['\r', '\n']));
        if buffer.is_empty() {
            checkpoint.offset = line_end;
        } else if !entries.is_empty() {
            checkpoint.offset = line_start;
        }
        for entry in entries {
            events.extend(router.route(&entry));
        }
        line_start = line_end;
    }
    let awaiting_partial = checkpoint.offset < start_offset + bytes.len() as u64;
    if !caught_up && checkpoint.offset == start_offset {
        return Err(Error::Invalid(
            "Arena entry exceeds the 8 MiB capture limit",
        ));
    }
    checkpoint.file_id = file_id;
    checkpoint.prefix_len = metadata.len().min(256) as usize;
    checkpoint.prefix_hash = hash_range(&mut file, 0, checkpoint.prefix_len)?;
    checkpoint.anchor_hash = anchor(&mut file, checkpoint.offset)?;
    Ok(Batch {
        checkpoint,
        events,
        rotated: changed,
        caught_up,
        awaiting_partial,
        start_offset,
    })
}
fn hash_range(file: &mut File, offset: u64, length: usize) -> Result<String> {
    file.seek(SeekFrom::Start(offset))?;
    let mut bytes = vec![0; length];
    file.read_exact(&mut bytes)?;
    Ok(content_hash(&bytes))
}
fn anchor(file: &mut File, offset: u64) -> Result<String> {
    let length = offset.min(64);
    hash_range(file, offset - length, length as usize)
}
fn identity(file: &File) -> Result<String> {
    use std::hash::{DefaultHasher, Hash, Hasher};
    // Native volume/file IDs (Windows) and device/inode IDs (Unix), without
    // relying on timestamps. A hash-version change causes a safe replay.
    let handle = same_file::Handle::from_file(file.try_clone()?)?;
    let mut hash = DefaultHasher::new();
    handle.hash(&mut hash);
    Ok(format!("file-v1:{:016x}", hash.finish()))
}
