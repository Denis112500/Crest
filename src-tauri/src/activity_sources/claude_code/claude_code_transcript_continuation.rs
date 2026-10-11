//! Notices that Claude Code moved a conversation to a new session id. Seen 2026-10-10 with
//! Claude Code 2.1.296 after a deny and a new prompt: the old transcript ends with
//! `{"type":"continued-in","continuedInSessionId":"<new id>"}`, the new session starts with a
//! copy of the history, and the old id never sends `SessionEnd`. Without this, the old session
//! stays listed next to its successor until the silence timeout. Like the interrupt line, this
//! is Claude Code's internal format: if it changes, nothing matches and the silence timeout
//! still cleans up.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::backend_constants::CLAUDE_CODE_TRANSCRIPT_CONTINUATION_SEARCH_BYTES;

const CONTINUED_IN_ENTRY_TYPE: &str = "continued-in";

/// The sessions among `tracked_session_transcripts` that continued in `successor_session_id`.
/// Only transcripts in the successor's folder are read (a conversation stays in its project),
/// and only their last few KB, where Claude Code writes the line as it leaves the old id.
pub fn find_sessions_continued_in(
    successor_session_id: &str,
    successor_transcript_path: &Path,
    tracked_session_transcripts: Vec<(String, PathBuf)>,
) -> Vec<String> {
    tracked_session_transcripts
        .into_iter()
        .filter(|(session_id, transcript_path)| {
            session_id != successor_session_id && transcript_path.parent() == successor_transcript_path.parent()
        })
        .filter(|(_, transcript_path)| {
            read_transcript_end(transcript_path).lines().any(|line| is_continued_in_line(line, successor_session_id))
        })
        .map(|(session_id, _)| session_id)
        .collect()
}

fn is_continued_in_line(transcript_line: &str, successor_session_id: &str) -> bool {
    let Ok(transcript_entry) = serde_json::from_str::<Value>(transcript_line) else {
        return false;
    };
    transcript_entry.get("type").and_then(Value::as_str) == Some(CONTINUED_IN_ENTRY_TYPE)
        && transcript_entry.get("continuedInSessionId").and_then(Value::as_str) == Some(successor_session_id)
}

/// The file's last bytes as text; the first line may be cut, which only makes it unreadable JSON.
fn read_transcript_end(transcript_path: &Path) -> String {
    let Ok(mut transcript_file) = File::open(transcript_path) else {
        return String::new();
    };
    let file_length = transcript_file.metadata().map(|metadata| metadata.len()).unwrap_or(0);
    let read_start = file_length.saturating_sub(CLAUDE_CODE_TRANSCRIPT_CONTINUATION_SEARCH_BYTES);
    let mut end_bytes = Vec::new();
    if transcript_file.seek(SeekFrom::Start(read_start)).is_err() || transcript_file.read_to_end(&mut end_bytes).is_err() {
        return String::new();
    }
    String::from_utf8_lossy(&end_bytes).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The line as Claude Code 2.1.296 wrote it (ids shortened).
    const CONTINUED_IN_NEW_LINE: &str =
        r#"{"type":"continued-in","timestamp":"2026-10-10T17:59:55.915Z","sessionId":"old","continuedInSessionId":"new"}"#;

    #[test]
    fn only_the_line_naming_this_successor_counts() {
        assert!(is_continued_in_line(CONTINUED_IN_NEW_LINE, "new"));
        assert!(!is_continued_in_line(CONTINUED_IN_NEW_LINE, "other"));
        assert!(!is_continued_in_line(r#"{"type":"user","continuedInSessionId":"new"}"#, "new"));
        assert!(!is_continued_in_line("{ not json", "new"));
    }

    #[test]
    fn a_session_whose_transcript_ends_in_the_line_is_found_in_the_successors_folder_only() {
        let test_folder = std::env::temp_dir().join(format!("crest-continuation-test-{}", std::process::id()));
        let other_folder = test_folder.join("other-project");
        std::fs::create_dir_all(&other_folder).unwrap();
        let earlier_line = r#"{"type":"user","message":{"content":"hi"}}"#;
        let write_transcript = |transcript_path: &Path, last_line: &str| {
            std::fs::write(transcript_path, format!("{earlier_line}\n{last_line}\n")).unwrap();
            transcript_path.to_path_buf()
        };
        let continued = write_transcript(&test_folder.join("old.jsonl"), CONTINUED_IN_NEW_LINE);
        let still_running = write_transcript(&test_folder.join("running.jsonl"), r#"{"type":"assistant"}"#);
        let elsewhere = write_transcript(&other_folder.join("elsewhere.jsonl"), CONTINUED_IN_NEW_LINE);
        let successor = write_transcript(&test_folder.join("new.jsonl"), r#"{"type":"user"}"#);

        let tracked_session_transcripts = vec![
            ("old".to_string(), continued),
            ("running".to_string(), still_running),
            ("elsewhere".to_string(), elsewhere),
            ("new".to_string(), successor.clone()),
        ];
        assert_eq!(find_sessions_continued_in("new", &successor, tracked_session_transcripts), vec!["old".to_string()]);
        std::fs::remove_dir_all(&test_folder).unwrap();
    }
}
