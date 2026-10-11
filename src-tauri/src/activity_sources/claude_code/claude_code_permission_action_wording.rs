//! The few words the alert shows for a request that isn't a command: "Edit notes.md", "Open
//! github.com", "Search the web for “tauri tray”". Its own phrases, not the working status's
//! ("Searching the web"), because the user answers this one. Files by name only, web pages by
//! host only.

use serde_json::Value;

const MCP_TOOL_NAME_PREFIX: &str = "mcp__";
const MCP_TOOL_NAME_SEPARATOR: &str = "__";

pub fn word_permission_action(tool_name: &str, tool_input: Option<&Value>) -> String {
    let read_input_text = |field_name: &str| {
        tool_input
            .and_then(|input| input.get(field_name))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|input_text| !input_text.is_empty())
    };
    let file_name = |field_name: &str| {
        read_input_text(field_name)
            .and_then(|file_path| file_path.rsplit(['\\', '/']).find(|part| !part.is_empty()))
            .unwrap_or("a file")
            .to_string()
    };
    let with_quoted_input = |phrase: &str, field_name: &str| match read_input_text(field_name) {
        Some(input_text) => format!("{phrase} for “{input_text}”"),
        None => phrase.to_string(),
    };
    match tool_name {
        // Only reached when the command is missing or empty.
        "Bash" | "PowerShell" => "Run a command".to_string(),
        "Edit" | "MultiEdit" => format!("Edit {}", file_name("file_path")),
        "NotebookEdit" => format!("Edit {}", file_name("notebook_path")),
        "Write" => format!("Write {}", file_name("file_path")),
        "Read" => format!("Read {}", file_name("file_path")),
        "WebFetch" => match read_input_text("url").and_then(find_web_address_host) {
            Some(web_host) => format!("Open {web_host}"),
            None => "Open a web page".to_string(),
        },
        "WebSearch" => with_quoted_input("Search the web", "query"),
        "Grep" | "Glob" => with_quoted_input("Search the code", "pattern"),
        "Task" | "Agent" => match read_input_text("description") {
            Some(subagent_task) => format!("Start a subagent: {subagent_task}"),
            None => "Start a subagent".to_string(),
        },
        "Skill" => match read_input_text("skill") {
            Some(skill_name) => format!("Use the skill {skill_name}"),
            None => "Use a skill".to_string(),
        },
        other_tool_name if other_tool_name.starts_with(MCP_TOOL_NAME_PREFIX) => word_connected_tool(other_tool_name),
        other_tool_name => format!("Use {other_tool_name}"),
    }
}

/// "https://user@GitHub.com:443/path?q" → "github.com". Everything before an `@` is only a user
/// name, however much it looks like a site: `https://github.com@evil.example/` opens
/// evil.example. A backslash ends the host as a slash does, as browsers read it.
fn find_web_address_host(web_address: &str) -> Option<String> {
    let after_scheme = web_address.split_once("://").map_or(web_address, |(_, rest)| rest);
    let authority = after_scheme.split(['/', '\\', '?', '#']).next()?;
    let host_and_port = authority.rsplit('@').next()?;
    let web_host = match host_and_port.strip_prefix('[') {
        // An IPv6 address keeps its brackets; its colons aren't a port.
        Some(ipv6_rest) => format!("[{}]", ipv6_rest.split(']').next()?),
        None => host_and_port.split(':').next()?.to_string(),
    };
    (!web_host.is_empty() && web_host != "[]").then(|| web_host.to_lowercase())
}

/// "mcp__github__create_issue" → "github: create issue".
fn word_connected_tool(mcp_tool_name: &str) -> String {
    let server_and_tool = mcp_tool_name.trim_start_matches(MCP_TOOL_NAME_PREFIX);
    match server_and_tool.split_once(MCP_TOOL_NAME_SEPARATOR) {
        Some((server_name, tool_name)) if !server_name.is_empty() && !tool_name.is_empty() => {
            format!("{server_name}: {}", tool_name.replace('_', " "))
        }
        _ => "Use a connected tool".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn word_with_input(tool_name: &str, tool_input: Value) -> String {
        word_permission_action(tool_name, Some(&tool_input))
    }

    #[test]
    fn file_tools_name_only_the_file() {
        assert_eq!(word_with_input("Edit", json!({ "file_path": r"C:\Users\someone\project\notes.md" })), "Edit notes.md");
        assert_eq!(word_with_input("Read", json!({ "file_path": "/home/someone/src/lib.rs" })), "Read lib.rs");
        assert_eq!(word_permission_action("Write", None), "Write a file");
    }

    #[test]
    fn a_web_page_is_named_by_its_real_host_only() {
        let open_page = |web_address: &str| word_with_input("WebFetch", json!({ "url": web_address }));
        assert_eq!(open_page("https://docs.rs/serde/latest/serde/?search=x#top"), "Open docs.rs");
        assert_eq!(open_page("https://github.com@evil.example/login"), "Open evil.example");
        assert_eq!(open_page("https://user:secret@GitHub.com:8443/path"), "Open github.com");
        assert_eq!(open_page("https://evil.example\\@github.com/"), "Open evil.example");
        assert_eq!(open_page("example.org/page"), "Open example.org");
        assert_eq!(open_page("http://[::1]:1420/"), "Open [::1]");
        assert_eq!(open_page("https:///nothing"), "Open a web page");
    }

    #[test]
    fn searches_quote_what_they_look_for() {
        assert_eq!(word_with_input("WebSearch", json!({ "query": "tauri tray icon" })), "Search the web for “tauri tray icon”");
        assert_eq!(word_with_input("Grep", json!({ "pattern": "fn main" })), "Search the code for “fn main”");
        assert_eq!(word_permission_action("Glob", None), "Search the code");
    }

    #[test]
    fn subagents_skills_and_connected_tools_say_what_they_are() {
        assert_eq!(word_with_input("Task", json!({ "description": "Find the pipe code" })), "Start a subagent: Find the pipe code");
        assert_eq!(word_with_input("Skill", json!({ "skill": "code-review" })), "Use the skill code-review");
        assert_eq!(word_permission_action("mcp__github__create_issue", None), "github: create issue");
        assert_eq!(word_permission_action("mcp__broken", None), "Use a connected tool");
        assert_eq!(word_permission_action("SomethingNew", None), "Use SomethingNew");
    }
}
