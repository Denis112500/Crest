//! The Claude Code status thread: owns the session tracker, the permission queue and the
//! transcript watch, and publishes the activity after every change. Moved out of the activity
//! source when the permission requests made the loop grow.

use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::Instant;

use crate::activity_core::ActivityPublisher;
use crate::activity_sources::claude_code::claude_code_hook_event_intake::take_in_hook_event;
use crate::activity_sources::claude_code::claude_code_permission_answer::build_permission_hook_reply;
use crate::activity_sources::claude_code::claude_code_permission_answering_switch::ClaudeCodePermissionAnsweringSwitch;
use crate::activity_sources::claude_code::claude_code_permission_queue::ClaudeCodePermissionQueue;
use crate::activity_sources::claude_code::claude_code_session_list::ClaudeCodeSessionStatus;
use crate::activity_sources::claude_code::claude_code_session_tracker::ClaudeCodeSessionTracker;
use crate::activity_sources::claude_code::claude_code_status_message::ClaudeCodeStatusMessage;
use crate::activity_sources::claude_code::claude_code_transcript_interrupt::ClaudeCodeTranscriptInterrupt;
use crate::activity_sources::claude_code::claude_code_transcript_watch::{ClaudeCodeTranscriptNews, ClaudeCodeTranscriptWatch};
use crate::backend_constants::{CLAUDE_CODE_PERMISSION_REQUEST_MAX_AGE, CLAUDE_CODE_SESSION_SILENCE_TIMEOUT};

/// Sleeps until a message arrives or the next deadline (the quietest session's silence timeout,
/// the oldest request's maximum age). Ends on `StopPublishing` (also sent when the pipe server
/// couldn't start); every hook still waiting then gets "no decision". While answering in the pill is
/// switched off, every request leaves right after it arrives.
pub fn publish_claude_code_status(
    status_message_receiver: Receiver<ClaudeCodeStatusMessage>,
    transcript_watch_sender: Sender<ClaudeCodeStatusMessage>,
    activity_publisher: ActivityPublisher,
    permission_answering_switch: ClaudeCodePermissionAnsweringSwitch,
) {
    let mut session_tracker = ClaudeCodeSessionTracker::default();
    let home_folder = std::env::home_dir().map(|home_folder| home_folder.to_string_lossy().into_owned());
    let mut permission_queue = ClaudeCodePermissionQueue::new(home_folder);
    let mut transcript_watch = ClaudeCodeTranscriptWatch::new(transcript_watch_sender);
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
                take_in_hook_event(&hook_event_json, hook_reply, &mut session_tracker, &mut permission_queue, current_time)
            }
            Ok(ClaudeCodeStatusMessage::TranscriptFolderChanged(changed_folder)) => {
                for (session_id, transcript_news) in transcript_watch.find_session_news(&changed_folder) {
                    match transcript_news {
                        // Also a deny or Esc in Claude Code's own dialog: its request leaves the pill.
                        ClaudeCodeTranscriptNews::Interrupt(ClaudeCodeTranscriptInterrupt::TurnCancelled) => {
                            session_tracker.record_interrupt(&session_id, ClaudeCodeSessionStatus::Cancelled, current_time);
                            permission_queue.release_session_requests(&session_id);
                        }
                        ClaudeCodeTranscriptNews::Interrupt(ClaudeCodeTranscriptInterrupt::PermissionDenied) => {
                            session_tracker.record_interrupt(&session_id, ClaudeCodeSessionStatus::Denied, current_time);
                            permission_queue.release_session_requests(&session_id);
                        }
                        ClaudeCodeTranscriptNews::Interrupt(ClaudeCodeTranscriptInterrupt::BackgroundSubagents) => {
                            session_tracker.record_subagents_stopped(&session_id, current_time)
                        }
                        ClaudeCodeTranscriptNews::SessionTitle(session_title) => {
                            session_tracker.record_session_title(&session_id, session_title)
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
        transcript_watch.follow_busy_sessions(session_tracker.busy_session_transcripts());
        let sessions_waiting_for_an_answer = permission_queue.sessions_waiting_for_an_answer();
        match session_tracker.describe_activity(permission_queue.describe_alert(), &sessions_waiting_for_an_answer) {
            Some(claude_code_update) => activity_publisher.publish_activity_update(claude_code_update),
            None => activity_publisher.withdraw_activity(),
        }
    }
    permission_queue.release_all_requests();
    activity_publisher.withdraw_activity();
}
