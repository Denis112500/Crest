//! Follows every Claude Code session from its hook events and describes them for the pill.
//! Pure (the caller passes the time in), so every rule is unit-tested.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::activity_core::{ActivityPresence, ActivityUpdate};
use crate::activity_sources::claude_code::claude_code_hook_event::*;
use crate::activity_sources::claude_code::claude_code_session_list::{
    list_claude_code_sessions, ClaudeCodeSessionRow, ClaudeCodeSessionStatus, ListableClaudeCodeSession,
};
use crate::activity_sources::claude_code::claude_code_tool_summary::summarize_tool_use;
use crate::backend_constants::CLAUDE_CODE_ACTIVITY_DISPLAY_PRIORITY;

struct TrackedSession {
    project_folder_name: String,
    transcript_path: Option<PathBuf>,
    /// `None` right after `SessionStart`: an idle new session has nothing to show yet.
    session_status: Option<ClaudeCodeSessionStatus>,
    tool_summary: Option<String>,
    last_event_time: Instant,
    last_event_order: u64,
}

#[derive(Default)]
pub struct ClaudeCodeSessionTracker {
    sessions_by_id: HashMap<String, TrackedSession>,
    next_event_order: u64,
    /// Grows whenever any session has news for the user ("done", "needs your OK"). It is the
    /// activity's attention key, so another session merely moving to the top isn't news.
    attention_event_count: u64,
}

impl ClaudeCodeSessionTracker {
    pub fn record_hook_event(&mut self, hook_event: &ClaudeCodeHookEvent, event_time: Instant) {
        if hook_event.hook_event_name == SESSION_END_HOOK_EVENT {
            self.sessions_by_id.remove(&hook_event.session_id);
            return;
        }
        let event_order = self.next_event_order;
        self.next_event_order += 1;
        // Crest may start in the middle of a session, so any event can be the first one seen.
        let tracked_session = self.sessions_by_id.entry(hook_event.session_id.clone()).or_insert_with(|| TrackedSession {
            project_folder_name: String::new(),
            transcript_path: None,
            session_status: None,
            tool_summary: None,
            last_event_time: event_time,
            last_event_order: event_order,
        });
        if let Some(transcript_path) = &hook_event.transcript_path {
            tracked_session.transcript_path = Some(PathBuf::from(transcript_path));
        }
        if let Some(working_folder) = &hook_event.cwd {
            tracked_session.project_folder_name =
                working_folder.rsplit(['\\', '/']).find(|part| !part.is_empty()).unwrap_or_default().to_string();
        }
        tracked_session.last_event_time = event_time;
        tracked_session.last_event_order = event_order;
        match (hook_event.hook_event_name.as_str(), hook_event.notification_type.as_deref()) {
            (USER_PROMPT_SUBMIT_HOOK_EVENT, _) | (POST_TOOL_USE_HOOK_EVENT, _) => {
                tracked_session.show_status_without_tool(ClaudeCodeSessionStatus::Working)
            }
            (PRE_TOOL_USE_HOOK_EVENT, _) => {
                tracked_session.session_status = Some(ClaudeCodeSessionStatus::Working);
                let tool_name = hook_event.tool_name.as_deref().unwrap_or_default();
                tracked_session.tool_summary = Some(summarize_tool_use(tool_name, hook_event.tool_input.as_ref()));
            }
            (STOP_HOOK_EVENT, _) => {
                tracked_session.show_status_without_tool(ClaudeCodeSessionStatus::Done);
                self.attention_event_count += 1;
            }
            (NOTIFICATION_HOOK_EVENT, Some(PERMISSION_PROMPT_NOTIFICATION)) => {
                tracked_session.show_status_without_tool(ClaudeCodeSessionStatus::NeedsPermission);
                self.attention_event_count += 1;
            }
            // Claude Code sends this about a minute after "done"; "done" says more, and a second
            // peek a minute later would only be noise.
            (NOTIFICATION_HOOK_EVENT, Some(IDLE_PROMPT_NOTIFICATION))
                if tracked_session.session_status != Some(ClaudeCodeSessionStatus::Done) =>
            {
                tracked_session.show_status_without_tool(ClaudeCodeSessionStatus::WaitingForInput)
            }
            _ => {}
        }
    }

    /// The user interrupted the session's turn (seen in its transcript; there's no hook event for
    /// it). Claude Code then waits for input, without a peek: the user did it themselves.
    pub fn record_interrupt(&mut self, session_id: &str, event_time: Instant) {
        let event_order = self.next_event_order;
        let Some(tracked_session) = self.sessions_by_id.get_mut(session_id) else {
            return;
        };
        if tracked_session.session_status != Some(ClaudeCodeSessionStatus::Working) {
            return;
        }
        self.next_event_order += 1;
        tracked_session.show_status_without_tool(ClaudeCodeSessionStatus::WaitingForInput);
        tracked_session.last_event_time = event_time;
        tracked_session.last_event_order = event_order;
    }

    /// The working sessions whose transcript is known: the ones to watch for an interrupt.
    pub fn working_session_transcripts(&self) -> Vec<(String, PathBuf)> {
        self.sessions_by_id
            .iter()
            .filter(|(_, tracked_session)| tracked_session.session_status == Some(ClaudeCodeSessionStatus::Working))
            .filter_map(|(session_id, tracked_session)| {
                tracked_session.transcript_path.clone().map(|transcript_path| (session_id.clone(), transcript_path))
            })
            .collect()
    }

    /// A crashed or killed Claude Code never sends `SessionEnd`; its session goes after a silence.
    pub fn forget_sessions_silent_for(&mut self, silence_timeout: Duration, current_time: Instant) {
        self.sessions_by_id
            .retain(|_, tracked_session| current_time.duration_since(tracked_session.last_event_time) < silence_timeout);
    }

    /// When the quietest session reaches the silence timeout, if there is any session.
    pub fn next_silence_deadline(&self, silence_timeout: Duration) -> Option<Instant> {
        self.sessions_by_id.values().map(|tracked_session| tracked_session.last_event_time + silence_timeout).min()
    }

    /// Every session with something to show, as one activity: ongoing while any of them works or
    /// waits for the user's OK (it's blocked on them, so it keeps the pill on screen and sits
    /// next to playing music). `None` = nothing to show.
    pub fn describe_activity(&self) -> Option<ActivityUpdate> {
        let listable_sessions: Vec<ListableClaudeCodeSession> = self
            .sessions_by_id
            .values()
            .filter_map(|tracked_session| {
                Some(ListableClaudeCodeSession {
                    session_row: ClaudeCodeSessionRow {
                        session_status: tracked_session.session_status?,
                        tool_summary: tracked_session.tool_summary.clone(),
                        project_folder_name: tracked_session.project_folder_name.clone(),
                    },
                    last_event_order: tracked_session.last_event_order,
                })
            })
            .collect();
        if listable_sessions.is_empty() {
            return None;
        }
        let is_any_session_busy =
            listable_sessions.iter().any(|listable_session| listable_session.session_row.session_status.is_busy());
        Some(ActivityUpdate {
            display_priority: CLAUDE_CODE_ACTIVITY_DISPLAY_PRIORITY,
            activity_presence: if is_any_session_busy { ActivityPresence::Ongoing } else { ActivityPresence::Lingering },
            attention_key: self.attention_event_count.to_string(),
            activity_payload: serde_json::to_value(list_claude_code_sessions(listable_sessions)).ok()?,
        })
    }
}

impl TrackedSession {
    fn show_status_without_tool(&mut self, session_status: ClaudeCodeSessionStatus) {
        self.session_status = Some(session_status);
        self.tool_summary = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hook_event(session_id: &str, hook_event_name: &str, notification_type: Option<&str>) -> ClaudeCodeHookEvent {
        ClaudeCodeHookEvent {
            session_id: session_id.to_string(),
            hook_event_name: hook_event_name.to_string(),
            cwd: Some(r"C:\Projects\crest".to_string()),
            transcript_path: Some(format!(r"C:\transcripts\{session_id}.jsonl")),
            tool_name: Some("Bash".to_string()),
            tool_input: Some(serde_json::json!({ "command": "cargo test" })),
            notification_type: notification_type.map(str::to_string),
        }
    }

    /// The first row of the list: what the compact pill shows.
    fn top_session(tracker: &ClaudeCodeSessionTracker) -> serde_json::Value {
        tracker.describe_activity().unwrap().activity_payload["listedSessions"][0].clone()
    }

    #[test]
    fn a_new_idle_session_shows_nothing_until_a_prompt() {
        let (mut tracker, start_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        tracker.record_hook_event(&hook_event("a", SESSION_START_HOOK_EVENT, None), start_time);
        assert!(tracker.describe_activity().is_none());
        tracker.record_hook_event(&hook_event("a", USER_PROMPT_SUBMIT_HOOK_EVENT, None), start_time);
        assert_eq!(tracker.describe_activity().unwrap().activity_presence, ActivityPresence::Ongoing);
        assert_eq!(top_session(&tracker)["projectFolderName"], "crest");
    }

    #[test]
    fn a_tool_call_shows_its_summary_and_done_peeks_once() {
        let (mut tracker, event_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        tracker.record_hook_event(&hook_event("a", PRE_TOOL_USE_HOOK_EVENT, None), event_time);
        assert_eq!(top_session(&tracker)["toolSummary"], "Running cargo test");
        let working_key = tracker.describe_activity().unwrap().attention_key;
        tracker.record_hook_event(&hook_event("a", STOP_HOOK_EVENT, None), event_time);
        let done = tracker.describe_activity().unwrap();
        assert_eq!((done.activity_presence, top_session(&tracker)["sessionStatus"].clone()), (ActivityPresence::Lingering, "done".into()));
        assert_ne!(done.attention_key, working_key);
        tracker.record_hook_event(&hook_event("a", NOTIFICATION_HOOK_EVENT, Some(IDLE_PROMPT_NOTIFICATION)), event_time);
        assert_eq!(tracker.describe_activity().unwrap(), done);
    }

    #[test]
    fn a_permission_prompt_is_ongoing_news_and_session_end_withdraws() {
        let (mut tracker, event_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        tracker.record_hook_event(&hook_event("a", NOTIFICATION_HOOK_EVENT, Some(PERMISSION_PROMPT_NOTIFICATION)), event_time);
        assert_eq!(top_session(&tracker)["sessionStatus"], "needsPermission");
        assert_eq!(tracker.describe_activity().unwrap().activity_presence, ActivityPresence::Ongoing);
        tracker.record_hook_event(&hook_event("a", SESSION_END_HOOK_EVENT, None), event_time);
        assert!(tracker.describe_activity().is_none());
    }

    #[test]
    fn every_session_is_listed_and_the_activity_is_ongoing_while_any_of_them_works() {
        let (mut tracker, event_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        tracker.record_hook_event(&hook_event("finished", STOP_HOOK_EVENT, None), event_time);
        tracker.record_hook_event(&hook_event("working", USER_PROMPT_SUBMIT_HOOK_EVENT, None), event_time);
        let both = tracker.describe_activity().unwrap();
        assert_eq!(both.activity_presence, ActivityPresence::Ongoing);
        assert_eq!(both.activity_payload["listedSessions"].as_array().unwrap().len(), 2);
        assert_eq!(top_session(&tracker)["sessionStatus"], "working");
        tracker.record_hook_event(&hook_event("working", STOP_HOOK_EVENT, None), event_time);
        assert_eq!(tracker.describe_activity().unwrap().activity_presence, ActivityPresence::Lingering);
    }

    #[test]
    fn only_done_or_needs_ok_is_news_whichever_session_it_comes_from() {
        let (mut tracker, event_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        tracker.record_hook_event(&hook_event("first", USER_PROMPT_SUBMIT_HOOK_EVENT, None), event_time);
        let first_key = tracker.describe_activity().unwrap().attention_key;
        tracker.record_hook_event(&hook_event("second", USER_PROMPT_SUBMIT_HOOK_EVENT, None), event_time);
        assert_eq!(tracker.describe_activity().unwrap().attention_key, first_key);
        // "first" isn't the top row (the newer working session is), but its "done" is news.
        tracker.record_hook_event(&hook_event("first", STOP_HOOK_EVENT, None), event_time);
        assert_eq!(top_session(&tracker)["sessionStatus"], "working");
        assert_ne!(tracker.describe_activity().unwrap().attention_key, first_key);
    }

    #[test]
    fn an_interrupt_turns_working_into_waiting_without_a_peek_and_stops_the_watch() {
        let (mut tracker, event_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        tracker.record_hook_event(&hook_event("a", PRE_TOOL_USE_HOOK_EVENT, None), event_time);
        assert_eq!(tracker.working_session_transcripts().len(), 1);
        let working_key = tracker.describe_activity().unwrap().attention_key;
        tracker.record_interrupt("a", event_time);
        let waiting = tracker.describe_activity().unwrap();
        assert_eq!(top_session(&tracker)["sessionStatus"], "waitingForInput");
        assert_eq!(waiting.activity_presence, ActivityPresence::Lingering);
        assert_eq!(waiting.attention_key, working_key);
        assert!(tracker.working_session_transcripts().is_empty());
    }

    #[test]
    fn a_silent_session_is_forgotten_after_the_timeout() {
        let (mut tracker, start_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        let silence_timeout = Duration::from_secs(600);
        tracker.record_hook_event(&hook_event("a", USER_PROMPT_SUBMIT_HOOK_EVENT, None), start_time);
        assert_eq!(tracker.next_silence_deadline(silence_timeout), Some(start_time + silence_timeout));
        tracker.forget_sessions_silent_for(silence_timeout, start_time + Duration::from_secs(599));
        assert!(tracker.describe_activity().is_some());
        tracker.forget_sessions_silent_for(silence_timeout, start_time + silence_timeout);
        assert!(tracker.describe_activity().is_none());
        assert_eq!(tracker.next_silence_deadline(silence_timeout), None);
    }
}
