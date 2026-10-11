//! Windows: a connected pipe instance whose hook still waits for Crest's reply. It may be
//! answered from any thread; the server has moved on to the next hook.

use std::fs::File;

use crate::claude_code_hook_channel::length_prefixed_message::write_length_prefixed_message;
use crate::claude_code_hook_channel::{PendingHookReply, NO_HOOK_DECISION};

pub struct NamedPipePendingHookReply {
    /// `None` once answered, so `drop` knows not to answer again.
    connected_pipe: Option<File>,
}

impl NamedPipePendingHookReply {
    pub fn new(connected_pipe: File) -> Self {
        Self { connected_pipe: Some(connected_pipe) }
    }

    fn write_reply_once(&mut self, reply_text: &str) {
        let Some(mut connected_pipe) = self.connected_pipe.take() else {
            return;
        };
        // An error means the hook is gone (Claude Code answered elsewhere or ended it): expected.
        if write_length_prefixed_message(&mut connected_pipe, reply_text.as_bytes()).is_ok() {
            // On a pipe this waits until the hook has read the reply, so closing can't cut it off.
            let _ = connected_pipe.sync_all();
        }
    }
}

impl PendingHookReply for NamedPipePendingHookReply {
    fn send_reply(mut self: Box<Self>, reply_text: &str) {
        self.write_reply_once(reply_text);
    }
}

impl Drop for NamedPipePendingHookReply {
    fn drop(&mut self) {
        self.write_reply_once(NO_HOOK_DECISION);
    }
}
