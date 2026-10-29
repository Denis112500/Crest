//! The prompt a turn started with, as the first line of its unfolded steps (CC.3c): its first
//! line with text, hidden characters made visible, cut at a word. Kept only in memory and shown
//! only in the pill.

use crate::activity_sources::claude_code::visible_text_shaping::{cut_at_word_with_ellipsis, make_hidden_characters_visible};
use crate::backend_constants::CLAUDE_CODE_TURN_PROMPT_MAX_CHARACTERS;

/// `None` for a prompt with no text (an image alone, or whitespace).
pub fn describe_turn_prompt(prompt_text: &str) -> Option<String> {
    let first_line = prompt_text.lines().map(str::trim).find(|prompt_line| !prompt_line.is_empty())?;
    Some(cut_at_word_with_ellipsis(&make_hidden_characters_visible(first_line), CLAUDE_CODE_TURN_PROMPT_MAX_CHARACTERS))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_line_with_text_is_kept_and_cut_at_a_word() {
        assert_eq!(describe_turn_prompt("\n  Fix the tests\nthen commit").as_deref(), Some("Fix the tests"));
        let long_prompt = "Please read every file in the project and then explain how the pill decides its light";
        let prompt_line = describe_turn_prompt(long_prompt).unwrap();
        assert!(prompt_line.chars().count() <= CLAUDE_CODE_TURN_PROMPT_MAX_CHARACTERS);
        assert!(prompt_line.ends_with('…') && !prompt_line.contains("pill"));
        assert_eq!(describe_turn_prompt(" \n\t\n"), None);
        assert_eq!(describe_turn_prompt("run\u{202E}txt.exe").as_deref(), Some("run\u{FFFD}txt.exe"));
    }
}
