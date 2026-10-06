//! Replaces a text file in one step: write a temporary file next to it, then rename it over the
//! real one. A crash mid-write leaves the old file intact instead of a half-written one. Used for
//! Crest's own settings and for Claude Code's settings file.

use std::fs;
use std::path::Path;

pub fn replace_text_file_atomically(file_path: &Path, file_text: &str) -> Result<(), String> {
    if let Some(parent_folder) = file_path.parent() {
        fs::create_dir_all(parent_folder).map_err(|error| error.to_string())?;
    }
    let mut temporary_file_name = file_path.file_name().unwrap_or_default().to_os_string();
    temporary_file_name.push(".tmp");
    let temporary_file_path = file_path.with_file_name(temporary_file_name);
    fs::write(&temporary_file_path, file_text).map_err(|error| error.to_string())?;
    fs::rename(&temporary_file_path, file_path).map_err(|error| error.to_string())
}
