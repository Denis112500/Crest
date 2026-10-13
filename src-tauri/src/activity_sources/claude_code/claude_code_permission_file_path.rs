//! How the alert names a file Claude wants to read or change: inside the session's project by
//! its path from there ("src\main.ts"), anywhere else by its full path, with the home folder as
//! "~" ("~\.claude\settings.json"). The name alone made a file outside the project read like one
//! in it (code review before 0.4.0). A long path is shortened in the middle, so where it starts
//! and the file's name both stay visible.

use crate::backend_constants::CLAUDE_CODE_PERMISSION_FILE_PATH_MAX_CHARACTERS;

const HOME_FOLDER_MARK: &str = "~";
const SHORTENED_PART_MARK: &str = "…";

/// The folders a requested path is shown relative to.
pub struct PermissionRequestFolders<'a> {
    /// The session's working folder (`cwd` of the hook event).
    pub project_folder: Option<&'a str>,
    pub home_folder: Option<&'a str>,
}

/// Folders are compared exactly: a difference in letter case only means the full path shows,
/// which says more, never less.
pub fn show_requested_file_path(file_path: &str, request_folders: &PermissionRequestFolders) -> String {
    let path_separator = if file_path.contains('\\') { "\\" } else { "/" };
    let file_path_parts = split_path_parts(file_path);
    let shown_path = if let Some(parts_in_project) = find_parts_inside(&file_path_parts, request_folders.project_folder) {
        parts_in_project.join(path_separator)
    } else if let Some(parts_in_home) = find_parts_inside(&file_path_parts, request_folders.home_folder) {
        [&[HOME_FOLDER_MARK][..], parts_in_home].concat().join(path_separator)
    } else {
        file_path.to_string()
    };
    shorten_in_the_middle(&shown_path, path_separator)
}

/// The parts of the file's path after `folder`, if the file is inside it.
fn find_parts_inside<'path>(
    file_path_parts: &'path [&'path str],
    folder: Option<&'path str>,
) -> Option<&'path [&'path str]> {
    let folder_parts = split_path_parts(folder?);
    let parts_after_folder = file_path_parts.strip_prefix(folder_parts.as_slice())?;
    (!folder_parts.is_empty() && !parts_after_folder.is_empty()).then_some(parts_after_folder)
}

fn split_path_parts(path_text: &str) -> Vec<&str> {
    path_text.split(['\\', '/']).filter(|path_part| !path_part.is_empty()).collect()
}

/// "C:\Users\someone\a\b\c\file.txt" → "C:\…\a\b\c\file.txt": the first part, then as much of
/// the end as fits.
fn shorten_in_the_middle(shown_path: &str, path_separator: &str) -> String {
    let max_characters = CLAUDE_CODE_PERMISSION_FILE_PATH_MAX_CHARACTERS;
    if shown_path.chars().count() <= max_characters {
        return shown_path.to_string();
    }
    let first_part = split_path_parts(shown_path).into_iter().next().unwrap_or_default();
    let path_start = format!("{first_part}{path_separator}{SHORTENED_PART_MARK}{path_separator}");
    let end_character_count = max_characters.saturating_sub(path_start.chars().count());
    let path_characters: Vec<char> = shown_path.chars().collect();
    let path_end: String = path_characters[path_characters.len().saturating_sub(end_character_count)..].iter().collect();
    format!("{path_start}{path_end}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const WINDOWS_FOLDERS: PermissionRequestFolders =
        PermissionRequestFolders { project_folder: Some(r"C:\Users\someone\project"), home_folder: Some(r"C:\Users\someone") };

    #[test]
    fn a_file_in_the_project_shows_its_path_from_there() {
        assert_eq!(show_requested_file_path(r"C:\Users\someone\project\src\main.ts", &WINDOWS_FOLDERS), r"src\main.ts");
        let linux_folders = PermissionRequestFolders { project_folder: Some("/home/someone/project/"), home_folder: None };
        assert_eq!(show_requested_file_path("/home/someone/project/notes.md", &linux_folders), "notes.md");
    }

    #[test]
    fn a_file_outside_the_project_shows_where_it_is() {
        assert_eq!(show_requested_file_path(r"C:\Users\someone\.claude\settings.json", &WINDOWS_FOLDERS), r"~\.claude\settings.json");
        assert_eq!(show_requested_file_path(r"D:\other\notes.md", &WINDOWS_FOLDERS), r"D:\other\notes.md");
        // A folder whose name only starts like the project's isn't inside it.
        assert_eq!(show_requested_file_path(r"C:\Users\someone\project-old\a.txt", &WINDOWS_FOLDERS), r"~\project-old\a.txt");
        let no_folders = PermissionRequestFolders { project_folder: None, home_folder: None };
        assert_eq!(show_requested_file_path("/etc/hosts", &no_folders), "/etc/hosts");
    }

    #[test]
    fn a_different_letter_case_shows_the_full_path() {
        assert_eq!(show_requested_file_path(r"c:\users\someone\project\a.txt", &WINDOWS_FOLDERS), r"c:\users\someone\project\a.txt");
    }

    #[test]
    fn a_long_path_keeps_its_start_and_its_file_name() {
        let long_path = r"D:\archive\2026\backups\machines\laptop\config\very-long-file-name.json";
        let shown_path = show_requested_file_path(long_path, &WINDOWS_FOLDERS);
        assert_eq!(shown_path.chars().count(), CLAUDE_CODE_PERMISSION_FILE_PATH_MAX_CHARACTERS);
        assert!(shown_path.starts_with(r"D:\…\"));
        assert!(shown_path.ends_with(r"\config\very-long-file-name.json"));
    }
}
