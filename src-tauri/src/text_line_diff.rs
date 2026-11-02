//! Compares two texts line by line, the way `diff -u` shows it: lines only in the old text are
//! removed, lines only in the new one added, and long runs of unchanged lines are cut down to a
//! few lines of context around each change. Pure; the settings window shows the result.

use serde::Serialize;

use crate::backend_constants::TEXT_LINE_DIFF_MAX_COMPARED_LINE_PAIRS;

#[derive(Debug, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum TextLineDiffEntry {
    Unchanged { line_text: String },
    Added { line_text: String },
    Removed { line_text: String },
    /// Unchanged lines left out between two changes ("… N more").
    SkippedUnchanged { skipped_line_count: usize },
}

/// Empty when the texts are the same.
pub fn diff_text_lines(old_text: &str, new_text: &str, context_line_count: usize) -> Vec<TextLineDiffEntry> {
    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();
    if old_lines == new_lines {
        return Vec::new();
    }
    cut_unchanged_runs(align_lines(&old_lines, &new_lines), context_line_count)
}

/// The longest run of lines both texts share, in order, stays unchanged; everything else is
/// removed or added. Past the size limit the whole old text is shown removed and the new one
/// added: still true, only longer.
fn align_lines(old_lines: &[&str], new_lines: &[&str]) -> Vec<TextLineDiffEntry> {
    let removed = |line_text: &&str| TextLineDiffEntry::Removed { line_text: line_text.to_string() };
    let added = |line_text: &&str| TextLineDiffEntry::Added { line_text: line_text.to_string() };
    if old_lines.len().saturating_mul(new_lines.len()) > TEXT_LINE_DIFF_MAX_COMPARED_LINE_PAIRS {
        return old_lines.iter().map(removed).chain(new_lines.iter().map(added)).collect();
    }
    // shared_line_count_from[old][new]: how many lines old_lines[old..] and new_lines[new..] share.
    let mut shared_line_count_from = vec![vec![0usize; new_lines.len() + 1]; old_lines.len() + 1];
    for old_index in (0..old_lines.len()).rev() {
        for new_index in (0..new_lines.len()).rev() {
            shared_line_count_from[old_index][new_index] = if old_lines[old_index] == new_lines[new_index] {
                shared_line_count_from[old_index + 1][new_index + 1] + 1
            } else {
                shared_line_count_from[old_index + 1][new_index].max(shared_line_count_from[old_index][new_index + 1])
            };
        }
    }
    let mut aligned_lines = Vec::new();
    let (mut old_index, mut new_index) = (0, 0);
    while old_index < old_lines.len() && new_index < new_lines.len() {
        if old_lines[old_index] == new_lines[new_index] {
            aligned_lines.push(TextLineDiffEntry::Unchanged { line_text: old_lines[old_index].to_string() });
            old_index += 1;
            new_index += 1;
        } else if shared_line_count_from[old_index + 1][new_index] >= shared_line_count_from[old_index][new_index + 1] {
            aligned_lines.push(removed(&old_lines[old_index]));
            old_index += 1;
        } else {
            aligned_lines.push(added(&new_lines[new_index]));
            new_index += 1;
        }
    }
    aligned_lines.extend(old_lines[old_index..].iter().map(removed));
    aligned_lines.extend(new_lines[new_index..].iter().map(added));
    aligned_lines
}

/// Keeps `context_line_count` unchanged lines next to each change; a run at the very start or end
/// keeps only the side that touches a change.
fn cut_unchanged_runs(aligned_lines: Vec<TextLineDiffEntry>, context_line_count: usize) -> Vec<TextLineDiffEntry> {
    let mut shown_lines = Vec::new();
    let mut unchanged_run = Vec::new();
    for aligned_line in aligned_lines {
        if matches!(aligned_line, TextLineDiffEntry::Unchanged { .. }) {
            unchanged_run.push(aligned_line);
            continue;
        }
        let kept_before = if shown_lines.is_empty() { 0 } else { context_line_count };
        push_cut_run(&mut shown_lines, std::mem::take(&mut unchanged_run), kept_before, context_line_count);
        shown_lines.push(aligned_line);
    }
    push_cut_run(&mut shown_lines, unchanged_run, context_line_count, 0);
    shown_lines
}

fn push_cut_run(
    shown_lines: &mut Vec<TextLineDiffEntry>,
    mut unchanged_run: Vec<TextLineDiffEntry>,
    kept_at_start: usize,
    kept_at_end: usize,
) {
    if unchanged_run.len() <= kept_at_start + kept_at_end {
        shown_lines.append(&mut unchanged_run);
        return;
    }
    let lines_at_end = unchanged_run.split_off(unchanged_run.len() - kept_at_end);
    let skipped_line_count = unchanged_run.len() - kept_at_start;
    unchanged_run.truncate(kept_at_start);
    shown_lines.append(&mut unchanged_run);
    shown_lines.push(TextLineDiffEntry::SkippedUnchanged { skipped_line_count });
    shown_lines.extend(lines_at_end);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unchanged(line_text: &str) -> TextLineDiffEntry {
        TextLineDiffEntry::Unchanged { line_text: line_text.to_string() }
    }
    fn added(line_text: &str) -> TextLineDiffEntry {
        TextLineDiffEntry::Added { line_text: line_text.to_string() }
    }
    fn removed(line_text: &str) -> TextLineDiffEntry {
        TextLineDiffEntry::Removed { line_text: line_text.to_string() }
    }
    fn skipped(skipped_line_count: usize) -> TextLineDiffEntry {
        TextLineDiffEntry::SkippedUnchanged { skipped_line_count }
    }

    #[test]
    fn the_same_text_gives_no_lines() {
        assert!(diff_text_lines("a\nb\n", "a\nb\n", 2).is_empty());
    }

    #[test]
    fn added_and_removed_lines_sit_between_the_shared_ones() {
        assert_eq!(
            diff_text_lines("a\nb\nc", "a\nx\nc\nd", 2),
            vec![unchanged("a"), removed("b"), added("x"), unchanged("c"), added("d")]
        );
    }

    #[test]
    fn long_unchanged_runs_are_cut_to_the_context_around_each_change() {
        let old_text = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10";
        let new_text = "1\n2\n3\n4\nnew\n5\n6\n7\n8\n9\n10";
        assert_eq!(
            diff_text_lines(old_text, new_text, 1),
            vec![skipped(3), unchanged("4"), added("new"), unchanged("5"), skipped(5)]
        );
        let two_changes = "1\n2\n3\n4\nnew\n5\n6\n7\n8\n9\nalso new\n10";
        assert_eq!(
            diff_text_lines(old_text, two_changes, 1),
            vec![skipped(3), unchanged("4"), added("new"), unchanged("5"), skipped(3), unchanged("9"), added("also new"), unchanged("10")]
        );
    }

    #[test]
    fn a_short_run_between_changes_is_kept_whole() {
        assert_eq!(
            diff_text_lines("a\n1\n2\nb", "A\n1\n2\nB", 1),
            vec![removed("a"), added("A"), unchanged("1"), unchanged("2"), removed("b"), added("B")]
        );
    }
}
