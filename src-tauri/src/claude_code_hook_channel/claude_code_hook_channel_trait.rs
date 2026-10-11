//! The trait each OS implements to carry Claude Code's hook events from the short-lived hook
//! process to the running Crest, and an answer back, without any network.

use crate::claude_code_hook_channel::pending_hook_reply::PendingHookReply;

/// Called with each hook event's JSON and the hook's reply, which may be sent later (from any
/// thread) or dropped for "no decision".
pub type HookEventHandler = Box<dyn Fn(&str, Box<dyn PendingHookReply>) + Send>;

/// A server started by `start_hook_event_server`; stopping it ends its thread and drops the handler.
pub trait RunningHookEventServer: Send {
    fn stop(self: Box<Self>);
}

/// Both ends of the local channel between Claude Code's hooks and Crest. Each operating system
/// gets its own implementation in its own folder.
pub trait ClaudeCodeHookChannel {
    /// Crest's end: starts listening on its own thread, which sleeps until a hook connects.
    fn start_hook_event_server(hook_event_handler: HookEventHandler) -> Result<Box<dyn RunningHookEventServer>, String>;

    /// The hook's end: hands one event to the running Crest and waits for its reply. `None`
    /// when no Crest of this user is listening; the hook then simply has nothing to say.
    fn forward_hook_event(hook_event_json: &[u8]) -> Option<Vec<u8>>;
}
