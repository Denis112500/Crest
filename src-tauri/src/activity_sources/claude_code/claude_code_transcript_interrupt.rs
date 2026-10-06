//! Recognizes the transcript line Claude Code writes when the user interrupts a turn. Claude Code
//! sends no hook event for an interrupt (measured 2026-10-07 with Claude Code 2.1.289: no Stop,
//! no StopFailure, no PostToolUseFailure, no idle notification), so its session transcript is the
//! only place it shows up. The transcript is Claude Code's internal format: if it ever changes,
//! nothing matches and Crest falls back to forgetting the session after a silence.

use serde_json::Value;

/// Seen as `[Request interrupted by user]`; matched by its start so variants still count.
const INTERRUPT_MARKER_START: &str = "[Request interrupted by user";

/// True for a user entry whose text starts with the interrupt marker, e.g.
/// `{"type":"user","message":{"content":[{"type":"text","text":"[Request interrupted by user]"}]}}`.
pub fn is_interrupt_transcript_line(transcript_line: &str) -> bool {
    let Ok(transcript_entry) = serde_json::from_str::<Value>(transcript_line) else {
        return false;
    };
    if transcript_entry.get("type").and_then(Value::as_str) != Some("user") {
        return false;
    }
    match transcript_entry.pointer("/message/content") {
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
        assert!(is_interrupt_transcript_line(interrupt_line));
        let string_content_line = r#"{"type":"user","message":{"content":"[Request interrupted by user for tool use]"}}"#;
        assert!(is_interrupt_transcript_line(string_content_line));
    }

    #[test]
    fn other_entries_and_broken_lines_are_not_interrupts() {
        assert!(!is_interrupt_transcript_line(r#"{"type":"user","message":{"content":"run ping"}}"#));
        let assistant_quoting_the_marker =
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"[Request interrupted by user]"}]}}"#;
        assert!(!is_interrupt_transcript_line(assistant_quoting_the_marker));
        assert!(!is_interrupt_transcript_line("{ not json"));
    }
}
