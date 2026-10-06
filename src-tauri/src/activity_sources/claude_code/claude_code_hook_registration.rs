//! Adds Crest's hooks to Claude Code's settings (as JSON) and removes them again, leaving every
//! other setting and every other hook alone. Pure, so the rules are unit-tested; reading and
//! writing the file is `claude_code_settings_file.rs`'s job.

use std::path::Path;

use serde_json::{json, Map, Value};

use crate::activity_sources::claude_code::claude_code_hook_event::{STATUS_HOOK_EVENT_NAMES, TOOL_HOOK_EVENT_NAMES};
use crate::backend_constants::CLAUDE_CODE_HOOK_ARGUMENT;

const HOOKS_SETTING_NAME: &str = "hooks";
const EVERY_TOOL_MATCHER: &str = "*";

/// `"C:/Users/…/crest.exe" --claude-code-hook`: quoted for paths with spaces, forward slashes
/// because Claude Code runs hook commands through a shell where backslashes can be escapes.
pub fn build_crest_hook_command(crest_executable_path: &Path) -> String {
    let crest_executable_text = crest_executable_path.to_string_lossy().replace('\\', "/");
    format!("\"{crest_executable_text}\" {CLAUDE_CODE_HOOK_ARGUMENT}")
}

/// Crest's entries, the way they're added to `settings.json` (also shown in the settings window).
pub fn build_crest_hooks(crest_hook_command: &str) -> Map<String, Value> {
    let mut crest_hooks = Map::new();
    for hook_event_name in STATUS_HOOK_EVENT_NAMES {
        let crest_hook_handler = json!({ "type": "command", "command": crest_hook_command, "async": true });
        let mut matcher_group = json!({ "hooks": [crest_hook_handler] });
        if TOOL_HOOK_EVENT_NAMES.contains(&hook_event_name) {
            matcher_group["matcher"] = json!(EVERY_TOOL_MATCHER);
        }
        crest_hooks.insert(hook_event_name.to_string(), json!([matcher_group]));
    }
    crest_hooks
}

/// Replaces Crest's earlier entries (e.g. from another install path) with the current ones.
pub fn add_crest_hooks(claude_code_settings: &mut Value, crest_hook_command: &str) -> Result<(), String> {
    remove_crest_hooks(claude_code_settings);
    let settings_object =
        claude_code_settings.as_object_mut().ok_or("Claude Code's settings file doesn't hold a JSON object")?;
    let hooks_by_event_name = settings_object.entry(HOOKS_SETTING_NAME).or_insert_with(|| json!({}));
    let hooks_by_event_name = hooks_by_event_name.as_object_mut().ok_or("the \"hooks\" setting isn't a JSON object")?;
    for (hook_event_name, crest_matcher_groups) in build_crest_hooks(crest_hook_command) {
        let event_matcher_groups = hooks_by_event_name.entry(hook_event_name).or_insert_with(|| json!([]));
        let event_matcher_groups = event_matcher_groups.as_array_mut().ok_or("a hook event's setting isn't a list")?;
        event_matcher_groups.extend(crest_matcher_groups.as_array().cloned().unwrap_or_default());
    }
    Ok(())
}

/// Removes every hook whose command calls Crest, then any group, event or `hooks` setting that
/// became empty because of it. Leaves the settings untouched if they don't look as expected.
pub fn remove_crest_hooks(claude_code_settings: &mut Value) {
    let Some(settings_object) = claude_code_settings.as_object_mut() else {
        return;
    };
    let Some(hooks_by_event_name) = settings_object.get_mut(HOOKS_SETTING_NAME).and_then(Value::as_object_mut) else {
        return;
    };
    for event_matcher_groups in hooks_by_event_name.values_mut() {
        let Some(event_matcher_groups) = event_matcher_groups.as_array_mut() else {
            continue;
        };
        for matcher_group in event_matcher_groups.iter_mut() {
            if let Some(hook_handlers) = matcher_group.get_mut("hooks").and_then(Value::as_array_mut) {
                hook_handlers.retain(|hook_handler| !is_crest_hook_handler(hook_handler));
            }
        }
        event_matcher_groups.retain(|matcher_group| {
            matcher_group.get("hooks").and_then(Value::as_array).is_none_or(|hook_handlers| !hook_handlers.is_empty())
        });
    }
    hooks_by_event_name.retain(|_, event_matcher_groups| event_matcher_groups.as_array().is_none_or(|groups| !groups.is_empty()));
    if hooks_by_event_name.is_empty() {
        // `shift_remove`, not `remove`: with `preserve_order`, `remove` moves the last key into the
        // gap, which reordered the user's file (verified by testing on 2026-10-07).
        settings_object.shift_remove(HOOKS_SETTING_NAME);
    }
}

fn is_crest_hook_handler(hook_handler: &Value) -> bool {
    hook_handler
        .get("command")
        .and_then(Value::as_str)
        .is_some_and(|hook_command| hook_command.contains(CLAUDE_CODE_HOOK_ARGUMENT))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CREST_HOOK_COMMAND: &str = "\"C:/Crest/crest.exe\" --claude-code-hook";

    #[test]
    fn the_hook_command_is_quoted_with_forward_slashes() {
        let crest_executable_path = Path::new(r"C:\Users\someone\Dynamic Island\crest.exe");
        assert_eq!(
            build_crest_hook_command(crest_executable_path),
            "\"C:/Users/someone/Dynamic Island/crest.exe\" --claude-code-hook"
        );
    }

    #[test]
    fn adding_keeps_other_settings_and_hooks_and_removing_restores_them() {
        let original_settings = json!({
            "model": "opus",
            "hooks": { "Stop": [{ "hooks": [{ "type": "command", "command": "notify-me.sh" }] }] }
        });
        let mut claude_code_settings = original_settings.clone();
        add_crest_hooks(&mut claude_code_settings, CREST_HOOK_COMMAND).unwrap();
        assert_eq!(claude_code_settings["model"], "opus");
        assert_eq!(claude_code_settings["hooks"]["Stop"].as_array().unwrap().len(), 2);
        assert_eq!(claude_code_settings["hooks"]["PreToolUse"][0]["matcher"], "*");
        assert_eq!(claude_code_settings["hooks"]["SessionStart"][0]["hooks"][0]["async"], true);
        remove_crest_hooks(&mut claude_code_settings);
        assert_eq!(claude_code_settings, original_settings);
    }

    #[test]
    fn adding_twice_leaves_one_set_and_removing_from_scratch_leaves_no_hooks_setting() {
        let mut claude_code_settings = json!({});
        add_crest_hooks(&mut claude_code_settings, "\"C:/old/crest.exe\" --claude-code-hook").unwrap();
        add_crest_hooks(&mut claude_code_settings, CREST_HOOK_COMMAND).unwrap();
        assert_eq!(claude_code_settings["hooks"]["Stop"].as_array().unwrap().len(), 1);
        assert_eq!(claude_code_settings["hooks"]["Stop"][0]["hooks"][0]["command"], CREST_HOOK_COMMAND);
        remove_crest_hooks(&mut claude_code_settings);
        assert_eq!(claude_code_settings, json!({}));
    }

    #[test]
    fn removing_the_hooks_keeps_the_order_of_the_other_settings() {
        let mut claude_code_settings = json!({
            "model": "opus",
            "hooks": { "SessionStart": [{ "hooks": [{ "type": "command", "command": CREST_HOOK_COMMAND }] }] },
            "theme": "dark",
            "switchModelsOnFlag": true
        });
        remove_crest_hooks(&mut claude_code_settings);
        let remaining_setting_names: Vec<&String> = claude_code_settings.as_object().unwrap().keys().collect();
        assert_eq!(remaining_setting_names, ["model", "theme", "switchModelsOnFlag"]);
    }

    #[test]
    fn settings_that_are_not_an_object_are_refused() {
        assert!(add_crest_hooks(&mut json!([1, 2]), CREST_HOOK_COMMAND).is_err());
    }
}
