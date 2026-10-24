//! The short line a finished session shows under its row, from the final answer Claude Code
//! sends with `Stop` (`last_assistant_message`: the raw markdown of the turn's last message,
//! verified 2026-10-24). Claude's text is shown, so nothing in it may hide or reorder what it says.
//!
//! Which sentence (decided 2026-10-24 from 790 local turn endings): the answer's first sentence,
//! which usually says what happened ("R.1 is built: fixes 1–4."), while the last one is usually
//! a next step; but when the answer ends with a question, that question, since Claude waits for
//! the user's answer.

use crate::activity_sources::claude_code::markdown_prose_paragraphs::extract_prose_paragraphs;
use crate::activity_sources::claude_code::visible_text_shaping::{cut_at_word_with_ellipsis, make_hidden_characters_visible};
use crate::backend_constants::CLAUDE_CODE_TURN_SUMMARY_MAX_CHARACTERS;

/// `None` for an answer with no prose (empty, or only code or a table).
pub fn summarize_final_answer(final_answer: &str) -> Option<String> {
    let prose_paragraphs = extract_prose_paragraphs(&make_hidden_characters_visible(final_answer));
    let closing_sentence = *split_sentences(prose_paragraphs.last()?).last()?;
    let chosen_sentence =
        if is_question(closing_sentence) { closing_sentence.to_string() } else { find_opening_sentence(&prose_paragraphs) };
    Some(cut_at_word_with_ellipsis(&chosen_sentence, CLAUDE_CODE_TURN_SUMMARY_MAX_CHARACTERS))
}

/// A lead-in alone in its paragraph ("About the GIF:", 3 of 40 sampled answers) says little, so
/// the next paragraph's first sentence follows it.
fn find_opening_sentence(prose_paragraphs: &[String]) -> String {
    let first_paragraph_sentences = split_sentences(&prose_paragraphs[0]);
    let first_sentence = first_paragraph_sentences[0];
    match prose_paragraphs.get(1) {
        Some(next_paragraph) if first_paragraph_sentences.len() == 1 && first_sentence.ends_with(':') => {
            format!("{first_sentence} {}", split_sentences(next_paragraph)[0])
        }
        _ => first_sentence.to_string(),
    }
}

/// A sentence ends at ".", "!" or "?" followed by a space and a capital, a digit or an opening
/// quote, so "v0.5.0", "main.rs" and "e.g. this" don't end one. The paragraph's spaces are
/// single ASCII spaces (extract_prose_paragraphs).
fn split_sentences(prose_paragraph: &str) -> Vec<&str> {
    let paragraph_characters: Vec<(usize, char)> = prose_paragraph.char_indices().collect();
    let mut sentences = Vec::new();
    let mut sentence_start = 0;
    for character_window in paragraph_characters.windows(3) {
        let [(_, ending_character), (space_index, ' '), (_, next_start_character)] = character_window else {
            continue;
        };
        if matches!(ending_character, '.' | '!' | '?') && starts_a_sentence(*next_start_character) {
            sentences.push(&prose_paragraph[sentence_start..*space_index]);
            sentence_start = space_index + 1;
        }
    }
    sentences.push(&prose_paragraph[sentence_start..]);
    sentences
}

fn starts_a_sentence(start_character: char) -> bool {
    start_character.is_uppercase() || start_character.is_ascii_digit() || matches!(start_character, '"' | '“' | '(' | '\'')
}

fn is_question(sentence: &str) -> bool {
    sentence.trim_end_matches(['"', '”', ')', '\'']).ends_with('?')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activity_sources::claude_code::visible_text_shaping::ELLIPSIS;

    fn summary(final_answer: &str) -> Option<String> {
        summarize_final_answer(final_answer)
    }

    #[test]
    fn the_first_sentence_says_what_happened() {
        let final_answer = "## R.1\n\n**R.1 is built:** fixes 1–4 in `main.rs`. Tests pass.\n\n- one\n\nSay \"commit\".";
        assert_eq!(summary(final_answer).as_deref(), Some("R.1 is built: fixes 1–4 in main.rs."));
        assert_eq!(summary("Crest v0.5.0 is out, e.g. on GitHub. Next: posts.").as_deref(), Some("Crest v0.5.0 is out, e.g. on GitHub."));
    }

    #[test]
    fn a_lead_in_is_followed_by_what_it_introduces() {
        assert_eq!(summary("About the GIF:\n\n- It loads now. It's smaller.").as_deref(), Some("About the GIF: It loads now."));
        assert_eq!(summary("Two things changed:").as_deref(), Some("Two things changed:"));
    }

    #[test]
    fn a_closing_question_wins_over_the_first_sentence() {
        let final_answer = "The plan is ready.\n\n```text\nstep 1\n```\n\n**Shall I install it?** Then I'll build it.\n\nWhich branch?";
        assert_eq!(summary(final_answer).as_deref(), Some("Which branch?"));
        assert_eq!(summary("Is it fixed? Yes, it is.").as_deref(), Some("Is it fixed?"));
    }

    #[test]
    fn an_answer_without_prose_has_no_summary() {
        assert_eq!(summary(" \n\t\n"), None);
        assert_eq!(summary("```\nhi\n```\n| a |\n|---|"), None);
    }

    #[test]
    fn hidden_characters_show_and_a_long_sentence_is_cut_at_a_word() {
        assert_eq!(summary("Done\u{202E}txt.exe").as_deref(), Some("Done\u{FFFD}txt.exe"));
        let long_summary = summary(&"word, ".repeat(CLAUDE_CODE_TURN_SUMMARY_MAX_CHARACTERS)).unwrap();
        assert!(long_summary.chars().count() <= CLAUDE_CODE_TURN_SUMMARY_MAX_CHARACTERS);
        assert!(long_summary.ends_with(&format!("word{ELLIPSIS}")));
    }
}
