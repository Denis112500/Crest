//! What wakes the Claude Code status thread.

use std::path::PathBuf;

use crate::activity_sources::claude_code::claude_code_permission_answer::ClaudeCodePermissionAnswer;
use crate::claude_code_hook_channel::PendingHookReply;

pub enum ClaudeCodeStatusMessage {
    /// A hook event's JSON from the pipe server, with the hook still waiting for its reply.
    HookEvent { hook_event_json: String, hook_reply: Box<dyn PendingHookReply> },
    /// A file in a watched transcript folder was written (maybe an interrupt line).
    TranscriptFolderChanged(PathBuf),
    /// Allow or Deny was clicked in the pill's alert for the request with this id.
    PermissionAnswered { request_id: u64, permission_answer: ClaudeCodePermissionAnswer },
    /// The integration was switched off. Needed as a message because the folder watches hold
    /// senders too, so the channel never closes on its own.
    StopPublishing,
}
