//! The pill's two answers to a permission request: reading the action the alert's buttons send,
//! and the JSON the waiting hook prints for Claude Code (code.claude.com/docs/en/hooks).

use serde_json::json;

use crate::activity_sources::claude_code::claude_code_hook_event::PERMISSION_REQUEST_HOOK_EVENT;
use crate::ipc_channel_names::{CLAUDE_CODE_ALLOW_PERMISSION_ACTION_PREFIX, CLAUDE_CODE_DENY_PERMISSION_ACTION_PREFIX};

/// Claude reads this as the reason for the deny.
const PILL_DENY_MESSAGE: &str = "The user denied this from Crest.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClaudeCodePermissionAnswer {
    Allow,
    Deny,
}

/// `"allow-permission:7"` → (Allow, 7); anything else is `None`.
pub fn parse_permission_action(activity_action: &str) -> Option<(ClaudeCodePermissionAnswer, u64)> {
    let (permission_answer, request_id_text) =
        if let Some(request_id_text) = activity_action.strip_prefix(CLAUDE_CODE_ALLOW_PERMISSION_ACTION_PREFIX) {
            (ClaudeCodePermissionAnswer::Allow, request_id_text)
        } else {
            (ClaudeCodePermissionAnswer::Deny, activity_action.strip_prefix(CLAUDE_CODE_DENY_PERMISSION_ACTION_PREFIX)?)
        };
    Some((permission_answer, request_id_text.parse().ok()?))
}

/// A deny also stops Claude (`interrupt`), like Claude Code's own "No": the pill can't ask what
/// to do instead, so the turn ends and the session waits for the user (decided 2026-10-10).
pub fn build_permission_hook_reply(permission_answer: ClaudeCodePermissionAnswer) -> String {
    let permission_decision = match permission_answer {
        ClaudeCodePermissionAnswer::Allow => json!({ "behavior": "allow" }),
        ClaudeCodePermissionAnswer::Deny => json!({ "behavior": "deny", "message": PILL_DENY_MESSAGE, "interrupt": true }),
    };
    json!({ "hookSpecificOutput": { "hookEventName": PERMISSION_REQUEST_HOOK_EVENT, "decision": permission_decision } })
        .to_string()
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    #[test]
    fn button_actions_carry_the_answer_and_the_request_id() {
        assert_eq!(parse_permission_action("allow-permission:7"), Some((ClaudeCodePermissionAnswer::Allow, 7)));
        assert_eq!(parse_permission_action("deny-permission:12"), Some((ClaudeCodePermissionAnswer::Deny, 12)));
        assert_eq!(parse_permission_action("allow-permission:"), None);
        assert_eq!(parse_permission_action("allow-permission:-1"), None);
        assert_eq!(parse_permission_action("toggle-play-pause"), None);
    }

    #[test]
    fn allow_and_deny_become_claude_codes_permission_decisions() {
        let allow_reply: Value = serde_json::from_str(&build_permission_hook_reply(ClaudeCodePermissionAnswer::Allow)).unwrap();
        assert_eq!(allow_reply["hookSpecificOutput"]["hookEventName"], "PermissionRequest");
        assert_eq!(allow_reply["hookSpecificOutput"]["decision"], json!({ "behavior": "allow" }));
        let deny_reply: Value = serde_json::from_str(&build_permission_hook_reply(ClaudeCodePermissionAnswer::Deny)).unwrap();
        let deny_decision = &deny_reply["hookSpecificOutput"]["decision"];
        assert_eq!((&deny_decision["behavior"], &deny_decision["interrupt"]), (&json!("deny"), &json!(true)));
        assert!(deny_decision["message"].as_str().is_some_and(|deny_message| !deny_message.is_empty()));
    }
}
