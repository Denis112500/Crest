//! What a change to Claude Code's settings does to its hooks, for the settings window to show
//! before anything is written: a plain summary and the changed lines of the `hooks` block. Pure,
//! so it's unit-tested; reading the file is `claude_code_settings_file.rs`'s job.

use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::activity_sources::claude_code::claude_code_hook_registration::{is_crest_hook_handler, HOOKS_SETTING_NAME};
use crate::backend_constants::CLAUDE_CODE_HOOKS_DIFF_CONTEXT_LINE_COUNT;
use crate::text_line_diff::{diff_text_lines, TextLineDiffEntry};

/// Claude Code's switch that turns every hook off, Crest's included.
const DISABLE_ALL_HOOKS_SETTING_NAME: &str = "disableAllHooks";
/// Stands in for each of the user's own hooks in the shown lines: a hook's command can hold a
/// token, and the page never gets one.
const OWN_HOOK_PLACEHOLDER: &str = "(your own hook, unchanged)";

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeHooksChangePreview {
    pub added_crest_hook_count: usize,
    pub removed_crest_hook_count: usize,
    /// Settings other than `hooks` ("model", "env", …): only counted, never shown, because
    /// `env` can hold API keys.
    pub kept_other_setting_count: usize,
    /// Hooks that don't run Crest; Crest never changes them.
    pub kept_own_hook_count: usize,
    /// `disableAllHooks` is on: Claude Code runs no hook at all, so Crest would see nothing.
    pub is_every_hook_disabled: bool,
    pub hooks_diff_lines: Vec<TextLineDiffEntry>,
}

pub fn preview_claude_code_hooks_change(settings_before: &Value, settings_after: &Value) -> ClaudeCodeHooksChangePreview {
    let crest_hooks_before = list_hook_handlers(settings_before, is_crest_hook_handler);
    let mut crest_hooks_not_matched = crest_hooks_before.clone();
    let mut added_crest_hook_count = 0;
    for crest_hook_after in list_hook_handlers(settings_after, is_crest_hook_handler) {
        match crest_hooks_not_matched.iter().position(|crest_hook_before| *crest_hook_before == crest_hook_after) {
            Some(matched_position) => {
                crest_hooks_not_matched.remove(matched_position);
            }
            None => added_crest_hook_count += 1,
        }
    }
    let settings_object_after = settings_after.as_object();
    ClaudeCodeHooksChangePreview {
        added_crest_hook_count,
        removed_crest_hook_count: crest_hooks_not_matched.len(),
        kept_other_setting_count: settings_object_after
            .map_or(0, |settings_object| settings_object.keys().filter(|setting_name| *setting_name != HOOKS_SETTING_NAME).count()),
        kept_own_hook_count: list_hook_handlers(settings_after, |hook_handler| !is_crest_hook_handler(hook_handler)).len(),
        is_every_hook_disabled: settings_after.get(DISABLE_ALL_HOOKS_SETTING_NAME) == Some(&Value::Bool(true)),
        hooks_diff_lines: diff_text_lines(
            &shown_hooks_text(settings_before),
            &shown_hooks_text(settings_after),
            CLAUDE_CODE_HOOKS_DIFF_CONTEXT_LINE_COUNT,
        ),
    }
}

/// Each matching hook with the event it's under; a part that isn't shaped as Claude Code
/// expects is skipped, as Claude Code itself would.
fn list_hook_handlers(claude_code_settings: &Value, should_list: impl Fn(&Value) -> bool) -> Vec<(String, Value)> {
    let Some(hooks_by_event_name) = claude_code_settings.get(HOOKS_SETTING_NAME).and_then(Value::as_object) else {
        return Vec::new();
    };
    let mut listed_hook_handlers = Vec::new();
    for (hook_event_name, event_matcher_groups) in hooks_by_event_name {
        for matcher_group in event_matcher_groups.as_array().into_iter().flatten() {
            for hook_handler in matcher_group.get("hooks").and_then(Value::as_array).into_iter().flatten() {
                if should_list(hook_handler) {
                    listed_hook_handlers.push((hook_event_name.clone(), hook_handler.clone()));
                }
            }
        }
    }
    listed_hook_handlers
}

/// The `hooks` block as it's written to the file, with the user's own hooks replaced by the
/// placeholder.
fn shown_hooks_text(claude_code_settings: &Value) -> String {
    let mut shown_hooks = claude_code_settings.get(HOOKS_SETTING_NAME).cloned().unwrap_or_else(|| Value::Object(Map::new()));
    for event_matcher_groups in shown_hooks.as_object_mut().into_iter().flat_map(|hooks| hooks.values_mut()) {
        for matcher_group in event_matcher_groups.as_array_mut().into_iter().flatten() {
            for hook_handler in matcher_group.get_mut("hooks").and_then(Value::as_array_mut).into_iter().flatten() {
                if !is_crest_hook_handler(hook_handler) {
                    *hook_handler = json!(OWN_HOOK_PLACEHOLDER);
                }
            }
        }
    }
    serde_json::to_string_pretty(&json!({ HOOKS_SETTING_NAME: shown_hooks })).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity_sources::claude_code::claude_code_hook_registration::{add_crest_hooks, remove_crest_hooks};

    const CREST_HOOK_COMMAND: &str = "\"C:/Crest/crest.exe\" --claude-code-hook";

    fn user_settings() -> Value {
        json!({
            "model": "opus",
            "env": { "ANTHROPIC_API_KEY": "sk-secret" },
            "hooks": { "Stop": [{ "hooks": [{ "type": "command", "command": "notify --token abc123" }] }] }
        })
    }

    fn shown_text(hooks_change_preview: &ClaudeCodeHooksChangePreview) -> String {
        serde_json::to_string(&hooks_change_preview.hooks_diff_lines).unwrap()
    }

    #[test]
    fn switching_on_counts_crests_hooks_and_keeps_the_rest() {
        let settings_before = user_settings();
        let mut settings_after = settings_before.clone();
        add_crest_hooks(&mut settings_after, CREST_HOOK_COMMAND, true).unwrap();
        let hooks_change_preview = preview_claude_code_hooks_change(&settings_before, &settings_after);
        assert_eq!(hooks_change_preview.added_crest_hook_count, 12);
        assert_eq!(hooks_change_preview.removed_crest_hook_count, 0);
        assert_eq!(hooks_change_preview.kept_other_setting_count, 2);
        assert_eq!(hooks_change_preview.kept_own_hook_count, 1);
        assert!(!hooks_change_preview.is_every_hook_disabled);
        assert!(hooks_change_preview.hooks_diff_lines.iter().any(|diff_line| matches!(diff_line, TextLineDiffEntry::Added { .. })));
        assert!(!hooks_change_preview.hooks_diff_lines.iter().any(|diff_line| matches!(diff_line, TextLineDiffEntry::Removed { .. })));
    }

    #[test]
    fn the_shown_lines_never_hold_other_settings_or_the_users_own_hook_commands() {
        let settings_before = user_settings();
        let mut settings_after = settings_before.clone();
        add_crest_hooks(&mut settings_after, CREST_HOOK_COMMAND, true).unwrap();
        let shown_lines = shown_text(&preview_claude_code_hooks_change(&settings_before, &settings_after));
        assert!(!shown_lines.contains("sk-secret"));
        assert!(!shown_lines.contains("abc123"));
        assert!(!shown_lines.contains("opus"));
        assert!(shown_lines.contains(OWN_HOOK_PLACEHOLDER));
        assert!(shown_lines.contains("--claude-code-hook"));
    }

    #[test]
    fn re_pointing_at_another_crest_counts_as_removed_and_added_and_no_change_counts_nothing() {
        let mut settings_before = user_settings();
        add_crest_hooks(&mut settings_before, "\"C:/old/crest.exe\" --claude-code-hook", false).unwrap();
        let mut settings_after = settings_before.clone();
        add_crest_hooks(&mut settings_after, CREST_HOOK_COMMAND, false).unwrap();
        let hooks_change_preview = preview_claude_code_hooks_change(&settings_before, &settings_after);
        assert_eq!((hooks_change_preview.added_crest_hook_count, hooks_change_preview.removed_crest_hook_count), (11, 11));

        let unchanged_preview = preview_claude_code_hooks_change(&settings_after, &settings_after);
        assert_eq!((unchanged_preview.added_crest_hook_count, unchanged_preview.removed_crest_hook_count), (0, 0));
        assert!(unchanged_preview.hooks_diff_lines.is_empty());
    }

    #[test]
    fn switching_answering_off_removes_only_the_permission_hook() {
        let mut settings_before = json!({});
        add_crest_hooks(&mut settings_before, CREST_HOOK_COMMAND, true).unwrap();
        let mut settings_after = settings_before.clone();
        add_crest_hooks(&mut settings_after, CREST_HOOK_COMMAND, false).unwrap();
        let hooks_change_preview = preview_claude_code_hooks_change(&settings_before, &settings_after);
        assert_eq!((hooks_change_preview.added_crest_hook_count, hooks_change_preview.removed_crest_hook_count), (0, 1));
        assert!(shown_text(&hooks_change_preview).contains("PermissionRequest"));
    }

    #[test]
    fn switching_off_removes_every_crest_hook_and_notices_disabled_hooks() {
        let mut settings_before = user_settings();
        settings_before["disableAllHooks"] = json!(true);
        add_crest_hooks(&mut settings_before, CREST_HOOK_COMMAND, true).unwrap();
        let mut settings_after = settings_before.clone();
        remove_crest_hooks(&mut settings_after);
        let hooks_change_preview = preview_claude_code_hooks_change(&settings_before, &settings_after);
        assert_eq!((hooks_change_preview.added_crest_hook_count, hooks_change_preview.removed_crest_hook_count), (0, 12));
        assert_eq!(hooks_change_preview.kept_own_hook_count, 1);
        assert!(hooks_change_preview.is_every_hook_disabled);
    }
}
