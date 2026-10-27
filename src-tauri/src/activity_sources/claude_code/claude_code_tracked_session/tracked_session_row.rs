//! A tracked session's row in the pill's list.

use super::TrackedSession;
use crate::activity_sources::claude_code::claude_code_session_list::{
    ClaudeCodeSessionRow, ClaudeCodeSessionStatus, ListableClaudeCodeSession,
};

impl TrackedSession {
    /// Its row in the pill's list, `None` while there is nothing to show yet. A session with a
    /// request still waiting shows "needs your OK" whatever it sent since; one whose turn ended
    /// while background subagents still run shows as working on them ("Running 3 subagents").
    pub fn listable_session(&self, session_id: &str, is_waiting_for_an_answer: bool) -> Option<ListableClaudeCodeSession> {
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
                session_id: session_id.to_string(),
                session_status,
                tool_summary,
                project_folder_name: self.project_folder_name.clone(),
                unmentioned_subagent_count,
                usage_limit_resets_at: self.usage_limit_wait.map(|usage_limit_wait| usage_limit_wait.resets_at_unix_seconds),
                // Waiting on background subagents shows as working: the summary waits for "done".
                turn_summary: match session_status {
                    ClaudeCodeSessionStatus::Done | ClaudeCodeSessionStatus::Errored => self.turn_summary.clone(),
                    _ => None,
                },
                session_title: self.session_title.as_ref().map(|session_title| session_title.title_text.clone()),
                news_count: self.news_count,
                can_dismiss: !session_status.is_busy(),
            },
            last_event_order: self.last_event_order,
        })
    }
}
