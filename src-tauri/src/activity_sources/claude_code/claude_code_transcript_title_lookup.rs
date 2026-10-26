//! The title of a session Crest meets mid-way (Crest started later, or a dev build restarted): the
//! transcript watch only sees lines written while it follows a busy session, and Claude Code
//! writes its made-up title during the first turn and seldom again. So the transcript's start is
//! read once per session. Following a transcript as it grows is `transcript_tail_reader.rs`'s job.

use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::activity_sources::claude_code::claude_code_session_title::{find_transcript_session_title, ClaudeCodeSessionTitle};
use crate::backend_constants::CLAUDE_CODE_TRANSCRIPT_TITLE_SEARCH_BYTES;

/// `None` if the file can't be read or has no title line in its first bytes.
pub fn find_title_at_transcript_start(transcript_path: &Path) -> Option<ClaudeCodeSessionTitle> {
    let transcript_file = File::open(transcript_path).ok()?;
    let mut start_bytes = Vec::new();
    transcript_file.take(CLAUDE_CODE_TRANSCRIPT_TITLE_SEARCH_BYTES).read_to_end(&mut start_bytes).ok()?;
    find_title_in(&String::from_utf8_lossy(&start_bytes))
}

/// The last title line wins, except that a made-up title never replaces a chosen one. A line cut
/// at the end of the read bytes is unreadable JSON and simply doesn't match.
fn find_title_in(transcript_start: &str) -> Option<ClaudeCodeSessionTitle> {
    transcript_start.lines().filter_map(find_transcript_session_title).fold(None, |found_title, line_title| {
        if line_title.replaces(found_title.as_ref()) {
            Some(line_title)
        } else {
            found_title
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const GENERATED_TITLE_LINE: &str = r#"{"type":"ai-title","aiTitle":"Tauri source directory","sessionId":"a"}"#;

    #[test]
    fn the_title_among_other_lines_is_found_and_a_chosen_one_wins() {
        let transcript_start = format!("{{\"type\":\"user\"}}\n{GENERATED_TITLE_LINE}\n{{\"type\":\"assistant\"}}\n");
        assert_eq!(find_title_in(&transcript_start).unwrap().title_text, "Tauri source directory");
        let chosen_line = r#"{"type":"custom-title","customTitle":"Crest notes","sessionId":"a"}"#;
        let renamed_start = format!("{chosen_line}\n{GENERATED_TITLE_LINE}\n");
        assert_eq!(find_title_in(&renamed_start).unwrap().title_text, "Crest notes");
    }

    #[test]
    fn a_start_without_a_whole_title_line_has_no_title() {
        assert_eq!(find_title_in("{\"type\":\"user\"}\n"), None);
        let cut_title_line = &GENERATED_TITLE_LINE[..30];
        assert_eq!(find_title_in(&format!("{{\"type\":\"user\"}}\n{cut_title_line}")), None);
    }
}
