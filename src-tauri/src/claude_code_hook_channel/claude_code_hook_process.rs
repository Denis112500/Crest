//! What `crest.exe --claude-code-hook` does: Claude Code starts it for every hook event with the
//! event on stdin; it passes the event to the running Crest and prints Crest's reply for Claude
//! Code. It never starts Tauri, a window or the single-instance check.

use std::io::{Read, Write};

use crate::claude_code_hook_channel::{ClaudeCodeHookChannel, CurrentPlatformClaudeCodeHookChannel};

/// Always ends normally (exit code 0): a hook that fails would show an error in Claude Code,
/// and no output simply means "nothing to decide".
pub fn run_as_claude_code_hook() {
    let mut hook_event_json = Vec::new();
    if std::io::stdin().read_to_end(&mut hook_event_json).is_err() {
        return;
    }
    if let Some(crest_reply) = CurrentPlatformClaudeCodeHookChannel::forward_hook_event(&hook_event_json) {
        let _ = std::io::stdout().write_all(&crest_reply);
    }
}
