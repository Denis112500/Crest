//! Follows every Claude Code session from its hook events and describes them for the pill.
//! Pure (the caller passes the time in), so every rule is unit-tested. How one event changes one
//! session is in `claude_code_tracked_session.rs`; this file keeps the sessions, their order,
//! the news count and the timeouts.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use crate::activity_core::{ActivityPresence, ActivityUpdate};
use crate::activity_sources::claude_code::claude_code_hook_event::*;
use crate::activity_sources::claude_code::claude_code_permission_answer::ClaudeCodePermissionAnswer;
use crate::activity_sources::claude_code::claude_code_permission_queue::ClaudeCodePermissionAlert;
use crate::activity_sources::claude_code::claude_code_session_list::{list_claude_code_sessions, ListableClaudeCodeSession};
use crate::activity_sources::claude_code::claude_code_tracked_session::TrackedSession;
use crate::backend_constants::CLAUDE_CODE_ACTIVITY_DISPLAY_PRIORITY;

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
        let tracked_session = self
            .sessions_by_id
            .entry(hook_event.session_id.clone())
            .or_insert_with(|| TrackedSession::new(event_time, event_order));
        if tracked_session.apply_hook_event(hook_event, event_time, event_order) {
            self.attention_event_count += 1;
        }
    }

    /// The user interrupted the session's turn, or denied the permission it asked for (both are
    /// seen only in its transcript; neither sends a hook event). Claude Code then waits for input,
    /// without a peek: the user did it themselves.
    pub fn record_interrupt(&mut self, session_id: &str, event_time: Instant) {
        let event_order = self.next_event_order;
        let Some(tracked_session) = self.sessions_by_id.get_mut(session_id) else {
            return;
        };
        if !tracked_session.is_busy() {
            return;
        }
        self.next_event_order += 1;
        tracked_session.show_interrupt(event_time, event_order);
    }

    /// The pill answered the session's permission request. No peek either way: the user just did it.
    pub fn record_permission_answer(&mut self, session_id: &str, permission_answer: ClaudeCodePermissionAnswer, event_time: Instant) {
        let event_order = self.next_event_order;
        let Some(tracked_session) = self.sessions_by_id.get_mut(session_id) else {
            return;
        };
        self.next_event_order += 1;
        tracked_session.show_permission_answer(permission_answer, event_time, event_order);
    }

    /// The busy sessions (working, or waiting for an OK) whose transcript is known: the ones to
    /// watch for an interrupt or a denied permission.
    pub fn busy_session_transcripts(&self) -> Vec<(String, PathBuf)> {
        self.transcripts_of_sessions_where(TrackedSession::is_busy)
    }

    /// Every session whose transcript is known, to look for one that continued under a new id.
    pub fn session_transcripts(&self) -> Vec<(String, PathBuf)> {
        self.transcripts_of_sessions_where(|_| true)
    }

    fn transcripts_of_sessions_where(&self, is_wanted: impl Fn(&TrackedSession) -> bool) -> Vec<(String, PathBuf)> {
        self.sessions_by_id
            .iter()
            .filter(|(_, tracked_session)| is_wanted(tracked_session))
            .filter_map(|(session_id, tracked_session)| {
                tracked_session.transcript_path.clone().map(|transcript_path| (session_id.clone(), transcript_path))
            })
            .collect()
    }

    /// The session continued under a new id, which carries on in its place.
    pub fn forget_session(&mut self, session_id: &str) {
        self.sessions_by_id.remove(session_id);
    }

    /// The session stopped on a usage limit that resets in `time_until_reset` (read from its
    /// transcript): the pill says when, and the session stays listed until then.
    pub fn record_usage_limit_reset(
        &mut self,
        session_id: &str,
        resets_at_unix_seconds: u64,
        time_until_reset: Duration,
        current_time: Instant,
    ) {
        if let Some(tracked_session) = self.sessions_by_id.get_mut(session_id) {
            tracked_session.show_usage_limit_reset(resets_at_unix_seconds, time_until_reset, current_time);
        }
    }

    /// A crashed or killed Claude Code never sends `SessionEnd`; its session goes after a silence.
    /// A subagent that falls silent goes the same way (its stop may never be reported).
    pub fn forget_sessions_silent_for(&mut self, silence_timeout: Duration, current_time: Instant) {
        self.sessions_by_id.retain(|_, tracked_session| current_time < tracked_session.silence_deadline(silence_timeout));
        for tracked_session in self.sessions_by_id.values_mut() {
            tracked_session.forget_subagents_silent_for(silence_timeout, current_time);
        }
    }

    /// When the quietest session or subagent reaches the silence timeout, if there is any.
    pub fn next_silence_deadline(&self, silence_timeout: Duration) -> Option<Instant> {
        self.sessions_by_id
            .values()
            .flat_map(|tracked_session| {
                [
                    Some(tracked_session.silence_deadline(silence_timeout)),
                    tracked_session.next_subagent_silence_deadline(silence_timeout),
                ]
            })
            .flatten()
            .min()
    }

    /// Every session with something to show, as one activity: an alert while a permission request
    /// waits for an answer from the pill (it takes the pill over); else ongoing while any session
    /// works or waits for the user's OK (it keeps the pill on screen and sits next to playing
    /// music). `None` = nothing to show.
    pub fn describe_activity(
        &self,
        permission_alert: Option<ClaudeCodePermissionAlert>,
        sessions_waiting_for_an_answer: &[String],
    ) -> Option<ActivityUpdate> {
        let listable_sessions: Vec<ListableClaudeCodeSession> = self
            .sessions_by_id
            .iter()
            .filter_map(|(session_id, tracked_session)| {
                tracked_session.listable_session(sessions_waiting_for_an_answer.contains(session_id))
            })
            .collect();
        if listable_sessions.is_empty() {
            return None;
        }
        let is_any_session_busy =
            listable_sessions.iter().any(|listable_session| listable_session.session_row.session_status.is_busy());
        let activity_presence = match (&permission_alert, is_any_session_busy) {
            (Some(_), _) => ActivityPresence::Alert,
            (None, true) => ActivityPresence::Ongoing,
            (None, false) => ActivityPresence::Lingering,
        };
        let mut status_payload = list_claude_code_sessions(listable_sessions);
        status_payload.permission_alert = permission_alert;
        Some(ActivityUpdate {
            display_priority: CLAUDE_CODE_ACTIVITY_DISPLAY_PRIORITY,
            activity_presence,
            attention_key: self.attention_event_count.to_string(),
            activity_payload: serde_json::to_value(status_payload).ok()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity_sources::claude_code::claude_code_permission_description::ClaudeCodePermissionDescription;
    use crate::backend_constants::CLAUDE_CODE_USAGE_LIMIT_MAX_LISTED_WAIT;

    fn hook_event(session_id: &str, hook_event_name: &str, notification_type: Option<&str>) -> ClaudeCodeHookEvent {
        ClaudeCodeHookEvent {
            session_id: session_id.to_string(),
            hook_event_name: hook_event_name.to_string(),
            cwd: Some(r"C:\Projects\crest".to_string()),
            transcript_path: Some(format!(r"C:\transcripts\{session_id}.jsonl")),
            tool_name: Some("Bash".to_string()),
            tool_input: Some(serde_json::json!({ "command": "cargo test" })),
            notification_type: notification_type.map(str::to_string),
            ..ClaudeCodeHookEvent::default()
        }
    }

    fn subagent_event(session_id: &str, hook_event_name: &str, agent_id: &str) -> ClaudeCodeHookEvent {
        ClaudeCodeHookEvent { agent_id: Some(agent_id.to_string()), ..hook_event(session_id, hook_event_name, None) }
    }

    fn stop_failure(session_id: &str, error: &str) -> ClaudeCodeHookEvent {
        ClaudeCodeHookEvent { error: Some(error.to_string()), ..hook_event(session_id, STOP_FAILURE_HOOK_EVENT, None) }
    }

    /// The first row of the list: what the compact pill shows.
    fn top_session(tracker: &ClaudeCodeSessionTracker) -> serde_json::Value {
        tracker.describe_activity(None, &[]).unwrap().activity_payload["listedSessions"][0].clone()
    }

    #[test]
    fn a_new_idle_session_shows_nothing_until_a_prompt() {
        let (mut tracker, start_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        tracker.record_hook_event(&hook_event("a", SESSION_START_HOOK_EVENT, None), start_time);
        assert!(tracker.describe_activity(None, &[]).is_none());
        tracker.record_hook_event(&hook_event("a", USER_PROMPT_SUBMIT_HOOK_EVENT, None), start_time);
        assert_eq!(tracker.describe_activity(None, &[]).unwrap().activity_presence, ActivityPresence::Ongoing);
        assert_eq!(top_session(&tracker)["projectFolderName"], "crest");
    }

    #[test]
    fn a_tool_call_shows_its_summary_and_done_peeks_once() {
        let (mut tracker, event_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        tracker.record_hook_event(&hook_event("a", PRE_TOOL_USE_HOOK_EVENT, None), event_time);
        assert_eq!(top_session(&tracker)["toolSummary"], "Running cargo test");
        let working_key = tracker.describe_activity(None, &[]).unwrap().attention_key;
        tracker.record_hook_event(&hook_event("a", STOP_HOOK_EVENT, None), event_time);
        let done = tracker.describe_activity(None, &[]).unwrap();
        assert_eq!((done.activity_presence, top_session(&tracker)["sessionStatus"].clone()), (ActivityPresence::Lingering, "done".into()));
        assert_ne!(done.attention_key, working_key);
        tracker.record_hook_event(&hook_event("a", NOTIFICATION_HOOK_EVENT, Some(IDLE_PROMPT_NOTIFICATION)), event_time);
        assert_eq!(tracker.describe_activity(None, &[]).unwrap(), done);
    }

    #[test]
    fn a_permission_prompt_is_ongoing_news_and_session_end_withdraws() {
        let (mut tracker, event_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        tracker.record_hook_event(&hook_event("a", NOTIFICATION_HOOK_EVENT, Some(PERMISSION_PROMPT_NOTIFICATION)), event_time);
        assert_eq!(top_session(&tracker)["sessionStatus"], "needsPermission");
        assert_eq!(tracker.describe_activity(None, &[]).unwrap().activity_presence, ActivityPresence::Ongoing);
        tracker.record_hook_event(&hook_event("a", SESSION_END_HOOK_EVENT, None), event_time);
        assert!(tracker.describe_activity(None, &[]).is_none());
    }

    #[test]
    fn every_session_is_listed_and_the_activity_is_ongoing_while_any_of_them_works() {
        let (mut tracker, event_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        tracker.record_hook_event(&hook_event("finished", STOP_HOOK_EVENT, None), event_time);
        tracker.record_hook_event(&hook_event("working", USER_PROMPT_SUBMIT_HOOK_EVENT, None), event_time);
        let both = tracker.describe_activity(None, &[]).unwrap();
        assert_eq!(both.activity_presence, ActivityPresence::Ongoing);
        assert_eq!(both.activity_payload["listedSessions"].as_array().unwrap().len(), 2);
        assert_eq!(top_session(&tracker)["sessionStatus"], "working");
        tracker.record_hook_event(&hook_event("working", STOP_HOOK_EVENT, None), event_time);
        assert_eq!(tracker.describe_activity(None, &[]).unwrap().activity_presence, ActivityPresence::Lingering);
    }

    #[test]
    fn only_done_or_needs_ok_is_news_whichever_session_it_comes_from() {
        let (mut tracker, event_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        tracker.record_hook_event(&hook_event("first", USER_PROMPT_SUBMIT_HOOK_EVENT, None), event_time);
        let first_key = tracker.describe_activity(None, &[]).unwrap().attention_key;
        tracker.record_hook_event(&hook_event("second", USER_PROMPT_SUBMIT_HOOK_EVENT, None), event_time);
        assert_eq!(tracker.describe_activity(None, &[]).unwrap().attention_key, first_key);
        // "first" isn't the top row (the newer working session is), but its "done" is news.
        tracker.record_hook_event(&hook_event("first", STOP_HOOK_EVENT, None), event_time);
        assert_eq!(top_session(&tracker)["sessionStatus"], "working");
        assert_ne!(tracker.describe_activity(None, &[]).unwrap().attention_key, first_key);
    }

    #[test]
    fn an_interrupt_turns_working_into_waiting_without_a_peek_and_stops_the_watch() {
        let (mut tracker, event_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        tracker.record_hook_event(&hook_event("a", PRE_TOOL_USE_HOOK_EVENT, None), event_time);
        assert_eq!(tracker.busy_session_transcripts().len(), 1);
        let working_key = tracker.describe_activity(None, &[]).unwrap().attention_key;
        tracker.record_interrupt("a", event_time);
        let waiting = tracker.describe_activity(None, &[]).unwrap();
        assert_eq!(top_session(&tracker)["sessionStatus"], "waitingForInput");
        assert_eq!(waiting.activity_presence, ActivityPresence::Lingering);
        assert_eq!(waiting.attention_key, working_key);
        assert!(tracker.busy_session_transcripts().is_empty());
    }

    #[test]
    fn a_denied_permission_turns_needs_ok_into_waiting_without_a_peek() {
        let (mut tracker, event_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        tracker.record_hook_event(&hook_event("a", PRE_TOOL_USE_HOOK_EVENT, None), event_time);
        tracker.record_hook_event(&hook_event("a", NOTIFICATION_HOOK_EVENT, Some(PERMISSION_PROMPT_NOTIFICATION)), event_time);
        assert_eq!(tracker.busy_session_transcripts().len(), 1);
        let needs_ok_key = tracker.describe_activity(None, &[]).unwrap().attention_key;
        tracker.record_interrupt("a", event_time);
        let waiting = tracker.describe_activity(None, &[]).unwrap();
        assert_eq!(top_session(&tracker)["sessionStatus"], "waitingForInput");
        assert_eq!((waiting.activity_presence, waiting.attention_key), (ActivityPresence::Lingering, needs_ok_key));
        assert!(tracker.busy_session_transcripts().is_empty());
    }

    fn permission_alert() -> ClaudeCodePermissionAlert {
        ClaudeCodePermissionAlert {
            request_id: 0,
            project_folder_name: "crest".to_string(),
            request_description: ClaudeCodePermissionDescription::Action { action_text: "Edit notes.md".to_string() },
            other_waiting_request_count: 0,
        }
    }

    #[test]
    fn a_permission_request_needs_an_ok_at_once_and_its_notification_is_not_news_again() {
        let (mut tracker, event_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        tracker.record_hook_event(&hook_event("a", PRE_TOOL_USE_HOOK_EVENT, None), event_time);
        let working_key = tracker.describe_activity(None, &[]).unwrap().attention_key;
        tracker.record_hook_event(&hook_event("a", PERMISSION_REQUEST_HOOK_EVENT, None), event_time);
        let alert = tracker.describe_activity(Some(permission_alert()), &[]).unwrap();
        assert_eq!(alert.activity_presence, ActivityPresence::Alert);
        assert_eq!(alert.activity_payload["permissionAlert"]["requestDescription"]["actionText"], "Edit notes.md");
        assert_eq!(top_session(&tracker)["sessionStatus"], "needsPermission");
        assert_ne!(alert.attention_key, working_key);
        tracker.record_hook_event(&hook_event("a", NOTIFICATION_HOOK_EVENT, Some(PERMISSION_PROMPT_NOTIFICATION)), event_time);
        assert_eq!(tracker.describe_activity(Some(permission_alert()), &[]).unwrap(), alert);
        assert!(tracker.describe_activity(None, &[]).unwrap().activity_payload["permissionAlert"].is_null());
    }

    #[test]
    fn an_answer_from_the_pill_moves_the_session_on_without_a_peek() {
        let (mut tracker, event_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        tracker.record_hook_event(&hook_event("allowed", PERMISSION_REQUEST_HOOK_EVENT, None), event_time);
        tracker.record_hook_event(&hook_event("denied", PERMISSION_REQUEST_HOOK_EVENT, None), event_time);
        let needs_ok_key = tracker.describe_activity(None, &[]).unwrap().attention_key;
        tracker.record_permission_answer("allowed", ClaudeCodePermissionAnswer::Allow, event_time);
        tracker.record_permission_answer("denied", ClaudeCodePermissionAnswer::Deny, event_time);
        let after_answers = tracker.describe_activity(None, &[]).unwrap();
        let session_statuses: Vec<&serde_json::Value> = after_answers.activity_payload["listedSessions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|session_row| &session_row["sessionStatus"])
            .collect();
        assert_eq!(session_statuses, ["working", "waitingForInput"]);
        assert_eq!(after_answers.attention_key, needs_ok_key);
        // A tool allowed in Claude Code that then failed: the session works on.
        tracker.record_hook_event(&hook_event("denied", POST_TOOL_USE_FAILURE_HOOK_EVENT, None), event_time);
        assert_eq!(tracker.describe_activity(None, &[]).unwrap().activity_presence, ActivityPresence::Ongoing);
    }

    #[test]
    fn a_session_with_a_request_still_waiting_keeps_needing_an_ok() {
        let (mut tracker, event_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        let waiting_sessions = ["a".to_string()];
        tracker.record_hook_event(&hook_event("a", PERMISSION_REQUEST_HOOK_EVENT, None), event_time);
        // A parallel tool call of the same turn, then the pill answering the session's other request.
        tracker.record_hook_event(&hook_event("a", PRE_TOOL_USE_HOOK_EVENT, None), event_time);
        tracker.record_permission_answer("a", ClaudeCodePermissionAnswer::Allow, event_time);
        let still_waiting = tracker.describe_activity(Some(permission_alert()), &waiting_sessions).unwrap();
        let top_row = &still_waiting.activity_payload["listedSessions"][0];
        assert_eq!((&top_row["sessionStatus"], &top_row["toolSummary"]), (&"needsPermission".into(), &serde_json::Value::Null));
        assert_eq!(top_session(&tracker)["sessionStatus"], "working");
    }

    #[test]
    fn a_session_continued_under_a_new_id_leaves_the_list() {
        let (mut tracker, event_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        tracker.record_hook_event(&hook_event("old", STOP_HOOK_EVENT, None), event_time);
        tracker.record_hook_event(&hook_event("new", USER_PROMPT_SUBMIT_HOOK_EVENT, None), event_time);
        assert_eq!(tracker.session_transcripts().len(), 2);
        tracker.forget_session("old");
        let listed_sessions = tracker.describe_activity(None, &[]).unwrap().activity_payload["listedSessions"].clone();
        assert_eq!(listed_sessions.as_array().unwrap().len(), 1);
        assert_eq!(tracker.session_transcripts()[0].0, "new");
    }

    #[test]
    fn a_silent_session_is_forgotten_after_the_timeout() {
        let (mut tracker, start_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        let silence_timeout = Duration::from_secs(600);
        tracker.record_hook_event(&hook_event("a", USER_PROMPT_SUBMIT_HOOK_EVENT, None), start_time);
        assert_eq!(tracker.next_silence_deadline(silence_timeout), Some(start_time + silence_timeout));
        tracker.forget_sessions_silent_for(silence_timeout, start_time + Duration::from_secs(599));
        assert!(tracker.describe_activity(None, &[]).is_some());
        tracker.forget_sessions_silent_for(silence_timeout, start_time + silence_timeout);
        assert!(tracker.describe_activity(None, &[]).is_none());
        assert_eq!(tracker.next_silence_deadline(silence_timeout), None);
    }

    fn launch_two_subagents(tracker: &mut ClaudeCodeSessionTracker, event_time: Instant) {
        let agent_launch =
            ClaudeCodeHookEvent { tool_name: Some("Agent".to_string()), ..hook_event("a", PRE_TOOL_USE_HOOK_EVENT, None) };
        tracker.record_hook_event(&agent_launch, event_time);
        tracker.record_hook_event(&subagent_event("a", SUBAGENT_START_HOOK_EVENT, "explore-1"), event_time);
        tracker.record_hook_event(&subagent_event("a", SUBAGENT_START_HOOK_EVENT, "explore-2"), event_time);
    }

    /// The top row's line and its "· N subagents" count.
    fn top_line_and_subagent_count(tracker: &ClaudeCodeSessionTracker) -> (serde_json::Value, serde_json::Value) {
        let top_row = top_session(tracker);
        (top_row["toolSummary"].clone(), top_row["unmentionedSubagentCount"].clone())
    }

    #[test]
    fn subagents_are_counted_and_their_tool_calls_leave_the_main_line_alone() {
        let (mut tracker, event_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        launch_two_subagents(&mut tracker, event_time);
        tracker.record_hook_event(&subagent_event("a", PRE_TOOL_USE_HOOK_EVENT, "explore-1"), event_time);
        tracker.record_hook_event(&subagent_event("a", POST_TOOL_USE_HOOK_EVENT, "explore-2"), event_time);
        assert_eq!(top_line_and_subagent_count(&tracker), ("Running 2 subagents".into(), 0.into()));
        tracker.record_hook_event(&subagent_event("a", SUBAGENT_STOP_HOOK_EVENT, "explore-1"), event_time);
        assert_eq!(top_line_and_subagent_count(&tracker), ("Running a subagent".into(), 0.into()));
        // The main turn goes on, then ends, while a subagent still runs in the background.
        tracker.record_hook_event(&hook_event("a", PRE_TOOL_USE_HOOK_EVENT, None), event_time);
        assert_eq!(top_line_and_subagent_count(&tracker), ("Running cargo test".into(), 1.into()));
        tracker.record_hook_event(&hook_event("a", STOP_HOOK_EVENT, None), event_time);
        tracker.record_hook_event(&subagent_event("a", PRE_TOOL_USE_HOOK_EVENT, "explore-2"), event_time);
        assert_eq!(top_session(&tracker)["sessionStatus"], "done");
        assert_eq!(top_line_and_subagent_count(&tracker), (serde_json::Value::Null, 1.into()));
    }

    #[test]
    fn a_subagent_answered_in_claude_code_brings_the_subagent_line_back() {
        let (mut tracker, event_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        launch_two_subagents(&mut tracker, event_time);
        let working_key = tracker.describe_activity(None, &[]).unwrap().attention_key;
        tracker.record_hook_event(&subagent_event("a", PERMISSION_REQUEST_HOOK_EVENT, "explore-1"), event_time);
        assert_eq!(top_session(&tracker)["sessionStatus"], "needsPermission");
        assert_ne!(tracker.describe_activity(None, &[]).unwrap().attention_key, working_key);
        tracker.record_hook_event(&subagent_event("a", POST_TOOL_USE_HOOK_EVENT, "explore-1"), event_time);
        assert_eq!(top_session(&tracker)["sessionStatus"], "working");
        assert_eq!(top_line_and_subagent_count(&tracker), ("Running 2 subagents".into(), 0.into()));
    }

    #[test]
    fn an_interrupt_or_the_silence_timeout_ends_the_subagent_count() {
        let (mut tracker, start_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        let silence_timeout = Duration::from_secs(600);
        launch_two_subagents(&mut tracker, start_time);
        tracker.record_interrupt("a", start_time);
        assert_eq!(top_session(&tracker)["unmentionedSubagentCount"], 0);
        tracker.record_hook_event(&subagent_event("a", SUBAGENT_START_HOOK_EVENT, "background"), start_time);
        // The main session keeps talking; the subagent says nothing more.
        let later_time = start_time + Duration::from_secs(300);
        tracker.record_hook_event(&hook_event("a", USER_PROMPT_SUBMIT_HOOK_EVENT, None), later_time);
        assert_eq!(tracker.next_silence_deadline(silence_timeout), Some(start_time + silence_timeout));
        tracker.forget_sessions_silent_for(silence_timeout, start_time + silence_timeout);
        assert_eq!(top_session(&tracker)["unmentionedSubagentCount"], 0);
        assert_eq!(tracker.next_silence_deadline(silence_timeout), Some(later_time + silence_timeout));
    }

    #[test]
    fn a_usage_limit_is_news_once_and_stays_listed_until_its_reset() {
        let (mut tracker, start_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        let silence_timeout = Duration::from_secs(600);
        tracker.record_hook_event(&hook_event("a", PRE_TOOL_USE_HOOK_EVENT, None), start_time);
        let working_key = tracker.describe_activity(None, &[]).unwrap().attention_key;
        tracker.record_hook_event(&stop_failure("a", RATE_LIMIT_STOP_FAILURE_ERROR), start_time);
        let limited = tracker.describe_activity(None, &[]).unwrap();
        assert_eq!(limited.activity_presence, ActivityPresence::Lingering);
        assert_eq!(top_session(&tracker)["sessionStatus"], "usageLimited");
        assert_ne!(limited.attention_key, working_key);
        let three_hours = Duration::from_secs(3 * 60 * 60);
        tracker.record_usage_limit_reset("a", 1_791_327_000, three_hours, start_time);
        assert_eq!(top_session(&tracker)["usageLimitResetsAt"], 1_791_327_000);
        assert_eq!(tracker.next_silence_deadline(silence_timeout), Some(start_time + three_hours + silence_timeout));
        tracker.forget_sessions_silent_for(silence_timeout, start_time + three_hours);
        assert!(tracker.describe_activity(None, &[]).is_some());
        // Claude Code continues by itself at the reset.
        let resumed = hook_event("a", NOTIFICATION_HOOK_EVENT, Some(QUOTA_AUTO_RESUME_FIRED_NOTIFICATION));
        tracker.record_hook_event(&resumed, start_time + three_hours);
        assert_eq!(top_session(&tracker)["sessionStatus"], "working");
        assert!(top_session(&tracker)["usageLimitResetsAt"].is_null());
    }

    #[test]
    fn a_limit_days_away_is_listed_for_a_day_at_most_and_other_errors_only_end_the_turn() {
        let (mut tracker, start_time) = (ClaudeCodeSessionTracker::default(), Instant::now());
        let silence_timeout = Duration::from_secs(600);
        tracker.record_hook_event(&stop_failure("a", RATE_LIMIT_STOP_FAILURE_ERROR), start_time);
        tracker.record_usage_limit_reset("a", 1_791_900_000, Duration::from_secs(4 * 24 * 60 * 60), start_time);
        let expected_deadline = start_time + CLAUDE_CODE_USAGE_LIMIT_MAX_LISTED_WAIT + silence_timeout;
        assert_eq!(tracker.next_silence_deadline(silence_timeout), Some(expected_deadline));
        let key_before = tracker.describe_activity(None, &[]).unwrap().attention_key;
        tracker.record_hook_event(&hook_event("b", PRE_TOOL_USE_HOOK_EVENT, None), start_time);
        tracker.record_hook_event(&stop_failure("b", "overloaded"), start_time);
        let after_error = tracker.describe_activity(None, &[]).unwrap();
        let listed_sessions = &after_error.activity_payload["listedSessions"];
        assert_eq!(listed_sessions[0]["sessionStatus"], "usageLimited");
        assert_eq!(listed_sessions[1]["sessionStatus"], "waitingForInput");
        assert_eq!(after_error.attention_key, key_before);
    }
}
