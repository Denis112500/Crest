//! The fields of a Claude Code hook event that Crest uses (code.claude.com/docs/en/hooks; every
//! event carries `session_id`, `cwd` and `hook_event_name` on stdin as JSON). Unknown fields are
//! ignored, so newer Claude Code versions don't break it.

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct ClaudeCodeHookEvent {
    pub session_id: String,
    pub hook_event_name: String,
    #[serde(default)]
    pub cwd: Option<String>,
    /// The session's transcript (JSONL); watched for the interrupt line while the session is busy.
    #[serde(default)]
    pub transcript_path: Option<String>,
    #[serde(default)]
    pub tool_name: Option<String>,
    #[serde(default)]
    pub tool_input: Option<serde_json::Value>,
    #[serde(default)]
    pub notification_type: Option<String>,
    /// The subagent the event comes from: on `SubagentStart` / `SubagentStop`, and on the tool
    /// events a subagent's own tool calls send under its parent's session id (verified from docs).
    #[serde(default)]
    pub agent_id: Option<String>,
    /// Why a turn ended on an API error (`StopFailure` only), e.g. "rate_limit".
    #[serde(default)]
    pub error: Option<String>,
}

impl ClaudeCodeHookEvent {
    /// The turn stopped because a usage or rate limit was reached.
    pub fn is_usage_limit_stop(&self) -> bool {
        self.hook_event_name == STOP_FAILURE_HOOK_EVENT && self.error.as_deref() == Some(RATE_LIMIT_STOP_FAILURE_ERROR)
    }
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
