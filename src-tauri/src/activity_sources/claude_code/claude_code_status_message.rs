//! What wakes the Claude Code status thread.

use std::path::PathBuf;

pub enum ClaudeCodeStatusMessage {
    /// A hook event's JSON, from the pipe server.
    HookEvent(String),
    /// A file in a watched transcript folder was written (maybe an interrupt line).
    TranscriptFolderChanged(PathBuf),
    /// The integration was switched off. Needed as a message because the folder watches hold
    /// senders too, so the channel never closes on its own.
    StopPublishing,
}
