//! The fields of a Claude Code hook event that Crest uses (code.claude.com/docs/en/hooks; every
//! event carries `session_id`, `cwd` and `hook_event_name` on stdin as JSON). Unknown fields are
//! ignored, so newer Claude Code versions don't break it.

use serde::Deserialize;
use serde_json::Value;

/// Set by Claude Code for every command hook: "the project root where the session started"; it
/// stays put when Claude runs `cd` or enters a worktree, unlike `cwd` (verified from docs,
/// 2026-10-24). It's only in the hook's environment, not in its JSON.
pub const CLAUDE_CODE_PROJECT_FOLDER_VARIABLE: &str = "CLAUDE_PROJECT_DIR";
/// The field the hook process adds for it; Crest's own name, so a future Claude Code field
/// can't clash with it.
const CREST_SESSION_PROJECT_FOLDER_FIELD: &str = "crest_session_project_folder";

#[derive(Debug, Default, Deserialize)]
pub struct ClaudeCodeHookEvent {
    pub session_id: String,
    pub hook_event_name: String,
    /// Where Claude is now: it follows `cd`, so it names the session only as a fallback.
    #[serde(default)]
    pub cwd: Option<String>,
    /// Where the session started (`CREST_SESSION_PROJECT_FOLDER_FIELD`, added by the hook process).
    #[serde(default)]
    pub crest_session_project_folder: Option<String>,
    /// A title the user gave the session (`--name`, `/rename`), on `SessionStart` and
    /// `UserPromptSubmit` only (verified from docs); a made-up title is only in the transcript.
    #[serde(default)]
    pub session_title: Option<String>,
    /// The session's transcript (JSONL); watched for the interrupt line while the session is busy.
    #[serde(default)]
    pub transcript_path: Option<String>,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tool_input: Option<serde_json::Value>,
    /// The same id on a tool call's `PreToolUse` and on its `PostToolUse` / `PostToolUseFailure`
    /// (verified by testing, 2026-10-27), so a step's outcome finds its step.
    #[serde(default)]
    pub tool_use_id: Option<String>,
    #[serde(default)]
    pub notification_type: Option<String>,
    /// The subagent the event comes from: on `SubagentStart` / `SubagentStop`, and on the tool
    /// events a subagent's own tool calls send under its parent's session id (verified from docs).
    #[serde(default)]
    pub agent_id: Option<String>,
    /// Why a turn ended on an API error (`StopFailure` only), e.g. "rate_limit".
    #[serde(default)]
    pub error: Option<String>,
    /// On `Stop`: Claude's final answer as raw markdown, only the turn's last message (verified
    /// by testing, 2026-10-24). On `SubagentStop` the subagent's own, on `StopFailure` the error.
    #[serde(default)]
    pub last_assistant_message: Option<String>,
}

impl ClaudeCodeHookEvent {
    /// The turn stopped because a usage or rate limit was reached.
    pub fn is_usage_limit_stop(&self) -> bool {
        self.hook_event_name == STOP_FAILURE_HOOK_EVENT && self.error.as_deref() == Some(RATE_LIMIT_STOP_FAILURE_ERROR)
    }

    /// The folder that names the session and that the alert's paths are shown from: where the
    /// session started, else (an older hook process, or Claude Code without the variable) `cwd`.
    pub fn session_project_folder(&self) -> Option<&str> {
        self.crest_session_project_folder.as_deref().or(self.cwd.as_deref())
    }
}

/// Run in the hook process before the event goes to Crest. Anything it can't read is passed on
/// unchanged, so Crest decides about it as before.
pub fn add_session_project_folder(hook_event_json: Vec<u8>, session_project_folder: Option<&str>) -> Vec<u8> {
    let Some(session_project_folder) = session_project_folder.filter(|project_folder| !project_folder.is_empty()) else {
        return hook_event_json;
    };
    let Ok(Value::Object(mut hook_event_fields)) = serde_json::from_slice::<Value>(&hook_event_json) else {
        return hook_event_json;
    };
    hook_event_fields.insert(CREST_SESSION_PROJECT_FOLDER_FIELD.to_string(), Value::from(session_project_folder));
    serde_json::to_vec(&hook_event_fields).unwrap_or(hook_event_json)
}

pub const SESSION_START_HOOK_EVENT: &str = "SessionStart";
pub const USER_PROMPT_SUBMIT_HOOK_EVENT: &str = "UserPromptSubmit";
pub const PRE_TOOL_USE_HOOK_EVENT: &str = "PreToolUse";
pub const POST_TOOL_USE_HOOK_EVENT: &str = "PostToolUse";
/// Sent instead of `PostToolUse` when the tool failed (code.claude.com/docs/en/hooks).
pub const POST_TOOL_USE_FAILURE_HOOK_EVENT: &str = "PostToolUseFailure";
pub const STOP_HOOK_EVENT: &str = "Stop";
/// Sent instead of `Stop` when the turn ends on an API error (code.claude.com/docs/en/hooks).
pub const STOP_FAILURE_HOOK_EVENT: &str = "StopFailure";
pub const SUBAGENT_START_HOOK_EVENT: &str = "SubagentStart";
pub const SUBAGENT_STOP_HOOK_EVENT: &str = "SubagentStop";
pub const NOTIFICATION_HOOK_EVENT: &str = "Notification";
pub const SESSION_END_HOOK_EVENT: &str = "SessionEnd";
/// Claude Code asks for permission: the only event whose hook waits for Crest's answer.
pub const PERMISSION_REQUEST_HOOK_EVENT: &str = "PermissionRequest";

/// `notification_type` values Crest reacts to.
pub const PERMISSION_PROMPT_NOTIFICATION: &str = "permission_prompt";
pub const IDLE_PROMPT_NOTIFICATION: &str = "idle_prompt";
/// Claude Code waited for a claude.ai usage limit to reset and continues the task by itself.
pub const QUOTA_AUTO_RESUME_FIRED_NOTIFICATION: &str = "quota_auto_resume_fired";
/// The limit reset while the computer slept; Claude Code waits for Enter instead.
pub const QUOTA_AUTO_RESUME_STALE_NOTIFICATION: &str = "quota_auto_resume_stale";
/// Claude Code stopped waiting for the reset without continuing.
pub const QUOTA_AUTO_RESUME_DISABLED_NOTIFICATION: &str = "quota_auto_resume_disabled";

/// `StopFailure`'s `error` for a reached limit; a claude.ai usage limit is one too (seen in a
/// transcript on 2026-10-06: `"error":"rate_limit"` with `quotaLimits`).
pub const RATE_LIMIT_STOP_FAILURE_ERROR: &str = "rate_limit";

/// The events Crest asks Claude Code to send, all in the background (`async`), so Claude Code
/// never waits for Crest.
pub const STATUS_HOOK_EVENT_NAMES: [&str; 11] = [
    SESSION_START_HOOK_EVENT,
    USER_PROMPT_SUBMIT_HOOK_EVENT,
    PRE_TOOL_USE_HOOK_EVENT,
    POST_TOOL_USE_HOOK_EVENT,
    POST_TOOL_USE_FAILURE_HOOK_EVENT,
    STOP_HOOK_EVENT,
    STOP_FAILURE_HOOK_EVENT,
    SUBAGENT_START_HOOK_EVENT,
    SUBAGENT_STOP_HOOK_EVENT,
    NOTIFICATION_HOOK_EVENT,
    SESSION_END_HOOK_EVENT,
];

/// Events about a tool call need a matcher in Claude Code's settings ("*" = every tool).
pub const TOOL_HOOK_EVENT_NAMES: [&str; 4] =
    [PRE_TOOL_USE_HOOK_EVENT, POST_TOOL_USE_HOOK_EVENT, POST_TOOL_USE_FAILURE_HOOK_EVENT, PERMISSION_REQUEST_HOOK_EVENT];

#[cfg(test)]
mod tests {
    use super::*;

    fn read_event(hook_event_json: &[u8]) -> ClaudeCodeHookEvent {
        serde_json::from_slice(hook_event_json).unwrap()
    }

    #[test]
    fn the_start_folder_names_the_session_after_claude_moved_with_cd() {
        let moved_event = br#"{"session_id":"a","hook_event_name":"Stop","cwd":"C:\\Projects\\Crest\\src-tauri"}"#.to_vec();
        let stamped_event = read_event(&add_session_project_folder(moved_event.clone(), Some(r"C:\Projects\Crest")));
        assert_eq!(stamped_event.session_project_folder(), Some(r"C:\Projects\Crest"));
        assert_eq!(stamped_event.cwd.as_deref(), Some(r"C:\Projects\Crest\src-tauri"));
        assert_eq!(read_event(&moved_event).session_project_folder(), Some(r"C:\Projects\Crest\src-tauri"));
    }

    #[test]
    fn an_event_without_the_folder_or_that_isnt_an_object_passes_unchanged() {
        let hook_event_json = br#"{"session_id":"a","hook_event_name":"Stop"}"#.to_vec();
        assert_eq!(add_session_project_folder(hook_event_json.clone(), None), hook_event_json);
        assert_eq!(add_session_project_folder(hook_event_json.clone(), Some("")), hook_event_json);
        for unreadable_event in [b"not json".to_vec(), b"[1, 2]".to_vec()] {
            assert_eq!(add_session_project_folder(unreadable_event.clone(), Some(r"C:\Projects\Crest")), unreadable_event);
        }
    }
}
