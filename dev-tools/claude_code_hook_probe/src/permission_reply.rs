// What the listener sends back to Claude Code. Only PermissionRequest gets a real answer;
// every other event gets an empty one, which Claude Code reads as "nothing to decide".

use std::thread;
use std::time::Duration;

#[derive(Clone, Copy)]
pub enum PermissionAnswer {
    /// Empty reply: Claude Code falls back to its own permission dialog.
    NoDecision,
    Allow,
    Deny,
}

impl PermissionAnswer {
    pub fn from_label(answer_label: &str) -> Option<Self> {
        match answer_label {
            "none" => Some(Self::NoDecision),
            "allow" => Some(Self::Allow),
            "deny" => Some(Self::Deny),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::NoDecision => "none",
            Self::Allow => "allow",
            Self::Deny => "deny",
        }
    }
}

/// Waits `answer_delay` before answering a PermissionRequest, standing in for a person who
/// reads the alert first; meanwhile you can watch whether Claude Code shows its own dialog.
pub fn build_reply_to_hook_event(hook_event_name: &str, permission_answer: PermissionAnswer, answer_delay: Duration) -> String {
    if hook_event_name != "PermissionRequest" {
        return String::new();
    }
    thread::sleep(answer_delay);
    match permission_answer {
        PermissionAnswer::NoDecision => String::new(),
        PermissionAnswer::Allow => {
            r#"{"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"allow"}}}"#.to_string()
        }
        PermissionAnswer::Deny => r#"{"hookSpecificOutput":{"hookEventName":"PermissionRequest","decision":{"behavior":"deny","message":"Denied by the Crest hook probe"}}}"#.to_string(),
    }
}
