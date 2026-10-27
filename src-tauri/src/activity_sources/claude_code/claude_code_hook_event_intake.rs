//! What the status thread does with one hook event: read it, answer or hold its hook, record it,
//! and the transcript look-ups it calls for (a title, a usage limit's reset, a continued session).
//! Split from `claude_code_status_thread.rs` after CC.2, so the thread keeps only its loop.

use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

use crate::activity_sources::claude_code::claude_code_hook_event::{
    ClaudeCodeHookEvent, PERMISSION_REQUEST_HOOK_EVENT, SESSION_START_HOOK_EVENT, USER_PROMPT_SUBMIT_HOOK_EVENT,
};
use crate::activity_sources::claude_code::claude_code_permission_queue::ClaudeCodePermissionQueue;
use crate::activity_sources::claude_code::claude_code_session_tracker::ClaudeCodeSessionTracker;
use crate::activity_sources::claude_code::claude_code_transcript_continuation::find_sessions_continued_in;
use crate::activity_sources::claude_code::claude_code_transcript_title_lookup::find_title_at_transcript_start;
use crate::activity_sources::claude_code::claude_code_transcript_usage_limit::find_usage_limit_reset;
use crate::claude_code_hook_channel::{PendingHookReply, NO_HOOK_DECISION};

/// A permission request keeps its hook waiting in the queue; every other event is answered with
/// "no decision" at once, which ends its hook process.
pub fn take_in_hook_event(
    hook_event_json: &str,
    hook_reply: Box<dyn PendingHookReply>,
    session_tracker: &mut ClaudeCodeSessionTracker,
    permission_queue: &mut ClaudeCodePermissionQueue,
    current_time: Instant,
) {
    let hook_event = match serde_json::from_str::<ClaudeCodeHookEvent>(hook_event_json) {
        Ok(hook_event) => hook_event,
        Err(parse_error) => {
            eprintln!("Crest: ignoring a Claude Code hook event it can't read: {parse_error}");
            hook_reply.send_reply(NO_HOOK_DECISION);
            return;
        }
    };
    permission_queue.release_requests_answered_elsewhere(&hook_event);
    if hook_event.hook_event_name == PERMISSION_REQUEST_HOOK_EVENT {
        permission_queue.add_request(&hook_event, hook_reply, current_time);
    } else {
        hook_reply.send_reply(NO_HOOK_DECISION);
    }
    session_tracker.record_hook_event(&hook_event, current_time);
    // A session met mid-way has its made-up title only near its transcript's start.
    if let Some(transcript_path) = session_tracker.take_title_lookup_transcript(&hook_event.session_id) {
        if let Some(session_title) = find_title_at_transcript_start(&transcript_path) {
            session_tracker.record_session_title(&hook_event.session_id, session_title);
        }
    }
    if let (true, Some(transcript_path)) = (hook_event.is_usage_limit_stop(), &hook_event.transcript_path) {
        record_usage_limit_reset(&hook_event.session_id, Path::new(transcript_path), session_tracker, current_time);
    }
    // A session that continued under this id carries on here: its old id leaves. Checked on every
    // prompt too, since `SessionStart` may come before Claude Code writes the line that says so.
    let is_session_start_or_prompt =
        [SESSION_START_HOOK_EVENT, USER_PROMPT_SUBMIT_HOOK_EVENT].contains(&hook_event.hook_event_name.as_str());
    if let (true, Some(transcript_path)) = (is_session_start_or_prompt, &hook_event.transcript_path) {
        let tracked_session_transcripts = session_tracker.session_transcripts();
        for continued_session_id in
            find_sessions_continued_in(&hook_event.session_id, Path::new(transcript_path), tracked_session_transcripts)
        {
            session_tracker.forget_session(&continued_session_id);
            permission_queue.release_session_requests(&continued_session_id);
        }
    }
}

/// Read once, as the turn stops. A reset that has already passed belongs to an earlier limit
/// (this one's entry isn't written yet): the pill then only says the limit is reached.
fn record_usage_limit_reset(
    session_id: &str,
    transcript_path: &Path,
    session_tracker: &mut ClaudeCodeSessionTracker,
    current_time: Instant,
) {
    let Some(resets_at_unix_seconds) = find_usage_limit_reset(transcript_path) else {
        return;
    };
    let reset_moment = SystemTime::UNIX_EPOCH + Duration::from_secs(resets_at_unix_seconds);
    if let Ok(time_until_reset) = reset_moment.duration_since(SystemTime::now()) {
        session_tracker.record_usage_limit_reset(session_id, resets_at_unix_seconds, time_until_reset, current_time);
    }
}
