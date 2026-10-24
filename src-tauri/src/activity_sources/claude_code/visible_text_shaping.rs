//! Text from Claude Code made fit for the pill: characters that would hide or reorder what the
//! text says become a visible mark, and long text is cut with an ellipsis. Shared by the
//! permission alert (the user decides on that text) and the summary of a finished turn.

use crate::backend_constants::CLAUDE_CODE_PERMISSION_COMMAND_TAB_WIDTH;

pub const ELLIPSIS: char = '…';
/// Stands in for a character that would otherwise be invisible or change how the text reads.
const HIDDEN_CHARACTER_MARK: char = '\u{FFFD}';

/// Keeps line breaks (as "\n") and turns tabs into spaces; every other control character, and
/// every character that is invisible or reorders the text around it (zero-width spaces, bidi
/// overrides, a lone carriage return), becomes the mark, so what shows is what runs.
pub fn make_hidden_characters_visible(original_text: &str) -> String {
    let mut visible_text = String::with_capacity(original_text.len());
    for (line_index, original_line) in original_text.split('\n').enumerate() {
        if line_index > 0 {
            visible_text.push('\n');
        }
        for line_character in original_line.strip_suffix('\r').unwrap_or(original_line).chars() {
            match line_character {
                '\t' => visible_text.extend(std::iter::repeat_n(' ', CLAUDE_CODE_PERMISSION_COMMAND_TAB_WIDTH)),
                hidden_character if hidden_character.is_control() || is_invisible_or_reordering(hidden_character) => {
                    visible_text.push(HIDDEN_CHARACTER_MARK)
                }
                shown_character => visible_text.push(shown_character),
            }
        }
    }
    visible_text
}

/// Unicode's invisible formatting characters that a terminal or a page may silently apply:
/// soft hyphen, zero-width characters, direction marks, embeddings, overrides and isolates, the
/// word joiner and invisible operators, the byte order mark, and the line/paragraph separators.
fn is_invisible_or_reordering(text_character: char) -> bool {
    matches!(
        text_character,
        '\u{00AD}' | '\u{061C}' | '\u{180E}' | '\u{200B}'..='\u{200F}' | '\u{2028}'..='\u{202E}'
            | '\u{2060}'..='\u{2064}' | '\u{2066}'..='\u{206F}' | '\u{FEFF}'
    )
}

pub fn cut_with_ellipsis(full_text: &str, max_characters: usize) -> String {
    if full_text.chars().count() <= max_characters {
        return full_text.to_string();
    }
    let mut cut_text: String = full_text.chars().take(max_characters - 1).collect();
    cut_text.push(ELLIPSIS);
    cut_text
}
