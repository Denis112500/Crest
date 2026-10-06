//! Turns a tool call from a `PreToolUse` event into the short line the pill shows, like
//! "Editing notes.md" or "Running cargo test". Only file names, never full paths or contents.

use serde_json::Value;

use crate::backend_constants::CLAUDE_CODE_TOOL_SUMMARY_MAX_CHARACTERS;

const ELLIPSIS: char = '…';

pub fn summarize_tool_use(tool_name: &str, tool_input: Option<&Value>) -> String {
    let read_input_text = |field_name: &str| tool_input.and_then(|input| input.get(field_name)).and_then(Value::as_str);
    let tool_summary = match tool_name {
        "Edit" | "MultiEdit" | "Write" => describe_file_action("Editing", read_input_text("file_path")),
        "NotebookEdit" => describe_file_action("Editing", read_input_text("notebook_path")),
        "Read" => describe_file_action("Reading", read_input_text("file_path")),
        "Bash" | "PowerShell" => match read_input_text("command").map(first_line_of) {
            Some(command_line) if !command_line.is_empty() => format!("Running {command_line}"),
            _ => "Running a command".to_string(),
        },
        "Grep" | "Glob" => "Searching the code".to_string(),
        "WebSearch" => "Searching the web".to_string(),
        "WebFetch" => "Reading a web page".to_string(),
        "Task" | "Agent" => "Running a subagent".to_string(),
        "TodoWrite" => "Updating its to-do list".to_string(),
        other_tool_name if other_tool_name.starts_with("mcp__") => "Using a connected tool".to_string(),
        other_tool_name => format!("Using {other_tool_name}"),
    };
    shorten_to_limit(&tool_summary)
}

fn describe_file_action(action: &str, file_path: Option<&str>) -> String {
    match file_path.and_then(|path| path.rsplit(['\\', '/']).find(|part| !part.is_empty())) {
        Some(file_name) => format!("{action} {file_name}"),
        None => format!("{action} a file"),
    }
}

fn first_line_of(command_text: &str) -> &str {
    command_text.lines().next().unwrap_or_default().trim()
}

fn shorten_to_limit(tool_summary: &str) -> String {
    if tool_summary.chars().count() <= CLAUDE_CODE_TOOL_SUMMARY_MAX_CHARACTERS {
        return tool_summary.to_string();
    }
    let mut shortened: String = tool_summary.chars().take(CLAUDE_CODE_TOOL_SUMMARY_MAX_CHARACTERS - 1).collect();
    shortened.push(ELLIPSIS);
    shortened
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn file_tools_show_only_the_file_name() {
        let edit_input = json!({ "file_path": r"C:\Users\someone\project\notes.md", "old_string": "a" });
        assert_eq!(summarize_tool_use("Edit", Some(&edit_input)), "Editing notes.md");
        assert_eq!(summarize_tool_use("Read", Some(&json!({ "file_path": "/home/x/src/lib.rs" }))), "Reading lib.rs");
        assert_eq!(summarize_tool_use("Write", None), "Editing a file");
    }

    #[test]
    fn commands_show_their_first_line_cut_to_the_limit() {
        let short_command = json!({ "command": "cargo test\necho done" });
        assert_eq!(summarize_tool_use("Bash", Some(&short_command)), "Running cargo test");
        let long_command = json!({ "command": "npm run tauri build -- --no-bundle --verbose --target x86_64" });
        let long_summary = summarize_tool_use("PowerShell", Some(&long_command));
        assert_eq!(long_summary.chars().count(), CLAUDE_CODE_TOOL_SUMMARY_MAX_CHARACTERS);
        assert!(long_summary.ends_with(ELLIPSIS));
    }

    #[test]
    fn other_tools_get_a_plain_description() {
        assert_eq!(summarize_tool_use("Grep", None), "Searching the code");
        assert_eq!(summarize_tool_use("mcp__github__create_issue", None), "Using a connected tool");
        assert_eq!(summarize_tool_use("SomethingNew", None), "Using SomethingNew");
    }
}
