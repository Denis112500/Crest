//! The Claude Code plugin: receives Claude Code's hook events through the local channel and
//! turns them into pill activity. Exists only while the integration is switched on.

use std::sync::mpsc::{self, Sender};
use std::thread;

use crate::activity_core::{ActivityPublisher, ActivitySource};
use crate::activity_sources::claude_code::claude_code_permission_answer::parse_permission_action;
use crate::activity_sources::claude_code::claude_code_permission_answering_switch::ClaudeCodePermissionAnsweringSwitch;
use crate::activity_sources::claude_code::claude_code_status_message::ClaudeCodeStatusMessage;
use crate::activity_sources::claude_code::claude_code_status_thread::publish_claude_code_status;
use crate::claude_code_hook_channel::{ClaudeCodeHookChannel, CurrentPlatformClaudeCodeHookChannel, RunningHookEventServer};
use crate::ipc_channel_names::CLAUDE_CODE_ACTIVITY_KIND;

const CLAUDE_CODE_STATUS_THREAD_NAME: &str = "crest-claude-code-status";

pub struct ClaudeCodeActivitySource {
    running_hook_event_server: Option<Box<dyn RunningHookEventServer>>,
    status_message_sender: Option<Sender<ClaudeCodeStatusMessage>>,
    permission_answering_switch: ClaudeCodePermissionAnsweringSwitch,
}

impl ClaudeCodeActivitySource {
    pub fn new(permission_answering_switch: ClaudeCodePermissionAnsweringSwitch) -> Self {
        Self { running_hook_event_server: None, status_message_sender: None, permission_answering_switch }
    }
}

impl ActivitySource for ClaudeCodeActivitySource {
    fn activity_kind(&self) -> &'static str {
        CLAUDE_CODE_ACTIVITY_KIND
    }

    /// Two threads: the channel's server only receives hooks and hands each event, with its
    /// waiting hook, to the status thread, which owns everything else and answers the hooks.
    fn start_publishing(&mut self, activity_publisher: ActivityPublisher) -> Result<(), String> {
        let (status_message_sender, status_message_receiver) = mpsc::channel::<ClaudeCodeStatusMessage>();
        let interrupt_watch_sender = status_message_sender.clone();
        self.status_message_sender = Some(status_message_sender.clone());
        self.permission_answering_switch.connect_status_thread(status_message_sender.clone());
        let permission_answering_switch = self.permission_answering_switch.clone();
        thread::Builder::new()
            .name(CLAUDE_CODE_STATUS_THREAD_NAME.to_string())
            .spawn(move || {
                publish_claude_code_status(
                    status_message_receiver,
                    interrupt_watch_sender,
                    activity_publisher,
                    permission_answering_switch,
                )
            })
            .map_err(|error| error.to_string())?;
        let running_hook_event_server =
            CurrentPlatformClaudeCodeHookChannel::start_hook_event_server(Box::new(move |hook_event_json, hook_reply| {
                // If the status thread is gone, the message is dropped with its reply, which
                // then answers "no decision" by itself.
                let _ = status_message_sender
                    .send(ClaudeCodeStatusMessage::HookEvent { hook_event_json: hook_event_json.to_string(), hook_reply });
            }))?;
        self.running_hook_event_server = Some(running_hook_event_server);
        Ok(())
    }

    /// The server stops first, so no event arrives after the status thread's last word; that
    /// thread then answers the hooks still waiting, stops its folder watches and withdraws the
    /// activity on its way out.
    fn stop_publishing(&mut self) {
        if let Some(running_hook_event_server) = self.running_hook_event_server.take() {
            running_hook_event_server.stop();
        }
        if let Some(status_message_sender) = self.status_message_sender.take() {
            let _ = status_message_sender.send(ClaudeCodeStatusMessage::StopPublishing);
        }
    }

    /// Allow or Deny from the pill's alert; the status thread sends it to the waiting hook.
    fn perform_activity_action(&self, activity_action: &str) -> Result<(), String> {
        let (permission_answer, request_id) = parse_permission_action(activity_action)
            .ok_or_else(|| format!("the Claude Code activity has no action \"{activity_action}\""))?;
        self.status_message_sender
            .as_ref()
            .ok_or("the Claude Code integration isn't running")?
            .send(ClaudeCodeStatusMessage::PermissionAnswered { request_id, permission_answer })
            .map_err(|_| "the Claude Code status thread has stopped".to_string())
    }
}
