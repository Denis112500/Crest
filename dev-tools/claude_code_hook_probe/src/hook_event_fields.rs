// Pulls the few fields the probe prints out of a hook event's JSON, without a JSON crate.
// Claude Code puts these as top-level keys, and the first match is taken, which is enough
// for a probe (Crest itself will parse properly with serde_json).

const TOOL_INPUT_PREVIEW_CHARACTERS: usize = 120;

pub struct HookEventSummary {
    pub hook_event_name: String,
    pub details: String,
}

pub fn summarize_hook_event(hook_event_json: &str) -> HookEventSummary {
    let hook_event_name =
        read_string_field(hook_event_json, "hook_event_name").unwrap_or_else(|| "(no hook_event_name)".to_string());
    let mut detail_parts = Vec::new();
    if let Some(session_id) = read_string_field(hook_event_json, "session_id") {
        detail_parts.push(format!("session={}", session_id.chars().take(8).collect::<String>()));
    }
    // Only the last folder name: the full path would put the Windows user name in the log.
    if let Some(working_folder) = read_string_field(hook_event_json, "cwd") {
        let folder_name = working_folder.rsplit(['\\', '/']).find(|part| !part.is_empty()).unwrap_or_default();
        detail_parts.push(format!("folder={folder_name}"));
    }
    for field_name in ["permission_mode", "tool_name", "notification_type", "source", "reason", "message"] {
        if let Some(field_value) = read_string_field(hook_event_json, field_name) {
            detail_parts.push(format!("{field_name}={field_value}"));
        }
    }
    if let Some(tool_input) = read_object_field(hook_event_json, "tool_input") {
        let preview: String = tool_input.chars().take(TOOL_INPUT_PREVIEW_CHARACTERS).collect();
        detail_parts.push(format!("tool_input={preview}"));
    }
    HookEventSummary { hook_event_name, details: detail_parts.join("  ") }
}

/// Position right after `"field_name":` (with any spaces around the colon).
fn find_value_start(hook_event_json: &str, field_name: &str) -> Option<usize> {
    let quoted_field_name = format!("\"{field_name}\"");
    let mut search_from = 0;
    while let Some(found_at) = hook_event_json[search_from..].find(&quoted_field_name) {
        let after_name = search_from + found_at + quoted_field_name.len();
        let rest = &hook_event_json[after_name..];
        let trimmed_rest = rest.trim_start();
        if let Some(after_colon) = trimmed_rest.strip_prefix(':') {
            let value_offset = rest.len() - after_colon.trim_start().len();
            return Some(after_name + value_offset);
        }
        search_from = after_name;
    }
    None
}

fn read_string_field(hook_event_json: &str, field_name: &str) -> Option<String> {
    let value_start = find_value_start(hook_event_json, field_name)?;
    let mut value_characters = hook_event_json[value_start..].chars();
    if value_characters.next()? != '"' {
        return None;
    }
    let mut field_value = String::new();
    while let Some(character) = value_characters.next() {
        match character {
            '"' => return Some(field_value),
            '\\' => match value_characters.next()? {
                'n' => field_value.push(' '),
                't' => field_value.push(' '),
                escaped_character => field_value.push(escaped_character),
            },
            _ => field_value.push(character),
        }
    }
    None
}

/// The raw text of an object value, braces included, skipping braces inside strings.
fn read_object_field<'json>(hook_event_json: &'json str, field_name: &str) -> Option<&'json str> {
    let value_start = find_value_start(hook_event_json, field_name)?;
    let value_text = &hook_event_json[value_start..];
    if !value_text.starts_with('{') {
        return None;
    }
    let mut brace_depth = 0;
    let mut is_inside_string = false;
    let mut is_escaped = false;
    for (byte_index, character) in value_text.char_indices() {
        if is_inside_string {
            match (is_escaped, character) {
                (true, _) => is_escaped = false,
                (false, '\\') => is_escaped = true,
                (false, '"') => is_inside_string = false,
                _ => {}
            }
            continue;
        }
        match character {
            '"' => is_inside_string = true,
            '{' => brace_depth += 1,
            '}' => {
                brace_depth -= 1;
                if brace_depth == 0 {
                    return Some(&value_text[..=byte_index]);
                }
            }
            _ => {}
        }
    }
    None
}
