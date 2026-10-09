//! The list of Claude Code sessions the pill shows: which order, how many rows, what is counted.
//! Pure and unit-tested; the tracker hands it every session that has something to show.

use std::cmp::Reverse;

use serde::Serialize;

use crate::backend_constants::CLAUDE_CODE_LISTED_SESSION_MAX_COUNT;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ClaudeCodeSessionStatus {
    Working,
    Done,
    WaitingForInput,
    NeedsPermission,
}

impl ClaudeCodeSessionStatus {
    /// Working, or blocked on the user's OK: counted as "+1" and keeps the pill on screen.
    pub fn is_busy(self) -> bool {
        matches!(self, ClaudeCodeSessionStatus::Working | ClaudeCodeSessionStatus::NeedsPermission)
    }
}

/// One session as the pill shows it (mirrored in `claudeCodeStatusTypes.ts`).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeSessionRow {
    pub session_status: ClaudeCodeSessionStatus,
    /// Only while working on a tool, e.g. "Editing notes.md".
    pub tool_summary: Option<String>,
    pub project_folder_name: String,
}

/// What the pill's Claude Code views receive (mirrored in `claudeCodeStatusTypes.ts`).
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeStatusPayload {
    /// Most urgent first; the compact pill shows the first one.
    pub listed_sessions: Vec<ClaudeCodeSessionRow>,
    /// Sessions that didn't fit in the open pill ("+2 more").
    pub unlisted_session_count: usize,
    /// Sessions after the first that are working or need an OK ("+1" in the compact pill);
    /// finished or idle ones don't count.
    pub other_busy_session_count: usize,
}

/// A session that has something to show, with the order of its latest event (higher = newer).
pub struct ListableClaudeCodeSession {
    pub session_row: ClaudeCodeSessionRow,
    pub last_event_order: u64,
}

/// Needs your OK first (it's blocked on the user), then working, then waiting for you or done;
/// within each group the newest first. Only as many rows as the open pill has room for.
pub fn list_claude_code_sessions(mut listable_sessions: Vec<ListableClaudeCodeSession>) -> ClaudeCodeStatusPayload {
    listable_sessions.sort_by_key(|listable_session| {
        (urgency_rank(listable_session.session_row.session_status), Reverse(listable_session.last_event_order))
    });
    let other_busy_session_count = listable_sessions
        .iter()
        .skip(1)
        .filter(|listable_session| listable_session.session_row.session_status.is_busy())
        .count();
    let unlisted_session_count = listable_sessions.len().saturating_sub(CLAUDE_CODE_LISTED_SESSION_MAX_COUNT);
    let listed_sessions = listable_sessions
        .into_iter()
        .take(CLAUDE_CODE_LISTED_SESSION_MAX_COUNT)
        .map(|listable_session| listable_session.session_row)
        .collect();
    ClaudeCodeStatusPayload { listed_sessions, unlisted_session_count, other_busy_session_count }
}

fn urgency_rank(session_status: ClaudeCodeSessionStatus) -> u8 {
    match session_status {
        ClaudeCodeSessionStatus::NeedsPermission => 0,
        ClaudeCodeSessionStatus::Working => 1,
        ClaudeCodeSessionStatus::WaitingForInput | ClaudeCodeSessionStatus::Done => 2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listable(project_folder_name: &str, session_status: ClaudeCodeSessionStatus, last_event_order: u64) -> ListableClaudeCodeSession {
        ListableClaudeCodeSession {
            session_row: ClaudeCodeSessionRow {
                session_status,
                tool_summary: None,
                project_folder_name: project_folder_name.to_string(),
            },
            last_event_order,
        }
    }

    fn listed_folder_names(status_payload: &ClaudeCodeStatusPayload) -> Vec<&str> {
        status_payload.listed_sessions.iter().map(|session_row| session_row.project_folder_name.as_str()).collect()
    }

    #[test]
    fn needs_ok_comes_first_then_working_then_finished_newest_first_within_each() {
        let status_payload = list_claude_code_sessions(vec![
            listable("old done", ClaudeCodeSessionStatus::Done, 1),
            listable("working", ClaudeCodeSessionStatus::Working, 2),
            listable("needs ok", ClaudeCodeSessionStatus::NeedsPermission, 0),
            listable("new waiting", ClaudeCodeSessionStatus::WaitingForInput, 3),
        ]);
        assert_eq!(listed_folder_names(&status_payload), vec!["needs ok", "working", "new waiting"]);
        assert_eq!(status_payload.unlisted_session_count, 1);
    }

    #[test]
    fn only_busy_sessions_after_the_first_are_counted_including_unlisted_ones() {
        let status_payload = list_claude_code_sessions(vec![
            listable("a", ClaudeCodeSessionStatus::Working, 5),
            listable("b", ClaudeCodeSessionStatus::Working, 4),
            listable("c", ClaudeCodeSessionStatus::Working, 3),
            listable("d", ClaudeCodeSessionStatus::Working, 2),
            listable("e", ClaudeCodeSessionStatus::Done, 1),
        ]);
        assert_eq!(status_payload.listed_sessions.len(), CLAUDE_CODE_LISTED_SESSION_MAX_COUNT);
        assert_eq!((status_payload.other_busy_session_count, status_payload.unlisted_session_count), (3, 2));
    }

    #[test]
    fn a_single_session_has_nothing_counted() {
        let status_payload = list_claude_code_sessions(vec![listable("a", ClaudeCodeSessionStatus::Done, 0)]);
        assert_eq!((status_payload.other_busy_session_count, status_payload.unlisted_session_count), (0, 0));
    }
}
