//! The short line a finished session shows under its row, from the final answer Claude Code
//! sends with `Stop` (`last_assistant_message`: the raw markdown of the turn's last message,
//! verified 2026-10-24). Claude's text is shown, so nothing in it may hide or reorder what it says.

use crate::activity_sources::claude_code::visible_text_shaping::{cut_with_ellipsis, make_hidden_characters_visible};
use crate::backend_constants::CLAUDE_CODE_TURN_SUMMARY_MAX_CHARACTERS;

/// The answer's first non-empty line, or `None` for an answer with no text.
pub fn summarize_final_answer(final_answer: &str) -> Option<String> {
    let first_line = final_answer.lines().map(str::trim).find(|answer_line| !answer_line.is_empty())?;
    Some(cut_with_ellipsis(&make_hidden_characters_visible(first_line), CLAUDE_CODE_TURN_SUMMARY_MAX_CHARACTERS))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity_sources::claude_code::visible_text_shaping::ELLIPSIS;

    #[test]
    fn the_first_non_empty_line_is_the_summary() {
        let final_answer = "\r\n  The ping finished: 0% loss.  \r\n\r\n- a list item\n";
        assert_eq!(summarize_final_answer(final_answer).as_deref(), Some("The ping finished: 0% loss."));
        assert_eq!(summarize_final_answer(" \n\t\n"), None);
    }

    #[test]
    fn hidden_characters_show_and_a_long_line_is_cut() {
        assert_eq!(summarize_final_answer("Done\u{202E}txt.exe").as_deref(), Some("Done\u{FFFD}txt.exe"));
        let long_summary = summarize_final_answer(&"x".repeat(CLAUDE_CODE_TURN_SUMMARY_MAX_CHARACTERS + 1)).unwrap();
        assert_eq!(long_summary.chars().count(), CLAUDE_CODE_TURN_SUMMARY_MAX_CHARACTERS);
        assert!(long_summary.ends_with(ELLIPSIS));
    }
}
