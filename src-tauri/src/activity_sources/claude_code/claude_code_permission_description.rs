//! What the alert says Claude wants to do: a command exactly as it will run, or a short action
//! like "Edit notes.md". The user decides on this text, so nothing in it may hide what runs:
//! invisible and direction-changing characters show as a visible mark.

use serde::Serialize;
use serde_json::Value;

use crate::activity_sources::claude_code::claude_code_permission_action_wording::word_permission_action;
use crate::backend_constants::{
    CLAUDE_CODE_PERMISSION_ACTION_MAX_CHARACTERS, CLAUDE_CODE_PERMISSION_COMMAND_MAX_CHARACTERS,
    CLAUDE_CODE_PERMISSION_COMMAND_TAB_WIDTH,
};

const ELLIPSIS: char = '…';
/// Stands in for a character that would otherwise be invisible or change how the text reads.
const HIDDEN_CHARACTER_MARK: char = '\u{FFFD}';

/// Mirrored in `claudeCodeStatusTypes.ts`.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "descriptionKind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ClaudeCodePermissionDescription {
    /// Bash or PowerShell. `is_command_cut`: longer than Crest sends, so the pill can't show all
    /// of it and offers only Deny.
    Command { command_text: String, is_command_cut: bool },
    /// Every other tool, in a few words on one line.
    Action { action_text: String },
}

pub fn describe_permission_request(tool_name: &str, tool_input: Option<&Value>) -> ClaudeCodePermissionDescription {
    let command_text = tool_input.and_then(|input| input.get("command")).and_then(Value::as_str).map(str::trim);
    match (tool_name, command_text) {
        ("Bash" | "PowerShell", Some(command_text)) if !command_text.is_empty() => {
            let visible_command_text = make_hidden_characters_visible(command_text);
            let is_command_cut = visible_command_text.chars().count() > CLAUDE_CODE_PERMISSION_COMMAND_MAX_CHARACTERS;
            ClaudeCodePermissionDescription::Command {
                command_text: cut_with_ellipsis(&visible_command_text, CLAUDE_CODE_PERMISSION_COMMAND_MAX_CHARACTERS),
                is_command_cut,
            }
        }
        _ => {
            let action_text = make_hidden_characters_visible(&word_permission_action(tool_name, tool_input)).replace('\n', " ");
            ClaudeCodePermissionDescription::Action {
                action_text: cut_with_ellipsis(&action_text, CLAUDE_CODE_PERMISSION_ACTION_MAX_CHARACTERS),
            }
        }
    }
}

/// Keeps line breaks (as "\n") and turns tabs into spaces; every other control character, and
/// every character that is invisible or reorders the text around it (zero-width spaces, bidi
/// overrides, a lone carriage return), becomes the mark, so what shows is what runs.
fn make_hidden_characters_visible(original_text: &str) -> String {
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

fn cut_with_ellipsis(full_text: &str, max_characters: usize) -> String {
    if full_text.chars().count() <= max_characters {
        return full_text.to_string();
    }
    let mut cut_text: String = full_text.chars().take(max_characters - 1).collect();
    cut_text.push(ELLIPSIS);
    cut_text
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn describe_command(command_text: &str) -> (String, bool) {
        match describe_permission_request("Bash", Some(&json!({ "command": command_text }))) {
            ClaudeCodePermissionDescription::Command { command_text, is_command_cut } => (command_text, is_command_cut),
            other_description => panic!("expected a command, got {other_description:?}"),
        }
    }

    #[test]
    fn a_command_is_sent_whole_up_to_the_limit_and_cut_with_a_flag_beyond_it() {
        assert_eq!(describe_command("  cargo test\r\necho done\n"), ("cargo test\necho done".to_string(), false));
        let longest_whole_command = "x".repeat(CLAUDE_CODE_PERMISSION_COMMAND_MAX_CHARACTERS);
        assert_eq!(describe_command(&longest_whole_command), (longest_whole_command.clone(), false));
        let (cut_command, is_command_cut) = describe_command(&format!("{longest_whole_command}y"));
        assert!(is_command_cut);
        assert_eq!(cut_command.chars().count(), CLAUDE_CODE_PERMISSION_COMMAND_MAX_CHARACTERS);
        assert!(cut_command.ends_with(ELLIPSIS));
    }

    #[test]
    fn hidden_characters_in_a_command_become_visible_and_tabs_become_spaces() {
        let (shown_command, _) = describe_command("echo safe\u{202E}fr- mr\rnext\u{200B}\n\tindented\u{1B}[0m");
        assert_eq!(shown_command, "echo safe\u{FFFD}fr- mr\u{FFFD}next\u{FFFD}\n    indented\u{FFFD}[0m");
    }

    #[test]
    fn other_tools_get_one_visible_line() {
        let file_input = json!({ "file_path": "C:\\project\\evil\u{202E}txt.exe" });
        let action_of = |tool_name: &str, tool_input: Option<&Value>| describe_permission_request(tool_name, tool_input);
        assert_eq!(
            action_of("Edit", Some(&file_input)),
            ClaudeCodePermissionDescription::Action { action_text: "Edit evil\u{FFFD}txt.exe".into() }
        );
        assert_eq!(action_of("Bash", Some(&json!({ "command": "  " }))), ClaudeCodePermissionDescription::Action { action_text: "Run a command".into() });
        let ClaudeCodePermissionDescription::Action { action_text } =
            action_of("WebSearch", Some(&json!({ "query": format!("two\nlines {}", "q".repeat(200)) })))
        else {
            panic!("expected an action");
        };
        assert!(action_text.starts_with("Search the web for “two lines q"));
        assert_eq!(action_text.chars().count(), CLAUDE_CODE_PERMISSION_ACTION_MAX_CHARACTERS);
    }

    #[test]
    fn the_page_gets_the_kind_and_camel_case_fields() {
        let description = describe_permission_request("Bash", Some(&json!({ "command": "ls" })));
        assert_eq!(
            serde_json::to_value(description).unwrap(),
            json!({ "descriptionKind": "command", "commandText": "ls", "isCommandCut": false })
        );
    }
}
