//! Finds when a reached claude.ai usage limit resets. No hook event says it (code.claude.com/
//! docs/en/hooks); the transcript does: the error entry Claude Code writes as the turn stops
//! carries `"error":"rate_limit"` and `"quotaLimits":{"status":"rejected","resetsAt":<Unix
//! seconds>,…}` (seen 2026-10-06 with Claude Code 2.1.288). Claude Code's internal format: if
//! it changes, nothing matches and the pill only says the limit is reached.

use std::path::Path;

use serde_json::Value;

use crate::activity_sources::claude_code::claude_code_hook_event::RATE_LIMIT_STOP_FAILURE_ERROR;
use crate::activity_sources::claude_code::transcript_end_text::read_transcript_end;
use crate::backend_constants::CLAUDE_CODE_TRANSCRIPT_USAGE_LIMIT_SEARCH_BYTES;

/// The reset time (Unix seconds) of the newest usage-limit entry near the transcript's end. It
/// may be an earlier limit's if this one's entry isn't written yet: the caller drops a reset
/// that has already passed.
pub fn find_usage_limit_reset(transcript_path: &Path) -> Option<u64> {
    find_usage_limit_reset_in(&read_transcript_end(transcript_path, CLAUDE_CODE_TRANSCRIPT_USAGE_LIMIT_SEARCH_BYTES))
}

fn find_usage_limit_reset_in(transcript_end: &str) -> Option<u64> {
    transcript_end.lines().rev().find_map(read_usage_limit_reset)
}

fn read_usage_limit_reset(transcript_line: &str) -> Option<u64> {
    let transcript_entry = serde_json::from_str::<Value>(transcript_line).ok()?;
    if transcript_entry.get("error")?.as_str()? != RATE_LIMIT_STOP_FAILURE_ERROR {
        return None;
    }
    transcript_entry.get("quotaLimits")?.get("resetsAt")?.as_u64()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The entry as Claude Code 2.1.288 wrote it, without the message's usage numbers.
    const USAGE_LIMIT_LINE: &str = r#"{"type":"assistant","timestamp":"2026-10-06T22:12:11.599Z","message":{"model":"<synthetic>","content":[{"type":"text","text":"You've hit your session limit · resets 1:50am (Europe/Bucharest)"}]},"quotaLimits":{"status":"rejected","resetsAt":1791327000,"rateLimitType":"five_hour","isUsingOverage":false},"error":"rate_limit","isApiErrorMessage":true,"apiErrorStatus":429}"#;

    #[test]
    fn the_newest_usage_limit_entry_gives_the_reset_time() {
        let older_limit = USAGE_LIMIT_LINE.replace("1791327000", "1791300000");
        let transcript_end = format!("cut line\"}}\n{older_limit}\n{USAGE_LIMIT_LINE}\n{{\"type\":\"system\"}}\n");
        assert_eq!(find_usage_limit_reset_in(&transcript_end), Some(1_791_327_000));
    }

    #[test]
    fn other_errors_and_limits_without_a_reset_give_nothing() {
        assert_eq!(read_usage_limit_reset(&USAGE_LIMIT_LINE.replace("\"rate_limit\"", "\"overloaded\"")), None);
        assert_eq!(read_usage_limit_reset(r#"{"type":"assistant","error":"rate_limit","apiErrorStatus":429}"#), None);
        assert_eq!(find_usage_limit_reset_in("{ not json\n"), None);
    }
}
