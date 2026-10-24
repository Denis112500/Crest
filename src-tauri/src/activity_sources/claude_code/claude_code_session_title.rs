//! A session's title, shown after its folder so several sessions of one folder can be told apart.
//! Two sources: a title the user chose (`--name`, `/rename`), which hooks carry as
//! `session_title` (verified from docs), and the transcript's title lines. Those are Claude Code's
//! internal format (seen 2026-10-24, Claude Code 2.1.310): `{"type":"ai-title","aiTitle":…}` for
//! the title Claude Code makes up during the first turn (the one Windows Terminal shows as the
//! tab's name), `{"type":"custom-title","customTitle":…}` for a chosen one. If the format
//! changes, nothing matches and the row shows the folder alone.

use serde_json::Value;

use crate::activity_sources::claude_code::visible_text_shaping::{cut_at_word_with_ellipsis, make_hidden_characters_visible};
use crate::backend_constants::CLAUDE_CODE_SESSION_TITLE_MAX_CHARACTERS;

/// Checked before parsing: transcript lines can be hundreds of KB, and titles are rare.
const GENERATED_TITLE_TYPE: &str = "ai-title";
const CHOSEN_TITLE_TYPE: &str = "custom-title";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaudeCodeSessionTitle {
    pub title_text: String,
    pub is_chosen_by_user: bool,
}

impl ClaudeCodeSessionTitle {
    /// On one line, hidden characters visible, cut at a word; `None` when nothing is left.
    pub fn new(raw_title: &str, is_chosen_by_user: bool) -> Option<Self> {
        let one_line_title = make_hidden_characters_visible(raw_title).split_whitespace().collect::<Vec<_>>().join(" ");
        (!one_line_title.is_empty()).then(|| Self {
            title_text: cut_at_word_with_ellipsis(&one_line_title, CLAUDE_CODE_SESSION_TITLE_MAX_CHARACTERS),
            is_chosen_by_user,
        })
    }

    /// A made-up title never replaces one the user chose.
    pub fn replaces(&self, current_title: Option<&ClaudeCodeSessionTitle>) -> bool {
        self.is_chosen_by_user || !current_title.is_some_and(|current_title| current_title.is_chosen_by_user)
    }
}

pub fn find_transcript_session_title(transcript_line: &str) -> Option<ClaudeCodeSessionTitle> {
    if !transcript_line.contains(GENERATED_TITLE_TYPE) && !transcript_line.contains(CHOSEN_TITLE_TYPE) {
        return None;
    }
    let transcript_entry = serde_json::from_str::<Value>(transcript_line).ok()?;
    let (title_field, is_chosen_by_user) = match transcript_entry.get("type").and_then(Value::as_str)? {
        GENERATED_TITLE_TYPE => ("aiTitle", false),
        CHOSEN_TITLE_TYPE => ("customTitle", true),
        _ => return None,
    };
    ClaudeCodeSessionTitle::new(transcript_entry.get(title_field)?.as_str()?, is_chosen_by_user)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_kinds_of_title_line_are_read_and_other_lines_are_not() {
        let generated_line = r#"{"type":"ai-title","aiTitle":"Tauri source directory","sessionId":"a"}"#;
        assert_eq!(
            find_transcript_session_title(generated_line),
            Some(ClaudeCodeSessionTitle { title_text: "Tauri source directory".into(), is_chosen_by_user: false })
        );
        let chosen_line = r#"{"type":"custom-title","customTitle":"CC.2 milestone","sessionId":"a"}"#;
        assert!(find_transcript_session_title(chosen_line).unwrap().is_chosen_by_user);
        let answer_mentioning_titles = r#"{"type":"assistant","message":{"content":"the ai-title line"}}"#;
        assert_eq!(find_transcript_session_title(answer_mentioning_titles), None);
        assert_eq!(find_transcript_session_title(r#"{"type":"ai-title","aiTitle":"  \n "}"#), None);
    }

    #[test]
    fn a_title_is_one_visible_line_cut_at_a_word() {
        let shaped_title = ClaudeCodeSessionTitle::new("Fix\u{202E}txt\n  the  pill", false).unwrap();
        assert_eq!(shaped_title.title_text, "Fix\u{FFFD}txt the pill");
        let long_title = ClaudeCodeSessionTitle::new(&"word ".repeat(CLAUDE_CODE_SESSION_TITLE_MAX_CHARACTERS), false).unwrap();
        assert!(long_title.title_text.chars().count() <= CLAUDE_CODE_SESSION_TITLE_MAX_CHARACTERS);
    }

    #[test]
    fn a_made_up_title_never_replaces_a_chosen_one() {
        let generated_title = ClaudeCodeSessionTitle::new("TCP", false).unwrap();
        let chosen_title = ClaudeCodeSessionTitle::new("Networking notes", true).unwrap();
        assert!(generated_title.replaces(None));
        assert!(generated_title.replaces(Some(&generated_title)));
        assert!(!generated_title.replaces(Some(&chosen_title)));
        assert!(chosen_title.replaces(Some(&generated_title)));
    }
}
