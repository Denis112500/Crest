//! Reads the last bytes of a Claude Code transcript once, for the lines Claude Code writes as a
//! turn or a session ends (a continuation under a new id, a reached usage limit). Following a
//! transcript as it grows is `transcript_tail_reader.rs`'s job.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// The file's last `byte_count` bytes as text; the first line may be cut, which only makes it
/// unreadable JSON. Empty if the file can't be read.
pub fn read_transcript_end(transcript_path: &Path, byte_count: u64) -> String {
    let Ok(mut transcript_file) = File::open(transcript_path) else {
        return String::new();
    };
    let file_length = transcript_file.metadata().map(|metadata| metadata.len()).unwrap_or(0);
    let read_start = file_length.saturating_sub(byte_count);
    let mut end_bytes = Vec::new();
    if transcript_file.seek(SeekFrom::Start(read_start)).is_err() || transcript_file.read_to_end(&mut end_bytes).is_err() {
        return String::new();
    }
    String::from_utf8_lossy(&end_bytes).into_owned()
}
