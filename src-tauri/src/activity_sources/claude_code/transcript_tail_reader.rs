//! Reads only what was added to a growing file since the last look, line by line. Started at the
//! file's end, so a transcript's earlier content is never read at all.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

pub struct TranscriptTailReader {
    transcript_path: PathBuf,
    read_offset: u64,
    /// The start of a line whose end hasn't been written yet.
    unfinished_line_bytes: Vec<u8>,
}

impl TranscriptTailReader {
    pub fn starting_at_end(transcript_path: &Path) -> Self {
        let file_length = std::fs::metadata(transcript_path).map(|metadata| metadata.len()).unwrap_or(0);
        Self { transcript_path: transcript_path.to_path_buf(), read_offset: file_length, unfinished_line_bytes: Vec::new() }
    }

    pub fn transcript_path(&self) -> &Path {
        &self.transcript_path
    }

    /// The complete lines added since the last call. A file that got shorter (rewritten) is
    /// followed from its new end.
    pub fn read_new_lines(&mut self) -> Vec<String> {
        let Ok(mut transcript_file) = File::open(&self.transcript_path) else {
            return Vec::new();
        };
        let file_length = transcript_file.metadata().map(|metadata| metadata.len()).unwrap_or(0);
        if file_length < self.read_offset {
            self.read_offset = file_length;
            self.unfinished_line_bytes.clear();
            return Vec::new();
        }
        let mut added_bytes = Vec::new();
        if transcript_file.seek(SeekFrom::Start(self.read_offset)).is_err()
            || transcript_file.read_to_end(&mut added_bytes).is_err()
        {
            return Vec::new();
        }
        self.read_offset += added_bytes.len() as u64;
        self.unfinished_line_bytes.extend_from_slice(&added_bytes);
        let Some(last_line_end) = self.unfinished_line_bytes.iter().rposition(|&byte| byte == b'\n') else {
            return Vec::new();
        };
        let finished_bytes: Vec<u8> = self.unfinished_line_bytes.drain(..=last_line_end).collect();
        finished_bytes
            .split(|&byte| byte == b'\n')
            .filter(|line_bytes| !line_bytes.is_empty())
            .map(|line_bytes| String::from_utf8_lossy(line_bytes).into_owned())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::fs::OpenOptions;
    use std::io::Write;

    use super::*;

    #[test]
    fn only_lines_added_after_the_start_are_read_and_a_half_line_waits_for_its_end() {
        let transcript_path = std::env::temp_dir().join(format!("crest-transcript-test-{}.jsonl", std::process::id()));
        std::fs::write(&transcript_path, "old line\n").unwrap();
        let mut tail_reader = TranscriptTailReader::starting_at_end(&transcript_path);
        let mut transcript_file = OpenOptions::new().append(true).open(&transcript_path).unwrap();
        transcript_file.write_all(b"first new line\nsecond ha").unwrap();
        assert_eq!(tail_reader.read_new_lines(), vec!["first new line".to_string()]);
        transcript_file.write_all(b"lf\n").unwrap();
        assert_eq!(tail_reader.read_new_lines(), vec!["second half".to_string()]);
        assert!(tail_reader.read_new_lines().is_empty());
        drop(transcript_file);
        std::fs::remove_file(&transcript_path).unwrap();
    }
}
