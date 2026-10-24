//! The Claude Code status thread: owns the session tracker, the permission queue and the
//! interrupt watch, and publishes the activity after every change. Moved out of the activity
//! source when the permission requests made the loop grow.

use std::path::Path;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant, SystemTime};

use crate::activity_core::ActivityPublisher;
use crate::activity_sources::claude_code::claude_code_hook_event::{
    ClaudeCodeHookEvent, PERMISSION_REQUEST_HOOK_EVENT, SESSION_START_HOOK_EVENT, USER_PROMPT_SUBMIT_HOOK_EVENT,
};
use crate::activity_sources::claude_code::claude_code_interrupt_watch::ClaudeCodeInterruptWatch;
use crate::activity_sources::claude_code::claude_code_permission_answer::build_permission_hook_reply;
use crate::activity_sources::claude_code::claude_code_permission_answering_switch::ClaudeCodePermissionAnsweringSwitch;
use crate::activity_sources::claude_code::claude_code_permission_queue::ClaudeCodePermissionQueue;
use crate::activity_sources::claude_code::claude_code_session_list::ClaudeCodeSessionStatus;
use crate::activity_sources::claude_code::claude_code_session_tracker::ClaudeCodeSessionTracker;
use crate::activity_sources::claude_code::claude_code_status_message::ClaudeCodeStatusMessage;
use crate::activity_sources::claude_code::claude_code_transcript_continuation::find_sessions_continued_in;
use crate::activity_sources::claude_code::claude_code_transcript_interrupt::ClaudeCodeTranscriptInterrupt;
use crate::activity_sources::claude_code::claude_code_transcript_usage_limit::find_usage_limit_reset;
use crate::backend_constants::{CLAUDE_CODE_PERMISSION_REQUEST_MAX_AGE, CLAUDE_CODE_SESSION_SILENCE_TIMEOUT};
use crate::claude_code_hook_channel::{PendingHookReply, NO_HOOK_DECISION};

/// Sleeps until a message arrives or the next deadline (the quietest session's silence timeout,
/// the oldest request's maximum age). Ends on `StopPublishing` (also sent when the pipe server
/// couldn't start); every hook still waiting then gets "no decision". While answering in the pill is
/// switched off, every request leaves right after it arrives.
pub fn publish_claude_code_status(
    status_message_receiver: Receiver<ClaudeCodeStatusMessage>,
    interrupt_watch_sender: Sender<ClaudeCodeStatusMessage>,
    activity_publisher: ActivityPublisher,
    permission_answering_switch: ClaudeCodePermissionAnsweringSwitch,
) {
    let mut session_tracker = ClaudeCodeSessionTracker::default();
    let home_folder = std::env::home_dir().map(|home_folder| home_folder.to_string_lossy().into_owned());
    let mut permission_queue = ClaudeCodePermissionQueue::new(home_folder);
    let mut interrupt_watch = ClaudeCodeInterruptWatch::new(interrupt_watch_sender);
    loop {
        let next_deadline = [
            session_tracker.next_silence_deadline(CLAUDE_CODE_SESSION_SILENCE_TIMEOUT),
            permission_queue.next_expiry(CLAUDE_CODE_PERMISSION_REQUEST_MAX_AGE),
        ]
        .into_iter()
        .flatten()
        .min();
        let next_message = match next_deadline {
            Some(deadline) => status_message_receiver.recv_timeout(deadline.saturating_duration_since(Instant::now())),
            None => status_message_receiver.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        let current_time = Instant::now();
        match next_message {
            Ok(ClaudeCodeStatusMessage::HookEvent { hook_event_json, hook_reply }) => {
                record_hook_event(&hook_event_json, hook_reply, &mut session_tracker, &mut permission_queue, current_time)
            }
            Ok(ClaudeCodeStatusMessage::TranscriptFolderChanged(changed_folder)) => {
                for (session_id, transcript_interrupt) in interrupt_watch.find_session_interrupts(&changed_folder) {
                    match transcript_interrupt {
                        // Also a deny or Esc in Claude Code's own dialog: its request leaves the pill.
                        ClaudeCodeTranscriptInterrupt::TurnCancelled => {
                            session_tracker.record_interrupt(&session_id, ClaudeCodeSessionStatus::Cancelled, current_time);
                            permission_queue.release_session_requests(&session_id);
                        }
                        ClaudeCodeTranscriptInterrupt::PermissionDenied => {
                            session_tracker.record_interrupt(&session_id, ClaudeCodeSessionStatus::Denied, current_time);
                            permission_queue.release_session_requests(&session_id);
                        }
                        ClaudeCodeTranscriptInterrupt::BackgroundSubagents => {
                            session_tracker.record_subagents_stopped(&session_id, current_time)
                        }
                    }
                }
            }
            Ok(ClaudeCodeStatusMessage::PermissionAnswered { request_id, permission_answer }) => {
                let hook_reply_text = build_permission_hook_reply(permission_answer);
                if let Some(answered_session_id) = permission_queue.answer_request(request_id, &hook_reply_text) {
                    session_tracker.record_permission_answer(&answered_session_id, permission_answer, current_time);
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                session_tracker.forget_sessions_silent_for(CLAUDE_CODE_SESSION_SILENCE_TIMEOUT, current_time);
                permission_queue.release_requests_older_than(CLAUDE_CODE_PERMISSION_REQUEST_MAX_AGE, current_time);
            }
            Ok(ClaudeCodeStatusMessage::SessionDismissed { session_id }) => {
                session_tracker.dismiss_finished_session(&session_id)
            }
            Ok(ClaudeCodeStatusMessage::PermissionAnsweringChanged) => {}
            Ok(ClaudeCodeStatusMessage::StopPublishing) | Err(RecvTimeoutError::Disconnected) => break,
        }
        if !permission_answering_switch.is_answering_on() {
            permission_queue.release_all_requests();
        }
        interrupt_watch.follow_busy_sessions(session_tracker.busy_session_transcripts());
        let sessions_waiting_for_an_answer = permission_queue.sessions_waiting_for_an_answer();
        match session_tracker.describe_activity(permission_queue.describe_alert(), &sessions_waiting_for_an_answer) {
            Some(claude_code_update) => activity_publisher.publish_activity_update(claude_code_update),
            None => activity_publisher.withdraw_activity(),
        }
    }
    permission_queue.release_all_requests();
    activity_publisher.withdraw_activity();
}

/// A permission request keeps its hook waiting in the queue; every other event is answered with
/// "no decision" at once, which ends its hook process.
fn record_hook_event(
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
