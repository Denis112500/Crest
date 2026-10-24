//! What the alert says Claude wants to do: a command exactly as it will run, a connected tool
//! with its input, or a short action like "Edit notes.md". The user decides on this text, so
//! nothing in it may hide what runs: invisible and direction-changing characters show as a mark.

use serde::Serialize;
use serde_json::Value;

use crate::activity_sources::claude_code::claude_code_permission_action_wording::word_permission_action;
use crate::activity_sources::claude_code::claude_code_permission_file_path::PermissionRequestFolders;
use crate::activity_sources::claude_code::claude_code_permission_tool_call_listing::list_connected_tool_call_input;
use crate::activity_sources::claude_code::visible_text_shaping::{cut_with_ellipsis, make_hidden_characters_visible};
use crate::backend_constants::{
    CLAUDE_CODE_PERMISSION_ACTION_MAX_CHARACTERS, CLAUDE_CODE_PERMISSION_COMMAND_MAX_CHARACTERS,
    CLAUDE_CODE_PERMISSION_TOOL_CALL_MAX_CHARACTERS,
};

/// Mirrored in `claudeCodeStatusTypes.ts`.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "descriptionKind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ClaudeCodePermissionDescription {
    /// Bash or PowerShell. `is_command_cut`: longer than Crest sends, so the pill can't show all
    /// of it and offers only Deny.
    Command { command_text: String, is_command_cut: bool },
    /// A connected (MCP) tool with input: its name on the first line, then its input fields.
    /// `is_call_cut` as for a command.
    ToolCall { call_text: String, is_call_cut: bool },
    /// Every other tool, in a few words on one line.
    Action { action_text: String },
}

pub fn describe_permission_request(
    tool_name: &str,
    tool_input: Option<&Value>,
    request_folders: &PermissionRequestFolders,
) -> ClaudeCodePermissionDescription {
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
            let action_wording = word_permission_action(tool_name, tool_input, request_folders);
            if let Some(input_listing) = list_connected_tool_call_input(tool_name, tool_input) {
                let visible_call_text = make_hidden_characters_visible(&format!("{action_wording}\n{input_listing}"));
                return ClaudeCodePermissionDescription::ToolCall {
                    is_call_cut: visible_call_text.chars().count() > CLAUDE_CODE_PERMISSION_TOOL_CALL_MAX_CHARACTERS,
                    call_text: cut_with_ellipsis(&visible_call_text, CLAUDE_CODE_PERMISSION_TOOL_CALL_MAX_CHARACTERS),
                };
            }
            let action_text = make_hidden_characters_visible(&action_wording).replace('\n', " ");
            ClaudeCodePermissionDescription::Action {
                action_text: cut_with_ellipsis(&action_text, CLAUDE_CODE_PERMISSION_ACTION_MAX_CHARACTERS),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::activity_sources::claude_code::visible_text_shaping::ELLIPSIS;

    const NO_FOLDERS: PermissionRequestFolders = PermissionRequestFolders { project_folder: None, home_folder: None };

    fn describe_command(command_text: &str) -> (String, bool) {
        match describe_permission_request("Bash", Some(&json!({ "command": command_text })), &NO_FOLDERS) {
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
        let action_of =
            |tool_name: &str, tool_input: Option<&Value>| describe_permission_request(tool_name, tool_input, &NO_FOLDERS);
        assert_eq!(
            action_of("Edit", Some(&file_input)),
            ClaudeCodePermissionDescription::Action { action_text: "Edit C:\\project\\evil\u{FFFD}txt.exe".into() }
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
    fn a_connected_tool_shows_its_input_under_its_name_and_is_cut_like_a_command() {
        let issue_input = json!({ "title": "Fix\u{202E}txt", "body": "x".repeat(CLAUDE_CODE_PERMISSION_TOOL_CALL_MAX_CHARACTERS) });
        let ClaudeCodePermissionDescription::ToolCall { call_text, is_call_cut } =
            describe_permission_request("mcp__github__create_issue", Some(&issue_input), &NO_FOLDERS)
        else {
            panic!("expected a tool call");
        };
        assert!(call_text.starts_with("github: create issue\ntitle: Fix\u{FFFD}txt\nbody: xxx"));
        assert!(is_call_cut);
        assert_eq!(call_text.chars().count(), CLAUDE_CODE_PERMISSION_TOOL_CALL_MAX_CHARACTERS);
        assert_eq!(
            serde_json::to_value(describe_permission_request("mcp__files__read", Some(&json!({ "path": "a" })), &NO_FOLDERS)).unwrap(),
            json!({ "descriptionKind": "toolCall", "callText": "files: read\npath: a", "isCallCut": false })
        );
    }

    #[test]
    fn the_page_gets_the_kind_and_camel_case_fields() {
        let description = describe_permission_request("Bash", Some(&json!({ "command": "ls" })), &NO_FOLDERS);
        assert_eq!(
            serde_json::to_value(description).unwrap(),
            json!({ "descriptionKind": "command", "commandText": "ls", "isCommandCut": false })
        );
    }
}
