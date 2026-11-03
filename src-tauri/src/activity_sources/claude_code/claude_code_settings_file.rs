//! Claude Code's user settings file (`~/.claude/settings.json`, or the folder in
//! `CLAUDE_CONFIG_DIR`): where it is, reading it, and replacing it with a copy kept first. A
//! file Crest can't read is never overwritten.

use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::backend_constants::{
    CLAUDE_CODE_SETTINGS_BACKUP_EXTENSION, CLAUDE_CODE_SETTINGS_FILE_NAME, CLAUDE_CODE_SETTINGS_FOLDER_NAME,
};
use crate::text_file_atomic_replacement::replace_text_file_atomically;

/// Claude Code's own override for where its settings live.
const CLAUDE_CODE_CONFIG_FOLDER_VARIABLE: &str = "CLAUDE_CONFIG_DIR";
const USER_PROFILE_FOLDER_VARIABLE: &str = "USERPROFILE";
/// The fingerprint of a settings file that doesn't exist yet.
const MISSING_SETTINGS_FILE_FINGERPRINT: &str = "no-file";

/// The file as read, and as parsed.
pub struct ClaudeCodeSettingsSnapshot {
    pub original_settings_text: Option<String>,
    pub claude_code_settings: Value,
}

impl ClaudeCodeSettingsSnapshot {
    /// Identifies the text that was read, so a write can tell whether Claude Code rewrote its file
    /// since the user's preview (it does, seen 2026-10-07); the text itself never leaves Rust.
    /// Compared only within one run of Crest, so the standard hasher's fixed keys are enough.
    pub fn fingerprint(&self) -> String {
        let Some(settings_text) = &self.original_settings_text else {
            return MISSING_SETTINGS_FILE_FINGERPRINT.to_string();
        };
        let mut settings_text_hasher = DefaultHasher::new();
        settings_text.hash(&mut settings_text_hasher);
        format!("{:016x}", settings_text_hasher.finish())
    }
}

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

/// A missing file counts as empty settings; a file that isn't valid JSON is left alone with an
/// error, so a hand-edited file with a typo is never replaced.
pub fn read_claude_code_settings(settings_file_path: &Path) -> Result<ClaudeCodeSettingsSnapshot, String> {
    let original_settings_text = match fs::read_to_string(settings_file_path) {
        Ok(settings_text) => Some(settings_text),
        Err(read_error) if read_error.kind() == std::io::ErrorKind::NotFound => None,
        Err(read_error) => return Err(format!("could not read {}: {read_error}", settings_file_path.display())),
    };
    let claude_code_settings = match &original_settings_text {
        Some(settings_text) => serde_json::from_str(settings_text).map_err(|parse_error| {
            format!("{} isn't valid JSON, so Crest left it unchanged: {parse_error}", settings_file_path.display())
        })?,
        None => Value::Object(Default::default()),
    };
    Ok(ClaudeCodeSettingsSnapshot { original_settings_text, claude_code_settings })
}

/// The copy holds the text as it was read, so it's the user's file, not Crest's re-formatting.
pub fn replace_claude_code_settings(
    settings_file_path: &Path,
    original_settings_text: Option<&str>,
    new_claude_code_settings: &Value,
) -> Result<(), String> {
    if let Some(settings_text) = original_settings_text {
        let backup_file_path = settings_file_path.with_extension(CLAUDE_CODE_SETTINGS_BACKUP_EXTENSION);
        fs::write(&backup_file_path, settings_text).map_err(|error| format!("could not keep a copy: {error}"))?;
    }
    let new_settings_text = serde_json::to_string_pretty(new_claude_code_settings).map_err(|error| error.to_string())?;
    replace_text_file_atomically(settings_file_path, &new_settings_text)
}
