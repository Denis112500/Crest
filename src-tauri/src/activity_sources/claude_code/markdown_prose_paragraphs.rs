//! The prose of a markdown answer as plain paragraphs, for a one-line summary. Code blocks,
//! tables, headings, rules and HTML tags are dropped; quote and list markers, emphasis marks,
//! backticks and link targets are removed, while link text and inline code stay. Not a full
//! markdown parser: it covers what Claude's answers use (790 local turn endings, 2026-10-24).
//! The inline markup is `markdown_inline_markup.rs`'s job.

use crate::activity_sources::claude_code::markdown_inline_markup::remove_inline_markup;

const CODE_FENCES: [&str; 2] = ["```", "~~~"];
const TASK_BOXES: [&str; 3] = ["[ ] ", "[x] ", "[X] "];

/// Each paragraph on one line with single spaces; a list item is a paragraph of its own.
pub fn extract_prose_paragraphs(markdown_text: &str) -> Vec<String> {
    let mut prose_paragraphs = Vec::new();
    let mut current_paragraph = String::new();
    let mut open_code_fence: Option<&str> = None;
    for markdown_line in markdown_text.lines() {
        let trimmed_line = markdown_line.trim();
        if let Some(code_fence) = open_code_fence {
            if trimmed_line.starts_with(code_fence) {
                open_code_fence = None;
            }
            continue;
        }
        if let Some(code_fence) = CODE_FENCES.into_iter().find(|code_fence| trimmed_line.starts_with(code_fence)) {
            open_code_fence = Some(code_fence);
            finish_paragraph(&mut current_paragraph, &mut prose_paragraphs);
            continue;
        }
        if has_no_prose(trimmed_line) {
            finish_paragraph(&mut current_paragraph, &mut prose_paragraphs);
            continue;
        }
        let (line_text, is_list_item) = strip_quote_and_list_markers(trimmed_line);
        if is_list_item {
            finish_paragraph(&mut current_paragraph, &mut prose_paragraphs);
        }
        let plain_line = remove_inline_markup(line_text);
        if !plain_line.is_empty() {
            if !current_paragraph.is_empty() {
                current_paragraph.push(' ');
            }
            current_paragraph.push_str(&plain_line);
        }
    }
    finish_paragraph(&mut current_paragraph, &mut prose_paragraphs);
    prose_paragraphs
}

fn finish_paragraph(current_paragraph: &mut String, prose_paragraphs: &mut Vec<String>) {
    if !current_paragraph.is_empty() {
        prose_paragraphs.push(std::mem::take(current_paragraph));
    }
}

/// A blank line, a table row, a heading or a horizontal rule.
fn has_no_prose(trimmed_line: &str) -> bool {
    let heading_level = trimmed_line.chars().take_while(|&line_character| line_character == '#').count();
    let is_heading = (1..=6).contains(&heading_level) && trimmed_line[heading_level..].starts_with([' ', '\t']);
    let rule_characters: String = trimmed_line.chars().filter(|line_character| !line_character.is_whitespace()).collect();
    let is_rule = rule_characters.len() >= 3
        && ['-', '*', '_'].iter().any(|&rule_mark| rule_characters.chars().all(|rule_character| rule_character == rule_mark));
    trimmed_line.is_empty() || trimmed_line.starts_with('|') || is_heading || is_rule
}

/// Returns the line without its leading `>` quote marks, list marker and task box, and whether
/// it was a list item.
fn strip_quote_and_list_markers(trimmed_line: &str) -> (&str, bool) {
    let mut line_text = trimmed_line;
    while let Some(quoted_text) = line_text.strip_prefix('>') {
        line_text = quoted_text.trim_start();
    }
    let bullet_text = ["- ", "* ", "+ "].into_iter().find_map(|bullet| line_text.strip_prefix(bullet));
    let digit_count = line_text.chars().take_while(char::is_ascii_digit).count();
    let numbered_text = (digit_count > 0)
        .then(|| [". ", ") "].into_iter().find_map(|number_end| line_text[digit_count..].strip_prefix(number_end)))
        .flatten();
    let Some(item_text) = bullet_text.or(numbered_text) else {
        return (line_text, false);
    };
    let item_text = item_text.trim_start();
    (TASK_BOXES.into_iter().find_map(|task_box| item_text.strip_prefix(task_box)).unwrap_or(item_text), true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_blocks_tables_headings_and_rules_are_dropped() {
        let markdown_text = "## What changed\nFirst line\nstill first.\n\n```rust\nlet x = 1;\n```\n| a | b |\n|---|---|\n---\n~~~\nunclosed code";
        assert_eq!(extract_prose_paragraphs(markdown_text), vec!["First line still first."]);
    }

    #[test]
    fn list_items_are_paragraphs_of_their_own_without_their_markers() {
        let markdown_text = "Here's the plan:\n- one\n* two\n12. three\n3) [x] four\n> > quoted";
        assert_eq!(extract_prose_paragraphs(markdown_text), vec!["Here's the plan:", "one", "two", "three", "four quoted"]);
    }
}
