//! A markdown line's text without its inline markup, for a one-line summary: link targets,
//! HTML tags and emphasis marks go, while link text and inline code stay.

/// Doubled marks are always emphasis; a single `_` is kept, since names like `snake_case` often
/// appear outside backticks.
const DOUBLED_EMPHASIS_MARKS: [&str; 3] = ["**", "__", "~~"];

/// Inline code (between backticks) is kept as written; the rest loses its link targets, HTML
/// tags and emphasis marks. Whitespace becomes single spaces.
pub fn remove_inline_markup(line_text: &str) -> String {
    let plain_line: String = line_text
        .split('`')
        .enumerate()
        .map(|(part_index, line_part)| {
            if part_index % 2 == 1 {
                line_part.to_string()
            } else {
                remove_emphasis_marks(&replace_links_and_tags(line_part))
            }
        })
        .collect();
    plain_line.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// `[text](target)` and `![text](target)` become their text, `<https://…>` its address, and
/// any other `<tag>` is dropped.
fn replace_links_and_tags(line_part: &str) -> String {
    let mut replaced_part = String::with_capacity(line_part.len());
    let mut remaining_part = line_part;
    while let Some(next_character) = remaining_part.chars().next() {
        let after_image_mark = remaining_part.strip_prefix('!').unwrap_or(remaining_part);
        if let Some((link_text, after_link)) = split_markdown_link(after_image_mark) {
            replaced_part.push_str(link_text);
            remaining_part = after_link;
            continue;
        }
        if let Some((tag_content, after_tag)) = split_angle_bracket_tag(remaining_part) {
            if tag_content.starts_with("http://") || tag_content.starts_with("https://") {
                replaced_part.push_str(tag_content);
            }
            remaining_part = after_tag;
            continue;
        }
        replaced_part.push(next_character);
        remaining_part = &remaining_part[next_character.len_utf8()..];
    }
    replaced_part
}

/// `[text](target)…` → (`text`, `…`).
fn split_markdown_link(link_start: &str) -> Option<(&str, &str)> {
    let (link_text, after_text) = link_start.strip_prefix('[')?.split_once("](")?;
    let (_, after_link) = after_text.split_once(')')?;
    (!link_text.contains('[')).then_some((link_text, after_link))
}

/// `<content>…` → (`content`, `…`), only when it looks like a tag or an address (a letter or
/// `/` right after `<`), so "a < b" stays.
fn split_angle_bracket_tag(tag_start: &str) -> Option<(&str, &str)> {
    let after_bracket = tag_start.strip_prefix('<')?;
    let first_character = after_bracket.chars().next()?;
    if !(first_character.is_ascii_alphabetic() || first_character == '/') {
        return None;
    }
    after_bracket.split_once('>')
}

/// A single `*` goes when it touches a letter or digit on one side only (`*word*:`); "2 * 3"
/// keeps it.
fn remove_emphasis_marks(line_part: &str) -> String {
    let without_doubled_marks =
        DOUBLED_EMPHASIS_MARKS.into_iter().fold(line_part.to_string(), |part_text, doubled_mark| part_text.replace(doubled_mark, ""));
    let part_characters: Vec<char> = without_doubled_marks.chars().collect();
    part_characters
        .iter()
        .enumerate()
        .filter(|&(character_index, &part_character)| {
            if part_character != '*' {
                return true;
            }
            let is_word_before = character_index > 0 && part_characters[character_index - 1].is_alphanumeric();
            let is_word_after = part_characters.get(character_index + 1).is_some_and(|after| after.is_alphanumeric());
            is_word_before == is_word_after
        })
        .map(|(_, &part_character)| part_character)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emphasis_links_and_tags_lose_their_markup_and_code_keeps_its_text() {
        let markdown_text = "**R.1** is *built*: `a**b` ~~old~~ [docs](https://x.y/z) ![logo](a.png) <br> <https://a.b>";
        assert_eq!(remove_inline_markup(markdown_text), "R.1 is built: a**b old docs logo https://a.b");
    }

    #[test]
    fn plain_symbols_that_only_look_like_markup_stay() {
        assert_eq!(remove_inline_markup("2 * 3 < 4 in snake_case and [not a link]"), "2 * 3 < 4 in snake_case and [not a link]");
    }
}
