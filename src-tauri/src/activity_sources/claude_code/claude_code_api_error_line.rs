//! The line under a session's row when its turn stopped on an API error other than a usage
//! limit ("Waiting for you"), so the user knows why it stopped. `StopFailure` carries the text
//! Claude Code showed in `last_assistant_message` and the error's type in `error` (verified from
//! docs); a local transcript had "API Error: Connection lost mid-response. The response above may
//! be incomplete." (seen 2026-10-26).

use crate::activity_sources::claude_code::visible_text_shaping::{cut_at_word_with_ellipsis, make_hidden_characters_visible};
use crate::backend_constants::CLAUDE_CODE_TURN_SUMMARY_MAX_CHARACTERS;

/// When Claude Code sent no text, the type says at least which kind of error it was.
const API_ERROR_TYPE_PREFIX: &str = "API error: ";

/// The shown text's first line, else "API error: server error" from the type; `None` with neither.
pub fn describe_api_error(shown_error_text: Option<&str>, error_type: Option<&str>) -> Option<String> {
    let shown_first_line = shown_error_text.and_then(|error_text| error_text.lines().map(str::trim).find(|error_line| !error_line.is_empty()));
    let error_line = match (shown_first_line, error_type) {
        (Some(shown_first_line), _) => shown_first_line.to_string(),
        (None, Some(error_type)) if !error_type.trim().is_empty() => format!("{API_ERROR_TYPE_PREFIX}{}", error_type.replace('_', " ")),
        _ => return None,
    };
    Some(cut_at_word_with_ellipsis(&make_hidden_characters_visible(&error_line), CLAUDE_CODE_TURN_SUMMARY_MAX_CHARACTERS))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_shown_text_comes_first_and_the_type_stands_in_for_it() {
        let connection_lost = "API Error: Connection lost mid-response. The response above may be incomplete.";
        assert_eq!(describe_api_error(Some(connection_lost), Some("server_error")).as_deref(), Some(connection_lost));
        assert_eq!(describe_api_error(Some("\n API Error: 500\u{202E}\nmore"), None).as_deref(), Some("API Error: 500\u{FFFD}"));
        assert_eq!(describe_api_error(Some("  "), Some("server_error")).as_deref(), Some("API error: server error"));
        assert_eq!(describe_api_error(None, Some(" ")), None);
        assert_eq!(describe_api_error(None, None), None);
    }
}
