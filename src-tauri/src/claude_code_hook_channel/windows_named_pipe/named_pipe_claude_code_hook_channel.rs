//! Ties the Windows pieces to the platform-neutral trait: the pipe's name is per user, the server
//! listens on it, the hook connects to it.

use crate::backend_constants::CLAUDE_CODE_HOOK_PIPE_NAME_PREFIX;
use crate::claude_code_hook_channel::windows_named_pipe::named_pipe_hook_event_client::forward_hook_event_through_pipe;
use crate::claude_code_hook_channel::windows_named_pipe::named_pipe_hook_event_server::NamedPipeHookEventServer;
use crate::claude_code_hook_channel::windows_named_pipe::windows_user_identity::read_current_user_sid;
use crate::claude_code_hook_channel::{ClaudeCodeHookChannel, HookEventHandler, RunningHookEventServer};

pub struct NamedPipeClaudeCodeHookChannel;

impl ClaudeCodeHookChannel for NamedPipeClaudeCodeHookChannel {
    fn start_hook_event_server(hook_event_handler: HookEventHandler) -> Result<Box<dyn RunningHookEventServer>, String> {
        let current_user_sid = read_current_user_sid()?;
        let hook_event_server =
            NamedPipeHookEventServer::start(pipe_name_for_user(&current_user_sid), current_user_sid, hook_event_handler)?;
        Ok(Box::new(hook_event_server))
    }

    fn forward_hook_event(hook_event_json: &[u8]) -> Option<Vec<u8>> {
        let current_user_sid = read_current_user_sid().ok()?;
        forward_hook_event_through_pipe(&pipe_name_for_user(&current_user_sid), &current_user_sid, hook_event_json)
    }
}

/// Pipe names are shared by every user of the PC; the SID in the name keeps them apart.
fn pipe_name_for_user(user_sid: &str) -> String {
    format!("{CLAUDE_CODE_HOOK_PIPE_NAME_PREFIX}{user_sid}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_event_crosses_a_real_pipe_and_the_reply_comes_back() {
        let current_user_sid = read_current_user_sid().unwrap();
        // Built from the real prefix, so a malformed prefix fails here instead of in the app.
        let test_pipe_name = pipe_name_for_user(&format!("test-{}", std::process::id()));
        let hook_event_server = NamedPipeHookEventServer::start(
            test_pipe_name.clone(),
            current_user_sid.clone(),
            Box::new(|hook_event_json| format!("seen: {hook_event_json}")),
        )
        .unwrap();
        let crest_reply = forward_hook_event_through_pipe(&test_pipe_name, &current_user_sid, br#"{"hook_event_name":"Stop"}"#);
        assert_eq!(crest_reply.as_deref(), Some(br#"seen: {"hook_event_name":"Stop"}"#.as_slice()));

        // A second server on the same name is refused: the name can't be shared or taken over.
        assert!(NamedPipeHookEventServer::start(test_pipe_name.clone(), current_user_sid.clone(), Box::new(|_| String::new())).is_err());

        Box::new(hook_event_server).stop();
        assert_eq!(forward_hook_event_through_pipe(&test_pipe_name, &current_user_sid, b"{}"), None);
    }
}
