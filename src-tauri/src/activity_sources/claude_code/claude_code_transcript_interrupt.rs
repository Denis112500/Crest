//! Recognizes the transcript lines Claude Code writes when the user interrupts a turn or stops
//! its background subagents. Claude Code sends no hook event for an interrupt (measured
//! 2026-10-07 with Claude Code 2.1.289: no Stop, no StopFailure, no PostToolUseFailure, no idle
//! notification), nor `SubagentStop` for stopped background subagents (seen 2026-10-23 with
//! 2.1.308), so the session transcript is the only place either shows up. Denying a permission
//! ends the turn the same way as an interrupt, with no hook event either (seen 2026-10-10). The
//! transcript is Claude Code's internal format: if it ever changes, nothing matches and Crest
//! falls back to the silence timeout.

use serde_json::Value;

/// Seen as `[Request interrupted by user]` (Esc) and `[Request interrupted by user for tool use]`
/// (a denied permission); matched by its start so both count.
const INTERRUPT_MARKER_START: &str = "[Request interrupted by user";
/// The system entry written when the user stops the background subagents.
const BACKGROUND_SUBAGENTS_STOPPED_SUBTYPE: &str = "agents_killed";

/// What a transcript line says the user stopped.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClaudeCodeTranscriptInterrupt {
    /// The turn (Esc), or a permission denied in Claude Code.
    Turn,
    /// Every background subagent of the session.
    BackgroundSubagents,
}

pub fn find_transcript_interrupt(transcript_line: &str) -> Option<ClaudeCodeTranscriptInterrupt> {
    let transcript_entry = serde_json::from_str::<Value>(transcript_line).ok()?;
    match transcript_entry.get("type").and_then(Value::as_str)? {
        "user" if has_interrupt_marker(&transcript_entry) => Some(ClaudeCodeTranscriptInterrupt::Turn),
        "system" if transcript_entry.get("subtype").and_then(Value::as_str) == Some(BACKGROUND_SUBAGENTS_STOPPED_SUBTYPE) => {
            Some(ClaudeCodeTranscriptInterrupt::BackgroundSubagents)
        }
        _ => None,
    }
}

/// A user entry whose text starts with the interrupt marker, e.g.
/// `{"type":"user","message":{"content":[{"type":"text","text":"[Request interrupted by user]"}]}}`.
fn has_interrupt_marker(user_entry: &Value) -> bool {
    match user_entry.pointer("/message/content") {
        Some(Value::String(message_text)) => message_text.starts_with(INTERRUPT_MARKER_START),
        Some(Value::Array(content_items)) => content_items.iter().any(|content_item| {
            content_item.get("text").and_then(Value::as_str).is_some_and(|text| text.starts_with(INTERRUPT_MARKER_START))
        }),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_interrupt_entry_as_claude_code_writes_it_is_recognized() {
        let interrupt_line = r#"{"type":"user","message":{"role":"user","content":[{"type":"text","text":"[Request interrupted by user]"}]},"timestamp":"2026-10-06T21:20:46Z"}"#;
        assert_eq!(find_transcript_interrupt(interrupt_line), Some(ClaudeCodeTranscriptInterrupt::Turn));
        let string_content_line = r#"{"type":"user","message":{"content":"[Request interrupted by user for tool use]"}}"#;
        assert_eq!(find_transcript_interrupt(string_content_line), Some(ClaudeCodeTranscriptInterrupt::Turn));
    }

    #[test]
    fn stopped_background_subagents_are_recognized() {
        // As Claude Code 2.1.308 wrote it (ids shortened).
        let agents_killed_line = r#"{"parentUuid":"e64d1bb9","isSidechain":false,"type":"system","subtype":"agents_killed","timestamp":"2026-10-22T22:06:49.538Z","uuid":"38e99b7a","isMeta":false,"sessionId":"660e9d6d","version":"2.1.308"}"#;
        assert_eq!(find_transcript_interrupt(agents_killed_line), Some(ClaudeCodeTranscriptInterrupt::BackgroundSubagents));
        assert_eq!(find_transcript_interrupt(r#"{"type":"system","subtype":"turn_duration"}"#), None);
    }

    #[test]
    fn other_entries_and_broken_lines_are_not_interrupts() {
        assert_eq!(find_transcript_interrupt(r#"{"type":"user","message":{"content":"run ping"}}"#), None);
        let assistant_quoting_the_marker =
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"[Request interrupted by user]"}]}}"#;
        assert_eq!(find_transcript_interrupt(assistant_quoting_the_marker), None);
        assert_eq!(find_transcript_interrupt("{ not json"), None);
    }
}
