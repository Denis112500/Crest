//! The last part of a path as Claude Code sends it (a project folder, a file): Windows and
//! Unix separators both count, and a trailing separator is ignored.

pub fn find_last_path_part(path_text: &str) -> Option<&str> {
    path_text.rsplit(['\\', '/']).find(|path_part| !path_part.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn either_separator_and_a_trailing_one_are_handled() {
        assert_eq!(find_last_path_part(r"C:\Projects\crest"), Some("crest"));
        assert_eq!(find_last_path_part("/home/someone/src/lib.rs/"), Some("lib.rs"));
        assert_eq!(find_last_path_part(r"\\"), None);
    }
}
