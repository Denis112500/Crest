//! The Claude Code plugin: receives Claude Code's hook events through the local channel and
//! turns them into pill activity. Exists only while the integration is switched on.

use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Instant;

use crate::activity_core::{ActivityPublisher, ActivitySource};
use crate::activity_sources::claude_code::claude_code_hook_event::ClaudeCodeHookEvent;
use crate::activity_sources::claude_code::claude_code_interrupt_watch::ClaudeCodeInterruptWatch;
use crate::activity_sources::claude_code::claude_code_session_tracker::ClaudeCodeSessionTracker;
use crate::activity_sources::claude_code::claude_code_status_message::ClaudeCodeStatusMessage;
use crate::backend_constants::CLAUDE_CODE_SESSION_SILENCE_TIMEOUT;
use crate::claude_code_hook_channel::{ClaudeCodeHookChannel, CurrentPlatformClaudeCodeHookChannel, RunningHookEventServer};
use crate::ipc_channel_names::CLAUDE_CODE_ACTIVITY_KIND;

const CLAUDE_CODE_STATUS_THREAD_NAME: &str = "crest-claude-code-status";

#[derive(Default)]
pub struct ClaudeCodeActivitySource {
    running_hook_event_server: Option<Box<dyn RunningHookEventServer>>,
    status_message_sender: Option<Sender<ClaudeCodeStatusMessage>>,
}

impl ActivitySource for ClaudeCodeActivitySource {
    fn activity_kind(&self) -> &'static str {
        CLAUDE_CODE_ACTIVITY_KIND
    }

    /// Two threads: the channel's server answers hooks at once (status events need no decision)
    /// and hands each event to the status thread, which owns the session tracker.
    fn start_publishing(&mut self, activity_publisher: ActivityPublisher) -> Result<(), String> {
        let (status_message_sender, status_message_receiver) = mpsc::channel::<ClaudeCodeStatusMessage>();
        let interrupt_watch_sender = status_message_sender.clone();
        self.status_message_sender = Some(status_message_sender.clone());
        thread::Builder::new()
            .name(CLAUDE_CODE_STATUS_THREAD_NAME.to_string())
            .spawn(move || publish_claude_code_status(status_message_receiver, interrupt_watch_sender, activity_publisher))
            .map_err(|error| error.to_string())?;
        let running_hook_event_server =
            CurrentPlatformClaudeCodeHookChannel::start_hook_event_server(Box::new(move |hook_event_json| {
                let _ = status_message_sender.send(ClaudeCodeStatusMessage::HookEvent(hook_event_json.to_string()));
                String::new()
            }))?;
        self.running_hook_event_server = Some(running_hook_event_server);
        Ok(())
    }

    /// The server stops first, so no event arrives after the status thread's last word; that
    /// thread then stops its folder watches and withdraws the activity on its way out.
    fn stop_publishing(&mut self) {
        if let Some(running_hook_event_server) = self.running_hook_event_server.take() {
            running_hook_event_server.stop();
        }
        if let Some(status_message_sender) = self.status_message_sender.take() {
            let _ = status_message_sender.send(ClaudeCodeStatusMessage::StopPublishing);
        }
    }

    fn perform_activity_action(&self, activity_action: &str) -> Result<(), String> {
        Err(format!("the Claude Code activity has no action \"{activity_action}\""))
    }
}

/// Sleeps until an event arrives, a watched transcript changes, or the quietest session reaches
/// its silence timeout. Ends on `StopPublishing`, or if starting the pipe server failed.
fn publish_claude_code_status(
    status_message_receiver: Receiver<ClaudeCodeStatusMessage>,
    interrupt_watch_sender: Sender<ClaudeCodeStatusMessage>,
    activity_publisher: ActivityPublisher,
) {
    let mut session_tracker = ClaudeCodeSessionTracker::default();
    let mut interrupt_watch = ClaudeCodeInterruptWatch::new(interrupt_watch_sender);
    loop {
        let next_message = match session_tracker.next_silence_deadline(CLAUDE_CODE_SESSION_SILENCE_TIMEOUT) {
            Some(silence_deadline) => {
                status_message_receiver.recv_timeout(silence_deadline.saturating_duration_since(Instant::now()))
            }
            None => status_message_receiver.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        match next_message {
            Ok(ClaudeCodeStatusMessage::HookEvent(hook_event_json)) => {
                match serde_json::from_str::<ClaudeCodeHookEvent>(&hook_event_json) {
                    Ok(hook_event) => session_tracker.record_hook_event(&hook_event, Instant::now()),
                    Err(parse_error) => eprintln!("Crest: ignoring a Claude Code hook event it can't read: {parse_error}"),
                }
            }
            Ok(ClaudeCodeStatusMessage::TranscriptFolderChanged(changed_folder)) => {
                for interrupted_session_id in interrupt_watch.find_interrupted_sessions(&changed_folder) {
                    session_tracker.record_interrupt(&interrupted_session_id, Instant::now());
                }
            }
            Err(RecvTimeoutError::Timeout) => {
                session_tracker.forget_sessions_silent_for(CLAUDE_CODE_SESSION_SILENCE_TIMEOUT, Instant::now())
            }
            Ok(ClaudeCodeStatusMessage::StopPublishing) | Err(RecvTimeoutError::Disconnected) => break,
        }
        interrupt_watch.follow_working_sessions(session_tracker.working_session_transcripts());
        match session_tracker.describe_activity() {
            Some(claude_code_update) => activity_publisher.publish_activity_update(claude_code_update),
            None => activity_publisher.withdraw_activity(),
        }
    }
    activity_publisher.withdraw_activity();
}
