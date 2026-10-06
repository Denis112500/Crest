//! The fields of a Claude Code hook event that Crest uses (code.claude.com/docs/en/hooks; every
//! event carries `session_id`, `cwd` and `hook_event_name` on stdin as JSON). Unknown fields are
//! ignored, so newer Claude Code versions don't break it.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ClaudeCodeHookEvent {
    pub session_id: String,
    pub hook_event_name: String,
    #[serde(default)]
    pub cwd: Option<String>,
    /// The session's transcript (JSONL); watched for the interrupt line while the session works.
    #[serde(default)]
    pub transcript_path: Option<String>,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tool_input: Option<serde_json::Value>,
    #[serde(default)]
    pub notification_type: Option<String>,
}

pub const SESSION_START_HOOK_EVENT: &str = "SessionStart";
pub const USER_PROMPT_SUBMIT_HOOK_EVENT: &str = "UserPromptSubmit";
pub const PRE_TOOL_USE_HOOK_EVENT: &str = "PreToolUse";
pub const POST_TOOL_USE_HOOK_EVENT: &str = "PostToolUse";
pub const STOP_HOOK_EVENT: &str = "Stop";
pub const NOTIFICATION_HOOK_EVENT: &str = "Notification";
pub const SESSION_END_HOOK_EVENT: &str = "SessionEnd";

/// `notification_type` values Crest reacts to.
pub const PERMISSION_PROMPT_NOTIFICATION: &str = "permission_prompt";
pub const IDLE_PROMPT_NOTIFICATION: &str = "idle_prompt";

/// The events Crest asks Claude Code to send, all in the background (`async`), so Claude Code
/// never waits for Crest.
pub const STATUS_HOOK_EVENT_NAMES: [&str; 7] = [
    SESSION_START_HOOK_EVENT,
    USER_PROMPT_SUBMIT_HOOK_EVENT,
    PRE_TOOL_USE_HOOK_EVENT,
    POST_TOOL_USE_HOOK_EVENT,
    STOP_HOOK_EVENT,
    NOTIFICATION_HOOK_EVENT,
    SESSION_END_HOOK_EVENT,
];

/// Events about a tool call need a matcher in Claude Code's settings ("*" = every tool).
pub const TOOL_HOOK_EVENT_NAMES: [&str; 2] = [PRE_TOOL_USE_HOOK_EVENT, POST_TOOL_USE_HOOK_EVENT];
