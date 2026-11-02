//! What `crest.exe --claude-code-hook` does: Claude Code starts it for every hook event with the
//! event on stdin; it passes the event to the running Crest and prints Crest's reply for Claude
//! Code. It never starts Tauri, a window or the single-instance check.

use std::io::{Read, Write};

use crate::activity_sources::claude_code::{add_session_project_folder, CLAUDE_CODE_PROJECT_FOLDER_VARIABLE};
use crate::claude_code_hook_channel::claude_code_hook_deadline::{forward_within_hook_deadline, hook_deadline_for_event};
use crate::claude_code_hook_channel::{ClaudeCodeHookChannel, CurrentPlatformClaudeCodeHookChannel};

/// Always ends normally (exit code 0): a hook that fails would show an error in Claude Code,
/// and no output simply means "nothing to decide". `main` returns right after, which also ends a
/// forwarding thread still waiting past its deadline.
pub fn run_as_claude_code_hook() {
    let mut hook_event_json = Vec::new();
    if std::io::stdin().read_to_end(&mut hook_event_json).is_err() {
        return;
    }
    // Only the hook's environment says where the session started; the running Crest can't ask.
    let session_project_folder = std::env::var(CLAUDE_CODE_PROJECT_FOLDER_VARIABLE).ok();
    let hook_event_json = add_session_project_folder(hook_event_json, session_project_folder.as_deref());
    let hook_deadline = hook_deadline_for_event(&hook_event_json);
    let crest_reply = forward_within_hook_deadline(hook_deadline, move || {
        CurrentPlatformClaudeCodeHookChannel::forward_hook_event(&hook_event_json)
    });
    if let Some(crest_reply) = crest_reply {
        let _ = std::io::stdout().write_all(&crest_reply);
    }
}
