//! A hook's answer that Crest may give later: a permission request waits for a click in the
//! pill, while the channel's server is already free for the next hook.

/// What a hook prints when Crest has nothing to decide; Claude Code then goes on as usual.
pub const NO_HOOK_DECISION: &str = "";

/// The waiting hook at the other end of the channel. Answered at most once; an implementation
/// dropped unanswered sends `NO_HOOK_DECISION` itself, so no hook can be left waiting for
/// Crest until Claude Code's timeout.
pub trait PendingHookReply: Send {
    /// Hands `reply_text` to the hook, which prints it for Claude Code. Fails quietly when the
    /// hook is gone: Claude Code then got its answer elsewhere and closed the hook.
    fn send_reply(self: Box<Self>, reply_text: &str);
}
