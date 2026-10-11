//! What the alert says Claude wants to do: a command as it will run (its first lines), or a short
//! action like "Edit notes.md". Unlike the one-line tool summary, the user decides on this text,
//! so a command isn't reduced to its first line. Files show by name only, never full paths.

use serde::Serialize;
use serde_json::Value;

use crate::activity_sources::claude_code::claude_code_tool_summary::summarize_tool_use;
use crate::backend_constants::{CLAUDE_CODE_PERMISSION_COMMAND_LINE_MAX_CHARACTERS, CLAUDE_CODE_PERMISSION_COMMAND_MAX_LINES};

const ELLIPSIS: char = '…';

/// Mirrored in `claudeCodeStatusTypes.ts`.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "descriptionKind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ClaudeCodePermissionDescription {
    /// Bash or PowerShell; the last line ends in "…" when lines were left out.
    Command { command_lines: Vec<String> },
    /// Every other tool, in a few words.
    Action { action_text: String },
}

pub fn describe_permission_request(tool_name: &str, tool_input: Option<&Value>) -> ClaudeCodePermissionDescription {
    let read_input_text = |field_name: &str| tool_input.and_then(|input| input.get(field_name)).and_then(Value::as_str);
    let file_name = |field_name: &str| {
        read_input_text(field_name)
            .and_then(|file_path| file_path.rsplit(['\\', '/']).find(|part| !part.is_empty()))
            .unwrap_or("a file")
            .to_string()
    };
    let action_text = match tool_name {
        "Bash" | "PowerShell" => match read_input_text("command").map(str::trim) {
            Some(command_text) if !command_text.is_empty() => {
                return ClaudeCodePermissionDescription::Command { command_lines: shorten_command(command_text) };
            }
            _ => "Run a command".to_string(),
        },
        "Edit" | "MultiEdit" => format!("Edit {}", file_name("file_path")),
        "NotebookEdit" => format!("Edit {}", file_name("notebook_path")),
        "Write" => format!("Write {}", file_name("file_path")),
        other_tool_name => summarize_tool_use(other_tool_name, tool_input),
    };
    ClaudeCodePermissionDescription::Action { action_text }
}

fn shorten_command(command_text: &str) -> Vec<String> {
    let all_lines: Vec<&str> = command_text.lines().map(str::trim_end).collect();
    let mut shown_lines: Vec<String> = all_lines
        .iter()
        .take(CLAUDE_CODE_PERMISSION_COMMAND_MAX_LINES)
        .map(|command_line| shorten_line(command_line))
        .collect();
    if all_lines.len() > CLAUDE_CODE_PERMISSION_COMMAND_MAX_LINES {
        if let Some(last_shown_line) = shown_lines.last_mut() {
            if !last_shown_line.ends_with(ELLIPSIS) {
                last_shown_line.push(' ');
                last_shown_line.push(ELLIPSIS);
            }
        }
    }
    shown_lines
}

fn shorten_line(command_line: &str) -> String {
    if command_line.chars().count() <= CLAUDE_CODE_PERMISSION_COMMAND_LINE_MAX_CHARACTERS {
        return command_line.to_string();
    }
    let mut shortened: String = command_line.chars().take(CLAUDE_CODE_PERMISSION_COMMAND_LINE_MAX_CHARACTERS - 1).collect();
    shortened.push(ELLIPSIS);
    shortened
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn command_lines_of(command_text: &str) -> Vec<String> {
        match describe_permission_request("Bash", Some(&json!({ "command": command_text }))) {
            ClaudeCodePermissionDescription::Command { command_lines } => command_lines,
            other_description => panic!("expected a command, got {other_description:?}"),
        }
    }

    #[test]
    fn a_short_command_shows_in_full_and_a_long_one_is_cut_with_an_ellipsis() {
        assert_eq!(command_lines_of("cargo test\necho done\n"), vec!["cargo test", "echo done"]);
        let five_lines = command_lines_of("one\ntwo\nthree\nfour\nfive");
        assert_eq!(five_lines, vec!["one", "two", "three …"]);
        let very_long_line = "x".repeat(CLAUDE_CODE_PERMISSION_COMMAND_LINE_MAX_CHARACTERS + 10);
        let cut_line = &command_lines_of(&very_long_line)[0];
        assert_eq!(cut_line.chars().count(), CLAUDE_CODE_PERMISSION_COMMAND_LINE_MAX_CHARACTERS);
        assert!(cut_line.ends_with(ELLIPSIS));
    }

    #[test]
    fn file_tools_show_only_the_file_name_and_other_tools_a_few_words() {
        let edit_input = json!({ "file_path": r"C:\Users\someone\project\notes.md" });
        let action_of = |tool_name: &str, tool_input: Option<&Value>| describe_permission_request(tool_name, tool_input);
        assert_eq!(action_of("Edit", Some(&edit_input)), ClaudeCodePermissionDescription::Action { action_text: "Edit notes.md".into() });
        assert_eq!(action_of("Write", None), ClaudeCodePermissionDescription::Action { action_text: "Write a file".into() });
        assert_eq!(action_of("Bash", Some(&json!({ "command": "  " }))), ClaudeCodePermissionDescription::Action { action_text: "Run a command".into() });
        assert_eq!(action_of("WebSearch", None), ClaudeCodePermissionDescription::Action { action_text: "Searching the web".into() });
    }

    #[test]
    fn the_page_gets_the_kind_and_camel_case_fields() {
        let description = describe_permission_request("Bash", Some(&json!({ "command": "ls" })));
        assert_eq!(serde_json::to_value(description).unwrap(), json!({ "descriptionKind": "command", "commandLines": ["ls"] }));
    }
}
