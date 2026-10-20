//! What a connected (MCP) tool would be called with, one "key: value" line per input field, for
//! the alert's box: its name alone ("github: create issue") doesn't say what it would do.

use serde_json::Value;

use crate::activity_sources::claude_code::claude_code_permission_action_wording::MCP_TOOL_NAME_PREFIX;
use crate::backend_constants::CLAUDE_CODE_PERMISSION_TOOL_CALL_VALUE_INDENT_WIDTH;

/// `None` for every other tool, and for a connected tool without input fields: then its name is
/// all there is to show. Fields keep Claude Code's order (`serde_json` has `preserve_order`).
pub fn list_connected_tool_call_input(tool_name: &str, tool_input: Option<&Value>) -> Option<String> {
    if !tool_name.starts_with(MCP_TOOL_NAME_PREFIX) {
        return None;
    }
    let input_fields = tool_input?.as_object().filter(|input_fields| !input_fields.is_empty())?;
    let input_lines: Vec<String> = input_fields
        .iter()
        .map(|(field_name, field_value)| format!("{}: {}", show_field_name(field_name), show_field_value(field_value)))
        .collect();
    Some(input_lines.join("\n"))
}

/// A line break in a name would start a line that looks like another field.
fn show_field_name(field_name: &str) -> String {
    field_name.replace(['\r', '\n'], " ")
}

/// Text as it is, its further lines indented, so no line of a value can pass for a field of its
/// own; anything else as compact JSON, where line breaks inside strings stay escaped.
fn show_field_value(field_value: &Value) -> String {
    match field_value {
        Value::String(field_text) => {
            let line_indent = " ".repeat(CLAUDE_CODE_PERMISSION_TOOL_CALL_VALUE_INDENT_WIDTH);
            field_text.lines().collect::<Vec<_>>().join(&format!("\n{line_indent}"))
        }
        other_value => other_value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn lists_every_field_in_the_order_claude_code_sent_them() {
        let issue_input = json!({ "owner": "someone", "repo": "crest", "title": "Clicks", "labels": ["bug"], "draft": false });
        assert_eq!(
            list_connected_tool_call_input("mcp__github__create_issue", Some(&issue_input)).unwrap(),
            "owner: someone\nrepo: crest\ntitle: Clicks\nlabels: [\"bug\"]\ndraft: false"
        );
    }

    #[test]
    fn a_value_over_several_lines_cannot_pass_for_another_field() {
        let tricky_input = json!({ "body": "first line\r\nrepo: elsewhere", "nested": { "note": "a\nb" }, "odd\nname": 1 });
        assert_eq!(
            list_connected_tool_call_input("mcp__github__create_issue", Some(&tricky_input)).unwrap(),
            "body: first line\n  repo: elsewhere\nnested: {\"note\":\"a\\nb\"}\nodd name: 1"
        );
    }

    #[test]
    fn other_tools_and_connected_tools_without_input_get_no_listing() {
        assert_eq!(list_connected_tool_call_input("Edit", Some(&json!({ "file_path": "a.md" }))), None);
        assert_eq!(list_connected_tool_call_input("mcp__github__list_issues", Some(&json!({}))), None);
        assert_eq!(list_connected_tool_call_input("mcp__github__list_issues", None), None);
    }
}
