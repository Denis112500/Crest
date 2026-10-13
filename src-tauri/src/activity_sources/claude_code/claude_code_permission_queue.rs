//! The permission requests waiting for an answer, oldest first, each holding its waiting hook.
//! The pill shows the oldest. A request leaves when the pill answers it, when Claude Code's
//! events show it was answered there, or when it is too old; its hook gets exactly one reply
//! either way. Pure (the caller passes the time in), so every rule is unit-tested.

use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;

use crate::activity_sources::claude_code::claude_code_hook_event::*;
use crate::activity_sources::claude_code::claude_code_permission_description::{
    describe_permission_request, ClaudeCodePermissionDescription,
};
use crate::activity_sources::claude_code::claude_code_permission_file_path::PermissionRequestFolders;
use crate::activity_sources::claude_code::last_path_part::find_last_path_part;
use crate::claude_code_hook_channel::{PendingHookReply, NO_HOOK_DECISION};

/// Claude Code sends these through the permission hook too, but they are questions for the user:
/// AskUserQuestion's choices and ExitPlanMode's plan can't be shown in the pill, and Allow would
/// answer them unseen (AskUserQuestion verified by testing, 2026-10-12; ExitPlanMode by analogy).
const TOOLS_ANSWERED_ONLY_IN_CLAUDE_CODE: [&str; 2] = ["AskUserQuestion", "ExitPlanMode"];

/// The request the pill shows (mirrored in `claudeCodeStatusTypes.ts`).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodePermissionAlert {
    /// Sent back with the click, so it can only answer this request.
    pub request_id: u64,
    pub project_folder_name: String,
    pub request_description: ClaudeCodePermissionDescription,
    /// Requests behind this one, from any session ("+1 waiting").
    pub other_waiting_request_count: usize,
}

struct PendingPermissionRequest {
    request_id: u64,
    session_id: String,
    project_folder_name: String,
    tool_name: Option<String>,
    tool_input: Option<Value>,
    request_description: ClaudeCodePermissionDescription,
    arrival_time: Instant,
    hook_reply: Box<dyn PendingHookReply>,
}

#[derive(Default)]
pub struct ClaudeCodePermissionQueue {
    pending_requests: Vec<PendingPermissionRequest>,
    next_request_id: u64,
    /// Files there are shown as "~\…" in the alert; passed in, so the queue stays pure.
    home_folder: Option<String>,
}

impl ClaudeCodePermissionQueue {
    pub fn new(home_folder: Option<String>) -> Self {
        Self { home_folder, ..Self::default() }
    }

    /// A question rather than a permission is left to Claude Code: its hook ends at once.
    pub fn add_request(&mut self, hook_event: &ClaudeCodeHookEvent, hook_reply: Box<dyn PendingHookReply>, arrival_time: Instant) {
        let tool_name = hook_event.tool_name.as_deref().unwrap_or_default();
        if TOOLS_ANSWERED_ONLY_IN_CLAUDE_CODE.contains(&tool_name) {
            hook_reply.send_reply(NO_HOOK_DECISION);
            return;
        }
        let request_id = self.next_request_id;
        self.next_request_id += 1;
        self.pending_requests.push(PendingPermissionRequest {
            request_id,
            session_id: hook_event.session_id.clone(),
            project_folder_name: hook_event.cwd.as_deref().and_then(find_last_path_part).unwrap_or_default().to_string(),
            tool_name: hook_event.tool_name.clone(),
            tool_input: hook_event.tool_input.clone(),
            request_description: describe_permission_request(
                tool_name,
                hook_event.tool_input.as_ref(),
                &PermissionRequestFolders {
                    project_folder: hook_event.cwd.as_deref(),
                    home_folder: self.home_folder.as_deref(),
                },
            ),
            arrival_time,
            hook_reply,
        });
    }

    /// The pill answered. Returns the request's session, or `None` when the request already left
    /// (answered in Claude Code a moment before the click): then nothing is sent.
    pub fn answer_request(&mut self, request_id: u64, reply_text: &str) -> Option<String> {
        let request_position = self.pending_requests.iter().position(|pending| pending.request_id == request_id)?;
        let answered_request = self.pending_requests.remove(request_position);
        answered_request.hook_reply.send_reply(reply_text);
        Some(answered_request.session_id)
    }

    /// No hook event says "answered in Claude Code's dialog", so it's read from what follows:
    /// the tool ran (`PostToolUse`) or failed after running (`PostToolUseFailure`), which releases
    /// that one request; the turn ended or a new one began, which releases all of the session's.
    /// A denied request is noticed in the transcript instead (`release_session_requests`).
    pub fn release_requests_answered_elsewhere(&mut self, hook_event: &ClaudeCodeHookEvent) {
        match hook_event.hook_event_name.as_str() {
            POST_TOOL_USE_HOOK_EVENT | POST_TOOL_USE_FAILURE_HOOK_EVENT => {
                if let Some(request_position) = self.find_request_for_tool_use(hook_event) {
                    release_request(self.pending_requests.remove(request_position));
                }
            }
            STOP_HOOK_EVENT | USER_PROMPT_SUBMIT_HOOK_EVENT | SESSION_END_HOOK_EVENT => {
                self.release_session_requests(&hook_event.session_id)
            }
            _ => {}
        }
    }

    pub fn release_session_requests(&mut self, session_id: &str) {
        self.release_requests_where(|pending| pending.session_id == session_id);
    }

    pub fn release_requests_older_than(&mut self, max_age: Duration, current_time: Instant) {
        self.release_requests_where(|pending| current_time.duration_since(pending.arrival_time) >= max_age);
    }

    /// The integration is switched off: every waiting hook gets "no decision" now.
    pub fn release_all_requests(&mut self) {
        self.release_requests_where(|_| true);
    }

    /// When the oldest request reaches `max_age`, if any is waiting.
    pub fn next_expiry(&self, max_age: Duration) -> Option<Instant> {
        self.pending_requests.first().map(|oldest_request| oldest_request.arrival_time + max_age)
    }

    pub fn describe_alert(&self) -> Option<ClaudeCodePermissionAlert> {
        let oldest_request = self.pending_requests.first()?;
        Some(ClaudeCodePermissionAlert {
            request_id: oldest_request.request_id,
            project_folder_name: oldest_request.project_folder_name.clone(),
            request_description: oldest_request.request_description.clone(),
            other_waiting_request_count: self.pending_requests.len() - 1,
        })
    }

    /// Every session with a request waiting; each keeps "needs your OK" in the list.
    pub fn sessions_waiting_for_an_answer(&self) -> Vec<String> {
        self.pending_requests.iter().map(|pending| pending.session_id.clone()).collect()
    }

    /// `PostToolUse` has a `tool_use_id` but `PermissionRequest` doesn't, so the request is found
    /// by its session and tool: the same input first, else the oldest one for that tool (in case
    /// Claude Code passed the input on changed).
    fn find_request_for_tool_use(&self, hook_event: &ClaudeCodeHookEvent) -> Option<usize> {
        let is_same_tool = |pending: &PendingPermissionRequest| {
            pending.session_id == hook_event.session_id && pending.tool_name == hook_event.tool_name
        };
        self.pending_requests
            .iter()
            .position(|pending| is_same_tool(pending) && pending.tool_input == hook_event.tool_input)
            .or_else(|| self.pending_requests.iter().position(is_same_tool))
    }

    fn release_requests_where(&mut self, should_release: impl Fn(&PendingPermissionRequest) -> bool) {
        let (released_requests, kept_requests) =
            std::mem::take(&mut self.pending_requests).into_iter().partition(|pending| should_release(pending));
        self.pending_requests = kept_requests;
        released_requests.into_iter().for_each(release_request);
    }
}

/// Answered elsewhere or too old: the hook ends with nothing to say. A hook Claude Code already
/// closed (after a deny there) just doesn't receive it.
fn release_request(released_request: PendingPermissionRequest) {
    released_request.hook_reply.send_reply(NO_HOOK_DECISION);
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use serde_json::json;

    use super::*;

    type SentReplies = Arc<Mutex<Vec<(String, String)>>>;

    /// Stands in for the pipe: records which request got which reply.
    struct RecordedHookReply {
        request_label: String,
        sent_replies: SentReplies,
    }

    impl PendingHookReply for RecordedHookReply {
        fn send_reply(self: Box<Self>, reply_text: &str) {
            self.sent_replies.lock().unwrap().push((self.request_label, reply_text.to_string()));
        }
    }

    fn hook_event(session_id: &str, hook_event_name: &str, command: &str) -> ClaudeCodeHookEvent {
        ClaudeCodeHookEvent {
            session_id: session_id.to_string(),
            hook_event_name: hook_event_name.to_string(),
            cwd: Some(format!(r"C:\Projects\{session_id}")),
            transcript_path: None,
            tool_name: Some("Bash".to_string()),
            tool_input: Some(json!({ "command": command })),
            notification_type: None,
        }
    }

    fn queue_with_requests(requests: &[(&str, &str)], start_time: Instant) -> (ClaudeCodePermissionQueue, SentReplies) {
        let sent_replies = SentReplies::default();
        let mut permission_queue = ClaudeCodePermissionQueue::default();
        for (session_id, command) in requests {
            let hook_reply = RecordedHookReply { request_label: command.to_string(), sent_replies: Arc::clone(&sent_replies) };
            permission_queue.add_request(&hook_event(session_id, PERMISSION_REQUEST_HOOK_EVENT, command), Box::new(hook_reply), start_time);
        }
        (permission_queue, sent_replies)
    }

    fn replies(sent_replies: &SentReplies) -> Vec<(String, String)> {
        sent_replies.lock().unwrap().clone()
    }

    fn reply(request_label: &str, reply_text: &str) -> (String, String) {
        (request_label.to_string(), reply_text.to_string())
    }

    #[test]
    fn the_oldest_request_is_shown_with_the_count_behind_it() {
        let (permission_queue, _) = queue_with_requests(&[("a", "first"), ("b", "second")], Instant::now());
        let alert = permission_queue.describe_alert().unwrap();
        assert_eq!((alert.project_folder_name.as_str(), alert.other_waiting_request_count), ("a", 1));
        assert_eq!(
            alert.request_description,
            ClaudeCodePermissionDescription::Command { command_text: "first".into(), is_command_cut: false }
        );
        assert_eq!(permission_queue.sessions_waiting_for_an_answer(), ["a", "b"]);
        assert!(ClaudeCodePermissionQueue::default().describe_alert().is_none());
    }

    #[test]
    fn a_question_for_the_user_is_left_to_claude_code_at_once() {
        let (mut permission_queue, sent_replies) = queue_with_requests(&[], Instant::now());
        for question_tool_name in TOOLS_ANSWERED_ONLY_IN_CLAUDE_CODE {
            let mut question_event = hook_event("a", PERMISSION_REQUEST_HOOK_EVENT, "");
            question_event.tool_name = Some(question_tool_name.to_string());
            let hook_reply = RecordedHookReply { request_label: question_tool_name.to_string(), sent_replies: Arc::clone(&sent_replies) };
            permission_queue.add_request(&question_event, Box::new(hook_reply), Instant::now());
        }
        assert_eq!(replies(&sent_replies), vec![reply("AskUserQuestion", NO_HOOK_DECISION), reply("ExitPlanMode", NO_HOOK_DECISION)]);
        assert!(permission_queue.describe_alert().is_none());
    }

    #[test]
    fn a_file_request_says_whether_the_file_is_in_the_session_project_or_elsewhere() {
        let mut permission_queue = ClaudeCodePermissionQueue::new(Some(r"C:\Users\someone".to_string()));
        let sent_replies = SentReplies::default();
        for file_path in [r"C:\Projects\a\src\main.rs", r"C:\Users\someone\.claude\settings.json"] {
            let mut edit_event = hook_event("a", PERMISSION_REQUEST_HOOK_EVENT, "");
            edit_event.tool_name = Some("Edit".to_string());
            edit_event.tool_input = Some(json!({ "file_path": file_path }));
            let hook_reply = RecordedHookReply { request_label: file_path.to_string(), sent_replies: Arc::clone(&sent_replies) };
            permission_queue.add_request(&edit_event, Box::new(hook_reply), Instant::now());
        }
        let shown_action = |permission_queue: &ClaudeCodePermissionQueue| match permission_queue.describe_alert() {
            Some(ClaudeCodePermissionAlert {
                request_description: ClaudeCodePermissionDescription::Action { action_text }, ..
            }) => action_text,
            other_alert => panic!("expected a file action, got {other_alert:?}"),
        };
        assert_eq!(shown_action(&permission_queue), r"Edit src\main.rs");
        let first_request_id = permission_queue.describe_alert().unwrap().request_id;
        permission_queue.answer_request(first_request_id, "allow");
        assert_eq!(shown_action(&permission_queue), r"Edit ~\.claude\settings.json");
    }

    #[test]
    fn an_answer_reaches_only_its_own_request_and_a_stale_id_sends_nothing() {
        let (mut permission_queue, sent_replies) = queue_with_requests(&[("a", "first"), ("b", "second")], Instant::now());
        let second_id = permission_queue.describe_alert().unwrap().request_id + 1;
        assert_eq!(permission_queue.answer_request(second_id, "allow").as_deref(), Some("b"));
        assert_eq!(permission_queue.answer_request(second_id, "allow"), None);
        assert_eq!(replies(&sent_replies), vec![reply("second", "allow")]);
        assert_eq!(permission_queue.describe_alert().unwrap().other_waiting_request_count, 0);
    }

    #[test]
    fn a_tool_that_ran_releases_its_own_request_even_with_another_waiting_in_the_session() {
        let (mut permission_queue, sent_replies) = queue_with_requests(&[("a", "first"), ("a", "second")], Instant::now());
        permission_queue.release_requests_answered_elsewhere(&hook_event("a", POST_TOOL_USE_HOOK_EVENT, "second"));
        assert_eq!(replies(&sent_replies), vec![reply("second", NO_HOOK_DECISION)]);
        // A changed input still finds the session's request for that tool.
        permission_queue.release_requests_answered_elsewhere(&hook_event("a", POST_TOOL_USE_FAILURE_HOOK_EVENT, "changed"));
        assert_eq!(replies(&sent_replies).len(), 2);
        assert!(permission_queue.describe_alert().is_none());
    }

    #[test]
    fn the_end_of_a_turn_releases_all_of_that_sessions_requests_only() {
        let (mut permission_queue, sent_replies) =
            queue_with_requests(&[("a", "first"), ("b", "second"), ("a", "third")], Instant::now());
        permission_queue.release_requests_answered_elsewhere(&hook_event("a", PRE_TOOL_USE_HOOK_EVENT, "other"));
        assert!(replies(&sent_replies).is_empty());
        permission_queue.release_requests_answered_elsewhere(&hook_event("a", STOP_HOOK_EVENT, ""));
        assert_eq!(replies(&sent_replies), vec![reply("first", NO_HOOK_DECISION), reply("third", NO_HOOK_DECISION)]);
        assert_eq!(permission_queue.describe_alert().unwrap().project_folder_name, "b");
        permission_queue.release_session_requests("b");
        assert!(permission_queue.describe_alert().is_none());
    }

    #[test]
    fn a_request_nobody_answers_leaves_at_its_maximum_age() {
        let start_time = Instant::now();
        let max_age = Duration::from_secs(600);
        let (mut permission_queue, sent_replies) = queue_with_requests(&[("a", "first")], start_time);
        assert_eq!(permission_queue.next_expiry(max_age), Some(start_time + max_age));
        permission_queue.release_requests_older_than(max_age, start_time + Duration::from_secs(599));
        assert!(permission_queue.describe_alert().is_some());
        permission_queue.release_requests_older_than(max_age, start_time + max_age);
        assert_eq!(replies(&sent_replies), vec![reply("first", NO_HOOK_DECISION)]);
        assert_eq!(permission_queue.next_expiry(max_age), None);
    }
}
