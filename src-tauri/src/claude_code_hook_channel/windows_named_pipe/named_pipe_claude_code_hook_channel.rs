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
    use std::sync::mpsc;
    use std::thread;

    use super::*;
    use crate::claude_code_hook_channel::PendingHookReply;

    /// Built from the real prefix, so a malformed prefix fails here instead of in the app; the
    /// test's name keeps tests running in parallel apart.
    fn test_pipe_name(test_name: &str) -> String {
        pipe_name_for_user(&format!("test-{test_name}-{}", std::process::id()))
    }

    #[test]
    fn an_event_crosses_a_real_pipe_and_the_reply_comes_back() {
        let current_user_sid = read_current_user_sid().unwrap();
        let test_pipe_name = test_pipe_name("round-trip");
        let hook_event_server = NamedPipeHookEventServer::start(
            test_pipe_name.clone(),
            current_user_sid.clone(),
            Box::new(|hook_event_json, hook_reply| hook_reply.send_reply(&format!("seen: {hook_event_json}"))),
        )
        .unwrap();
        let crest_reply = forward_hook_event_through_pipe(&test_pipe_name, &current_user_sid, br#"{"hook_event_name":"Stop"}"#);
        assert_eq!(crest_reply.as_deref(), Some(br#"seen: {"hook_event_name":"Stop"}"#.as_slice()));

        // A second server on the same name is refused: the name can't be shared or taken over.
        assert!(NamedPipeHookEventServer::start(test_pipe_name.clone(), current_user_sid.clone(), Box::new(|_, _| {})).is_err());

        Box::new(hook_event_server).stop();
        assert_eq!(forward_hook_event_through_pipe(&test_pipe_name, &current_user_sid, b"{}"), None);
    }

    #[test]
    fn a_reply_answered_later_lets_other_hooks_through_meanwhile_and_a_dropped_one_says_nothing() {
        let current_user_sid = read_current_user_sid().unwrap();
        let test_pipe_name = test_pipe_name("later");
        let (waiting_reply_sender, waiting_reply_receiver) = mpsc::channel::<Box<dyn PendingHookReply>>();
        let hook_event_server = NamedPipeHookEventServer::start(
            test_pipe_name.clone(),
            current_user_sid.clone(),
            Box::new(move |hook_event_json, hook_reply| match hook_event_json {
                "wait" => waiting_reply_sender.send(hook_reply).unwrap(),
                "drop" => drop(hook_reply),
                _ => hook_reply.send_reply("at once"),
            }),
        )
        .unwrap();

        let waiting_hook = {
            let (test_pipe_name, current_user_sid) = (test_pipe_name.clone(), current_user_sid.clone());
            thread::spawn(move || forward_hook_event_through_pipe(&test_pipe_name, &current_user_sid, b"wait"))
        };
        let waiting_reply = waiting_reply_receiver.recv().unwrap();
        let other_reply = forward_hook_event_through_pipe(&test_pipe_name, &current_user_sid, b"status");
        assert_eq!(other_reply.as_deref(), Some(b"at once".as_slice()));
        waiting_reply.send_reply("allow");
        assert_eq!(waiting_hook.join().unwrap().as_deref(), Some(b"allow".as_slice()));

        let dropped_reply = forward_hook_event_through_pipe(&test_pipe_name, &current_user_sid, b"drop");
        assert_eq!(dropped_reply.as_deref(), Some(b"".as_slice()));
        Box::new(hook_event_server).stop();
    }
}
