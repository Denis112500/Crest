//! How a hook event changes a tracked session, and whether it is news for the user.

use std::path::PathBuf;
use std::time::Instant;

use super::TrackedSession;
use crate::activity_sources::claude_code::claude_code_api_error_line::describe_api_error;
use crate::activity_sources::claude_code::claude_code_hook_event::*;
use crate::activity_sources::claude_code::claude_code_session_list::ClaudeCodeSessionStatus;
use crate::activity_sources::claude_code::claude_code_session_title::ClaudeCodeSessionTitle;
use crate::activity_sources::claude_code::claude_code_tool_summary::{summarize_tool_use, SUBAGENT_LAUNCH_TOOL_NAMES};
use crate::activity_sources::claude_code::claude_code_turn_summary::summarize_final_answer;
use crate::activity_sources::claude_code::last_path_part::find_last_path_part;

impl TrackedSession {
    /// Returns whether the event is news for the user ("done", a new "needs your OK", a reached
    /// usage limit, an API error).
    pub fn apply_hook_event(&mut self, hook_event: &ClaudeCodeHookEvent, event_time: Instant, event_order: u64) -> bool {
        self.note_event(event_time, event_order);
        // Whether a subagent's events name its own transcript or the session's isn't documented:
        // only the session's own events set the transcript the transcript watch follows.
        if let Some(agent_id) = &hook_event.agent_id {
            if let Some(is_news) = self.apply_subagent_event(agent_id, &hook_event.hook_event_name, event_time) {
                return is_news;
            }
        } else if let Some(transcript_path) = &hook_event.transcript_path {
            self.transcript_path = Some(PathBuf::from(transcript_path));
        }
        if let Some(project_folder) = hook_event.session_project_folder() {
            self.project_folder_name = find_last_path_part(project_folder).unwrap_or_default().to_string();
        }
        if let Some(chosen_title) = hook_event.session_title.as_deref().and_then(|raw_title| ClaudeCodeSessionTitle::new(raw_title, true)) {
            self.show_session_title(chosen_title);
        }
        match (hook_event.hook_event_name.as_str(), hook_event.notification_type.as_deref()) {
            (USER_PROMPT_SUBMIT_HOOK_EVENT, _)
            | (POST_TOOL_USE_HOOK_EVENT, _)
            | (POST_TOOL_USE_FAILURE_HOOK_EVENT, _)
            | (NOTIFICATION_HOOK_EVENT, Some(QUOTA_AUTO_RESUME_FIRED_NOTIFICATION)) => {
                self.show_status_without_tool(ClaudeCodeSessionStatus::Working);
                false
            }
            (PRE_TOOL_USE_HOOK_EVENT, _) => {
                let tool_name = hook_event.tool_name.as_deref().unwrap_or_default();
                self.show_status_without_tool(ClaudeCodeSessionStatus::Working);
                self.tool_summary = Some(summarize_tool_use(tool_name, hook_event.tool_input.as_ref()));
                self.is_tool_a_subagent_launch = SUBAGENT_LAUNCH_TOOL_NAMES.contains(&tool_name);
                false
            }
            // With background subagents still running, Claude Code's turn ends but the work
            // doesn't ("Waiting for 3 background agents to finish"): the news waits for the last one.
            (STOP_HOOK_EVENT, _) => {
                self.show_status_without_tool(ClaudeCodeSessionStatus::Done);
                self.turn_summary = hook_event.last_assistant_message.as_deref().and_then(summarize_final_answer);
                !self.is_waiting_on_subagents()
            }
            (STOP_FAILURE_HOOK_EVENT, _) if hook_event.is_usage_limit_stop() => {
                let is_news = self.session_status != Some(ClaudeCodeSessionStatus::UsageLimited);
                self.show_status_without_tool(ClaudeCodeSessionStatus::UsageLimited);
                is_news
            }
            // Any other API error ends the turn too (no `Stop` follows) without its work done:
            // news, and the row says which error it was.
            (STOP_FAILURE_HOOK_EVENT, _) => {
                self.show_status_without_tool(ClaudeCodeSessionStatus::Errored);
                self.turn_summary = describe_api_error(hook_event.last_assistant_message.as_deref(), hook_event.error.as_deref());
                true
            }
            (NOTIFICATION_HOOK_EVENT, Some(QUOTA_AUTO_RESUME_STALE_NOTIFICATION))
            | (NOTIFICATION_HOOK_EVENT, Some(QUOTA_AUTO_RESUME_DISABLED_NOTIFICATION)) => {
                self.show_status_without_tool(ClaudeCodeSessionStatus::WaitingForInput);
                false
            }
            // The request comes at once, the notification about 6 s later: one piece of news.
            (PERMISSION_REQUEST_HOOK_EVENT, _) | (NOTIFICATION_HOOK_EVENT, Some(PERMISSION_PROMPT_NOTIFICATION)) => {
                let is_news = self.session_status != Some(ClaudeCodeSessionStatus::NeedsPermission);
                if hook_event.agent_id.is_some() {
                    // The main line ("Running 2 subagents") comes back once the subagent moves on.
                    self.session_status = Some(ClaudeCodeSessionStatus::NeedsPermission);
                } else {
                    self.show_status_without_tool(ClaudeCodeSessionStatus::NeedsPermission);
                }
                is_news
            }
            // Claude Code sends this about a minute after a turn ends; "done", "cancelled" and the
            // rest say more, and a second peek a minute later would only be noise.
            (NOTIFICATION_HOOK_EVENT, Some(IDLE_PROMPT_NOTIFICATION)) if !self.has_turn_ending_status() => {
                self.show_status_without_tool(ClaudeCodeSessionStatus::WaitingForInput);
                false
            }
            _ => false,
        }
    }

    /// A subagent's own events only change its count, never the main line: parallel subagents
    /// would make it jump, and one still running in the background mustn't turn "done" back into
    /// "working". Returns whether the event was news, or `None` when it wasn't handled here: a
    /// subagent's permission request still goes through the session's own rules. The last
    /// subagent stopping after the turn ended is the session's "done".
    fn apply_subagent_event(&mut self, agent_id: &str, hook_event_name: &str, event_time: Instant) -> Option<bool> {
        match hook_event_name {
            SUBAGENT_START_HOOK_EVENT => self.running_subagents.record_sign_of_life(agent_id, event_time),
            SUBAGENT_STOP_HOOK_EVENT => {
                let was_waiting_on_subagents = self.is_waiting_on_subagents();
                self.running_subagents.record_stop(agent_id);
                return Some(was_waiting_on_subagents && !self.is_waiting_on_subagents());
            }
            PRE_TOOL_USE_HOOK_EVENT | POST_TOOL_USE_HOOK_EVENT | POST_TOOL_USE_FAILURE_HOOK_EVENT => {
                self.running_subagents.record_sign_of_life(agent_id, event_time);
                // A subagent that moves on had its permission request answered (in Claude Code).
                if self.session_status == Some(ClaudeCodeSessionStatus::NeedsPermission) {
                    self.session_status = Some(ClaudeCodeSessionStatus::Working);
                }
            }
            _ => return None,
        }
        Some(false)
    }
}
