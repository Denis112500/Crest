//! One Claude Code session as the tracker keeps it, and how a hook event, an interrupt or an
//! answer from the pill changes it. Split from `claude_code_session_tracker.rs` (code review
//! before 0.4.0), which keeps the sessions, their order, the news count and the timeouts.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::activity_sources::claude_code::claude_code_hook_event::*;
use crate::activity_sources::claude_code::claude_code_permission_answer::ClaudeCodePermissionAnswer;
use crate::activity_sources::claude_code::claude_code_running_subagents::ClaudeCodeRunningSubagents;
use crate::activity_sources::claude_code::claude_code_session_list::{
    ClaudeCodeSessionRow, ClaudeCodeSessionStatus, ListableClaudeCodeSession,
};
use crate::activity_sources::claude_code::claude_code_tool_summary::{summarize_tool_use, SUBAGENT_LAUNCH_TOOL_NAMES};
use crate::activity_sources::claude_code::claude_code_usage_limit_wait::ClaudeCodeUsageLimitWait;
use crate::activity_sources::claude_code::last_path_part::find_last_path_part;

pub struct TrackedSession {
    project_folder_name: String,
    pub transcript_path: Option<PathBuf>,
    /// `None` right after `SessionStart`: an idle new session has nothing to show yet.
    session_status: Option<ClaudeCodeSessionStatus>,
    tool_summary: Option<String>,
    /// The main tool call starts subagents, so the line counts them instead of naming the tool.
    is_tool_a_subagent_launch: bool,
    running_subagents: ClaudeCodeRunningSubagents,
    usage_limit_wait: Option<ClaudeCodeUsageLimitWait>,
    pub last_event_time: Instant,
    last_event_order: u64,
}

impl TrackedSession {
    /// Crest may start in the middle of a session, so any event can be the first one seen.
    pub fn new(event_time: Instant, event_order: u64) -> Self {
        Self {
            project_folder_name: String::new(),
            transcript_path: None,
            session_status: None,
            tool_summary: None,
            is_tool_a_subagent_launch: false,
            running_subagents: ClaudeCodeRunningSubagents::default(),
            usage_limit_wait: None,
            last_event_time: event_time,
            last_event_order: event_order,
        }
    }

    /// Returns whether the event is news for the user ("done", a new "needs your OK", a reached
    /// usage limit).
    pub fn apply_hook_event(&mut self, hook_event: &ClaudeCodeHookEvent, event_time: Instant, event_order: u64) -> bool {
        self.note_event(event_time, event_order);
        // Whether a subagent's events name its own transcript or the session's isn't documented:
        // only the session's own events set the transcript the interrupt watch follows.
        if let Some(agent_id) = &hook_event.agent_id {
            if let Some(is_news) = self.apply_subagent_event(agent_id, &hook_event.hook_event_name, event_time) {
                return is_news;
            }
        } else if let Some(transcript_path) = &hook_event.transcript_path {
            self.transcript_path = Some(PathBuf::from(transcript_path));
        }
        if let Some(working_folder) = &hook_event.cwd {
            self.project_folder_name = find_last_path_part(working_folder).unwrap_or_default().to_string();
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
                !self.is_waiting_on_subagents()
            }
            (STOP_FAILURE_HOOK_EVENT, _) if hook_event.is_usage_limit_stop() => {
                let is_news = self.session_status != Some(ClaudeCodeSessionStatus::UsageLimited);
                self.show_status_without_tool(ClaudeCodeSessionStatus::UsageLimited);
                is_news
            }
            // Any other API error ends the turn too (no `Stop` follows); Claude Code waits for input.
            (STOP_FAILURE_HOOK_EVENT, _)
            | (NOTIFICATION_HOOK_EVENT, Some(QUOTA_AUTO_RESUME_STALE_NOTIFICATION))
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

    /// The turn ended in a way that already tells the user what happened.
    fn has_turn_ending_status(&self) -> bool {
        matches!(
            self.session_status,
            Some(
                ClaudeCodeSessionStatus::Done
                    | ClaudeCodeSessionStatus::Cancelled
                    | ClaudeCodeSessionStatus::Denied
                    | ClaudeCodeSessionStatus::UsageLimited
            )
        )
    }

    /// The turn has ended, but subagents it started in the background still run.
    fn is_waiting_on_subagents(&self) -> bool {
        self.session_status == Some(ClaudeCodeSessionStatus::Done) && self.running_subagents.running_count() > 0
    }

    /// The reset time read from the transcript after the session stopped on a usage limit.
    pub fn show_usage_limit_reset(&mut self, resets_at_unix_seconds: u64, time_until_reset: Duration, current_time: Instant) {
        if self.session_status == Some(ClaudeCodeSessionStatus::UsageLimited) {
            self.usage_limit_wait = Some(ClaudeCodeUsageLimitWait::new(resets_at_unix_seconds, time_until_reset, current_time));
        }
    }

    /// Working (its own turn or only its subagents), or waiting for the user's OK.
    pub fn is_busy(&self) -> bool {
        self.session_status.is_some_and(ClaudeCodeSessionStatus::is_busy) || self.is_waiting_on_subagents()
    }

    /// The user cancelled the turn or denied in Claude Code (`stopped_status` says which).
    pub fn show_interrupt(&mut self, stopped_status: ClaudeCodeSessionStatus, event_time: Instant, event_order: u64) {
        self.show_status_without_tool(stopped_status);
        self.running_subagents.forget_all();
        self.note_event(event_time, event_order);
    }

    /// The user stopped the background subagents. A turn that was only waiting for them was
    /// cancelled: its work didn't finish. Esc, Ctrl+C and Ctrl+X Ctrl+K all write the same line
    /// then (seen 2026-10-23), so which key it was can't be told.
    pub fn show_subagents_stopped(&mut self, event_time: Instant, event_order: u64) {
        if self.is_waiting_on_subagents() {
            self.show_status_without_tool(ClaudeCodeSessionStatus::Cancelled);
        }
        self.running_subagents.forget_all();
        self.note_event(event_time, event_order);
    }

    /// An allow lets the tool run; a deny ends the turn (no `Stop` follows an interrupted one).
    pub fn show_permission_answer(&mut self, permission_answer: ClaudeCodePermissionAnswer, event_time: Instant, event_order: u64) {
        self.show_status_without_tool(match permission_answer {
            ClaudeCodePermissionAnswer::Allow => ClaudeCodeSessionStatus::Working,
            ClaudeCodePermissionAnswer::Deny => ClaudeCodeSessionStatus::Denied,
        });
        self.note_event(event_time, event_order);
    }

    /// When the session goes for lack of news: the silence timeout after its last event, or after
    /// the reset of a usage limit it waits for, whichever is later.
    pub fn silence_deadline(&self, silence_timeout: Duration) -> Instant {
        let after_last_event = self.last_event_time + silence_timeout;
        match self.usage_limit_wait {
            Some(usage_limit_wait) => usage_limit_wait.silence_deadline(after_last_event, silence_timeout),
            None => after_last_event,
        }
    }

    pub fn next_subagent_silence_deadline(&self, silence_timeout: Duration) -> Option<Instant> {
        self.running_subagents.next_silence_deadline(silence_timeout)
    }

    pub fn forget_subagents_silent_for(&mut self, silence_timeout: Duration, current_time: Instant) {
        self.running_subagents.forget_silent_for(silence_timeout, current_time);
    }

    /// Its row in the pill's list, `None` while there is nothing to show yet. A session with a
    /// request still waiting shows "needs your OK" whatever it sent since; one whose turn ended
    /// while background subagents still run shows as working on them ("Running 3 subagents").
    pub fn listable_session(&self, is_waiting_for_an_answer: bool) -> Option<ListableClaudeCodeSession> {
        let is_waiting_on_subagents = !is_waiting_for_an_answer && self.is_waiting_on_subagents();
        let session_status = match (is_waiting_for_an_answer, is_waiting_on_subagents) {
            (true, _) => ClaudeCodeSessionStatus::NeedsPermission,
            (false, true) => ClaudeCodeSessionStatus::Working,
            (false, false) => self.session_status?,
        };
        let (tool_summary, unmentioned_subagent_count) = if is_waiting_for_an_answer {
            (None, self.running_subagents.running_count())
        } else {
            let is_line_about_subagents = self.is_tool_a_subagent_launch || is_waiting_on_subagents;
            self.running_subagents.describe_with_line(self.tool_summary.as_deref(), is_line_about_subagents)
        };
        Some(ListableClaudeCodeSession {
            session_row: ClaudeCodeSessionRow {
                session_status,
                tool_summary,
                project_folder_name: self.project_folder_name.clone(),
                unmentioned_subagent_count,
                usage_limit_resets_at: self.usage_limit_wait.map(|usage_limit_wait| usage_limit_wait.resets_at_unix_seconds),
            },
            last_event_order: self.last_event_order,
        })
    }

    /// For the silence timeout (time) and the list's "newest first" (order).
    fn note_event(&mut self, event_time: Instant, event_order: u64) {
        self.last_event_time = event_time;
        self.last_event_order = event_order;
    }

    /// Leaving the usage-limited state also forgets when the limit resets.
    fn show_status_without_tool(&mut self, session_status: ClaudeCodeSessionStatus) {
        if session_status != ClaudeCodeSessionStatus::UsageLimited {
            self.usage_limit_wait = None;
        }
        self.session_status = Some(session_status);
        self.tool_summary = None;
        self.is_tool_a_subagent_launch = false;
    }
}
