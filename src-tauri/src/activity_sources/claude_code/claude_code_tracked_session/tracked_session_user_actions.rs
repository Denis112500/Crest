//! How the user's own actions change a tracked session: a cancel or deny in Claude Code (read
//! from the transcript), stopping the background subagents, and an answer from the pill.

use std::time::Instant;

use super::TrackedSession;
use crate::activity_sources::claude_code::claude_code_permission_answer::ClaudeCodePermissionAnswer;
use crate::activity_sources::claude_code::claude_code_session_list::ClaudeCodeSessionStatus;

impl TrackedSession {
    /// The user cancelled the turn or denied in Claude Code (`stopped_status` says which).
    pub fn show_interrupt(&mut self, stopped_status: ClaudeCodeSessionStatus, event_time: Instant, event_order: u64) {
        self.turn_steps.stop_running_steps();
        self.show_status_without_tool(stopped_status);
        self.running_subagents.forget_all();
        self.note_event(event_time, event_order);
    }

    /// The user stopped the background subagents. A turn that was only waiting for them was
    /// cancelled: its work didn't finish. Esc, Ctrl+C and Ctrl+X Ctrl+K all write the same line
    /// then (seen 2026-10-23), so which key it was can't be told. Returns whether it cancelled
    /// the turn.
    pub fn show_subagents_stopped(&mut self, event_time: Instant, event_order: u64) -> bool {
        let is_turn_cancelled = self.is_waiting_on_subagents();
        if is_turn_cancelled {
            self.turn_steps.stop_running_steps();
            self.show_status_without_tool(ClaudeCodeSessionStatus::Cancelled);
        } else {
            self.turn_steps.stop_running_subagent_steps();
        }
        self.running_subagents.forget_all();
        self.note_event(event_time, event_order);
        is_turn_cancelled
    }

    /// An allow lets the tool run; a deny ends the turn (no `Stop` follows an interrupted one).
    pub fn show_permission_answer(&mut self, permission_answer: ClaudeCodePermissionAnswer, event_time: Instant, event_order: u64) {
        if permission_answer == ClaudeCodePermissionAnswer::Deny {
            self.turn_steps.stop_running_steps();
        }
        self.show_status_without_tool(match permission_answer {
            ClaudeCodePermissionAnswer::Allow => ClaudeCodeSessionStatus::Working,
            ClaudeCodePermissionAnswer::Deny => ClaudeCodeSessionStatus::Denied,
        });
        self.note_event(event_time, event_order);
    }
}
