//! How long the hook copy of `crest.exe` waits for Crest. Status events get a short deadline, so
//! a frozen or stuck Crest can't leave hook copies running; a permission request waits for its
//! answer, bounded by Crest's own queue and Claude Code's timeout.

use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use serde::Deserialize;

use crate::activity_sources::claude_code::PERMISSION_REQUEST_HOOK_EVENT;
use crate::backend_constants::CLAUDE_CODE_STATUS_HOOK_DEADLINE;

/// Only the name is read here; the whole event is Crest's to parse.
#[derive(Deserialize)]
struct HookEventNameField {
    hook_event_name: String,
}

/// `None` means no deadline. An event that can't be read gets the status deadline: Crest can't
/// read it either, so there is no answer worth waiting for.
pub fn hook_deadline_for_event(hook_event_json: &[u8]) -> Option<Duration> {
    match serde_json::from_slice::<HookEventNameField>(hook_event_json) {
        Ok(event_name_field) if event_name_field.hook_event_name == PERMISSION_REQUEST_HOOK_EVENT => None,
        _ => Some(CLAUDE_CODE_STATUS_HOOK_DEADLINE),
    }
}

/// With a deadline, the forwarding runs on its own thread and the caller stops waiting when time
/// is up. The thread is left behind on purpose: the hook process ends right after, and ending a
/// process ends all its threads, so a Crest that never answers can't keep it alive.
pub fn forward_within_hook_deadline(
    hook_deadline: Option<Duration>,
    forward_to_crest: impl FnOnce() -> Option<Vec<u8>> + Send + 'static,
) -> Option<Vec<u8>> {
    let Some(hook_deadline) = hook_deadline else {
        return forward_to_crest();
    };
    let (crest_reply_sender, crest_reply_receiver) = mpsc::channel();
    thread::spawn(move || {
        // The receiver is gone once the deadline passed; the late reply has nowhere to go.
        let _ = crest_reply_sender.send(forward_to_crest());
    });
    crest_reply_receiver.recv_timeout(hook_deadline).ok().flatten()
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use super::*;

    const SHORT_TEST_DEADLINE: Duration = Duration::from_millis(200);

    #[test]
    fn status_events_get_the_deadline_and_permission_requests_wait() {
        assert_eq!(hook_deadline_for_event(br#"{"hook_event_name":"Stop","session_id":"a"}"#), Some(CLAUDE_CODE_STATUS_HOOK_DEADLINE));
        assert_eq!(hook_deadline_for_event(br#"{"hook_event_name":"PreToolUse"}"#), Some(CLAUDE_CODE_STATUS_HOOK_DEADLINE));
        assert_eq!(hook_deadline_for_event(br#"{"hook_event_name":"PermissionRequest","tool_name":"Bash"}"#), None);
        assert_eq!(hook_deadline_for_event(br#"{"session_id":"a"}"#), Some(CLAUDE_CODE_STATUS_HOOK_DEADLINE));
        assert_eq!(hook_deadline_for_event(b"not json"), Some(CLAUDE_CODE_STATUS_HOOK_DEADLINE));
    }

    #[test]
    fn a_reply_in_time_comes_through() {
        let crest_reply = forward_within_hook_deadline(Some(SHORT_TEST_DEADLINE), || Some(b"reply".to_vec()));
        assert_eq!(crest_reply.as_deref(), Some(b"reply".as_slice()));
        assert_eq!(forward_within_hook_deadline(None, || Some(b"reply".to_vec())).as_deref(), Some(b"reply".as_slice()));
    }

    #[test]
    fn a_crest_that_never_answers_is_given_up_on_at_the_deadline() {
        let (_never_sent, never_answered) = mpsc::channel::<Vec<u8>>();
        let waiting_started = Instant::now();
        let crest_reply = forward_within_hook_deadline(Some(SHORT_TEST_DEADLINE), move || never_answered.recv().ok());
        assert_eq!(crest_reply, None);
        assert!(waiting_started.elapsed() < SHORT_TEST_DEADLINE * 3);
    }
}
