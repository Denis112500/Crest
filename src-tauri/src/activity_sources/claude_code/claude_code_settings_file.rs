//! Reads and writes Claude Code's user settings file (`~/.claude/settings.json`, or the folder
//! in `CLAUDE_CONFIG_DIR`). A copy is kept before every change, and a file Crest can't read is
//! never overwritten.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::activity_sources::claude_code::claude_code_hook_registration::{
    add_crest_hooks, build_crest_hook_command, remove_crest_hooks,
};
use crate::backend_constants::{
    CLAUDE_CODE_SETTINGS_BACKUP_EXTENSION, CLAUDE_CODE_SETTINGS_FILE_NAME, CLAUDE_CODE_SETTINGS_FOLDER_NAME,
};
use crate::text_file_atomic_replacement::replace_text_file_atomically;

/// Claude Code's own override for where its settings live.
const CLAUDE_CODE_CONFIG_FOLDER_VARIABLE: &str = "CLAUDE_CONFIG_DIR";
const USER_PROFILE_FOLDER_VARIABLE: &str = "USERPROFILE";

pub fn claude_code_settings_file_path() -> Result<PathBuf, String> {
    let settings_folder = match std::env::var_os(CLAUDE_CODE_CONFIG_FOLDER_VARIABLE) {
        Some(configured_folder) => PathBuf::from(configured_folder),
        None => PathBuf::from(
            std::env::var_os(USER_PROFILE_FOLDER_VARIABLE).ok_or("the user profile folder is unknown")?,
        )
        .join(CLAUDE_CODE_SETTINGS_FOLDER_NAME),
    };
    Ok(settings_folder.join(CLAUDE_CODE_SETTINGS_FILE_NAME))
}

pub fn add_crest_hooks_to_claude_code_settings(crest_executable_path: &Path) -> Result<(), String> {
    let crest_hook_command = build_crest_hook_command(crest_executable_path);
    change_claude_code_settings(|claude_code_settings| add_crest_hooks(claude_code_settings, &crest_hook_command))
}

pub fn remove_crest_hooks_from_claude_code_settings() -> Result<(), String> {
    change_claude_code_settings(|claude_code_settings| {
        remove_crest_hooks(claude_code_settings);
        Ok(())
    })
}

/// A missing file counts as empty settings; a file that isn't valid JSON is left alone with an
/// error, so a hand-edited file with a typo is never replaced. Nothing is written if nothing changed.
fn change_claude_code_settings(apply_change: impl FnOnce(&mut Value) -> Result<(), String>) -> Result<(), String> {
    let settings_file_path = claude_code_settings_file_path()?;
    let original_settings_text = match fs::read_to_string(&settings_file_path) {
        Ok(settings_text) => Some(settings_text),
        Err(read_error) if read_error.kind() == std::io::ErrorKind::NotFound => None,
        Err(read_error) => return Err(format!("could not read {}: {read_error}", settings_file_path.display())),
    };
    let mut claude_code_settings = match &original_settings_text {
        Some(settings_text) => serde_json::from_str(settings_text).map_err(|parse_error| {
            format!("{} isn't valid JSON, so Crest left it unchanged: {parse_error}", settings_file_path.display())
        })?,
        None => Value::Object(Default::default()),
    };
    let settings_before_change = claude_code_settings.clone();
    apply_change(&mut claude_code_settings)?;
    if claude_code_settings == settings_before_change {
        return Ok(());
    }
    if let Some(settings_text) = &original_settings_text {
        let backup_file_path = settings_file_path.with_extension(CLAUDE_CODE_SETTINGS_BACKUP_EXTENSION);
        fs::write(&backup_file_path, settings_text).map_err(|error| format!("could not keep a copy: {error}"))?;
    }
    let new_settings_text = serde_json::to_string_pretty(&claude_code_settings).map_err(|error| error.to_string())?;
    replace_text_file_atomically(&settings_file_path, &new_settings_text)
}
