//! One Claude Code session as the tracker keeps it, and how a hook event, an interrupt or an
//! answer from the pill changes it. Split from `claude_code_session_tracker.rs` (code review
//! before 0.4.0), which keeps the sessions, their order, the news count and the timeouts.

use std::path::PathBuf;
use std::time::Instant;

use crate::activity_sources::claude_code::claude_code_hook_event::*;
use crate::activity_sources::claude_code::claude_code_permission_answer::ClaudeCodePermissionAnswer;
use crate::activity_sources::claude_code::claude_code_session_list::{
    ClaudeCodeSessionRow, ClaudeCodeSessionStatus, ListableClaudeCodeSession,
};
use crate::activity_sources::claude_code::claude_code_tool_summary::summarize_tool_use;
use crate::activity_sources::claude_code::last_path_part::find_last_path_part;

pub struct TrackedSession {
    project_folder_name: String,
    pub transcript_path: Option<PathBuf>,
    /// `None` right after `SessionStart`: an idle new session has nothing to show yet.
    session_status: Option<ClaudeCodeSessionStatus>,
    tool_summary: Option<String>,
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
            last_event_time: event_time,
            last_event_order: event_order,
        }
    }

    /// Returns whether the event is news for the user ("done", or a new "needs your OK").
    pub fn apply_hook_event(&mut self, hook_event: &ClaudeCodeHookEvent, event_time: Instant, event_order: u64) -> bool {
        if let Some(transcript_path) = &hook_event.transcript_path {
            self.transcript_path = Some(PathBuf::from(transcript_path));
        }
        if let Some(working_folder) = &hook_event.cwd {
            self.project_folder_name = find_last_path_part(working_folder).unwrap_or_default().to_string();
        }
        self.note_event(event_time, event_order);
        match (hook_event.hook_event_name.as_str(), hook_event.notification_type.as_deref()) {
            (USER_PROMPT_SUBMIT_HOOK_EVENT, _) | (POST_TOOL_USE_HOOK_EVENT, _) | (POST_TOOL_USE_FAILURE_HOOK_EVENT, _) => {
                self.show_status_without_tool(ClaudeCodeSessionStatus::Working);
                false
            }
            (PRE_TOOL_USE_HOOK_EVENT, _) => {
                self.session_status = Some(ClaudeCodeSessionStatus::Working);
                let tool_name = hook_event.tool_name.as_deref().unwrap_or_default();
                self.tool_summary = Some(summarize_tool_use(tool_name, hook_event.tool_input.as_ref()));
                false
            }
            (STOP_HOOK_EVENT, _) => {
                self.show_status_without_tool(ClaudeCodeSessionStatus::Done);
                true
            }
            // The request comes at once, the notification about 6 s later: one piece of news.
            (PERMISSION_REQUEST_HOOK_EVENT, _) | (NOTIFICATION_HOOK_EVENT, Some(PERMISSION_PROMPT_NOTIFICATION)) => {
                let is_news = self.session_status != Some(ClaudeCodeSessionStatus::NeedsPermission);
                self.show_status_without_tool(ClaudeCodeSessionStatus::NeedsPermission);
                is_news
            }
            // Claude Code sends this about a minute after "done"; "done" says more, and a second
            // peek a minute later would only be noise.
            (NOTIFICATION_HOOK_EVENT, Some(IDLE_PROMPT_NOTIFICATION))
                if self.session_status != Some(ClaudeCodeSessionStatus::Done) =>
            {
                self.show_status_without_tool(ClaudeCodeSessionStatus::WaitingForInput);
                false
            }
            _ => false,
        }
    }

    /// Working, or waiting for the user's OK.
    pub fn is_busy(&self) -> bool {
        self.session_status.is_some_and(ClaudeCodeSessionStatus::is_busy)
    }

    /// The user interrupted the turn or denied in Claude Code: it waits for input now.
    pub fn show_interrupt(&mut self, event_time: Instant, event_order: u64) {
        self.show_status_without_tool(ClaudeCodeSessionStatus::WaitingForInput);
        self.note_event(event_time, event_order);
    }

    /// An allow lets the tool run; a deny ends the turn (no `Stop` follows an interrupted one).
    pub fn show_permission_answer(&mut self, permission_answer: ClaudeCodePermissionAnswer, event_time: Instant, event_order: u64) {
        self.show_status_without_tool(match permission_answer {
            ClaudeCodePermissionAnswer::Allow => ClaudeCodeSessionStatus::Working,
            ClaudeCodePermissionAnswer::Deny => ClaudeCodeSessionStatus::WaitingForInput,
        });
        self.note_event(event_time, event_order);
    }

    /// Its row in the pill's list, `None` while there is nothing to show yet. A session with a
    /// request still waiting shows "needs your OK" whatever it sent since.
    pub fn listable_session(&self, is_waiting_for_an_answer: bool) -> Option<ListableClaudeCodeSession> {
        Some(ListableClaudeCodeSession {
            session_row: ClaudeCodeSessionRow {
                session_status: if is_waiting_for_an_answer {
                    ClaudeCodeSessionStatus::NeedsPermission
                } else {
                    self.session_status?
                },
                tool_summary: self.tool_summary.clone().filter(|_| !is_waiting_for_an_answer),
                project_folder_name: self.project_folder_name.clone(),
            },
            last_event_order: self.last_event_order,
        })
    }

    /// For the silence timeout (time) and the list's "newest first" (order).
    fn note_event(&mut self, event_time: Instant, event_order: u64) {
        self.last_event_time = event_time;
        self.last_event_order = event_order;
    }

    fn show_status_without_tool(&mut self, session_status: ClaudeCodeSessionStatus) {
        self.session_status = Some(session_status);
        self.tool_summary = None;
    }
}
