//! Reads and writes Claude Code's user settings file (`~/.claude/settings.json`, or the folder
//! in `CLAUDE_CONFIG_DIR`), and previews a change without writing. A copy is kept before every
//! change, and a file Crest can't read is never overwritten.

use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};

use serde::Serialize;
use serde_json::Value;

use crate::activity_sources::claude_code::claude_code_hook_registration::{
    add_crest_hooks, build_crest_hook_command, remove_crest_hooks,
};
use crate::activity_sources::claude_code::claude_code_hooks_change_preview::{
    preview_claude_code_hooks_change, ClaudeCodeHooksChangePreview,
};
use crate::backend_constants::{
    CLAUDE_CODE_SETTINGS_BACKUP_EXTENSION, CLAUDE_CODE_SETTINGS_FILE_NAME, CLAUDE_CODE_SETTINGS_FOLDER_NAME,
};
use crate::text_file_atomic_replacement::replace_text_file_atomically;

/// Claude Code's own override for where its settings live.
const CLAUDE_CODE_CONFIG_FOLDER_VARIABLE: &str = "CLAUDE_CONFIG_DIR";
const USER_PROFILE_FOLDER_VARIABLE: &str = "USERPROFILE";
/// The fingerprint of a settings file that doesn't exist yet.
const MISSING_SETTINGS_FILE_FINGERPRINT: &str = "no-file";

/// What a switch in the settings window would do to Claude Code's settings file.
pub enum ClaudeCodeSettingsChange {
    AddCrestHooks { crest_executable_path: PathBuf, should_add_permission_hook: bool },
    RemoveCrestHooks,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeSettingsChangePreview {
    #[serde(flatten)]
    pub hooks_change: ClaudeCodeHooksChangePreview,
    /// Identifies the file text the preview was made from, so a write can tell whether Claude
    /// Code rewrote its file meanwhile (it does, seen 2026-10-07). The text itself never leaves
    /// Rust.
    pub settings_file_fingerprint: String,
}

/// The file as read, and as parsed.
struct ClaudeCodeSettingsSnapshot {
    original_settings_text: Option<String>,
    claude_code_settings: Value,
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

pub fn add_crest_hooks_to_claude_code_settings(
    crest_executable_path: &Path,
    should_add_permission_hook: bool,
) -> Result<(), String> {
    change_claude_code_settings(&ClaudeCodeSettingsChange::AddCrestHooks {
        crest_executable_path: crest_executable_path.to_path_buf(),
        should_add_permission_hook,
    })
}

pub fn remove_crest_hooks_from_claude_code_settings() -> Result<(), String> {
    change_claude_code_settings(&ClaudeCodeSettingsChange::RemoveCrestHooks)
}

/// Reads the file without writing anything. `None` previews leaving the file as it is (the
/// answering switch while the integration is off).
pub fn preview_claude_code_settings_change(
    settings_change: Option<&ClaudeCodeSettingsChange>,
) -> Result<ClaudeCodeSettingsChangePreview, String> {
    let settings_snapshot = read_claude_code_settings(&claude_code_settings_file_path()?)?;
    let mut settings_after_change = settings_snapshot.claude_code_settings.clone();
    if let Some(settings_change) = settings_change {
        apply_claude_code_settings_change(settings_change, &mut settings_after_change)?;
    }
    Ok(ClaudeCodeSettingsChangePreview {
        hooks_change: preview_claude_code_hooks_change(&settings_snapshot.claude_code_settings, &settings_after_change),
        settings_file_fingerprint: fingerprint_settings_text(settings_snapshot.original_settings_text.as_deref()),
    })
}

/// Nothing is written if nothing changed.
fn change_claude_code_settings(settings_change: &ClaudeCodeSettingsChange) -> Result<(), String> {
    let settings_file_path = claude_code_settings_file_path()?;
    let ClaudeCodeSettingsSnapshot { original_settings_text, mut claude_code_settings } =
        read_claude_code_settings(&settings_file_path)?;
    let settings_before_change = claude_code_settings.clone();
    apply_claude_code_settings_change(settings_change, &mut claude_code_settings)?;
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

/// A missing file counts as empty settings; a file that isn't valid JSON is left alone with an
/// error, so a hand-edited file with a typo is never replaced.
fn read_claude_code_settings(settings_file_path: &Path) -> Result<ClaudeCodeSettingsSnapshot, String> {
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

fn apply_claude_code_settings_change(
    settings_change: &ClaudeCodeSettingsChange,
    claude_code_settings: &mut Value,
) -> Result<(), String> {
    match settings_change {
        ClaudeCodeSettingsChange::AddCrestHooks { crest_executable_path, should_add_permission_hook } => {
            let crest_hook_command = build_crest_hook_command(crest_executable_path);
            add_crest_hooks(claude_code_settings, &crest_hook_command, *should_add_permission_hook)
        }
        ClaudeCodeSettingsChange::RemoveCrestHooks => {
            remove_crest_hooks(claude_code_settings);
            Ok(())
        }
    }
}

/// Compared only within one run of Crest (preview, then write), so the standard hasher's
/// fixed keys are enough; it isn't meant to be stored.
fn fingerprint_settings_text(settings_text: Option<&str>) -> String {
    let Some(settings_text) = settings_text else {
        return MISSING_SETTINGS_FILE_FINGERPRINT.to_string();
    };
    let mut settings_text_hasher = DefaultHasher::new();
    settings_text.hash(&mut settings_text_hasher);
    format!("{:016x}", settings_text_hasher.finish())
}
