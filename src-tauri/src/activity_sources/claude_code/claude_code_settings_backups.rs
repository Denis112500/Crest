//! The dated copies of Claude Code's settings file in Crest's own folder: one kept right before
//! each change Crest makes, only the newest few kept, and Crest's older single copy moved in.

use std::fs::{self, OpenOptions};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::backend_constants::{
    CLAUDE_CODE_SETTINGS_BACKUP_FILE_NAME_PREFIX, CLAUDE_CODE_SETTINGS_BACKUP_FILE_NAME_SUFFIX,
    CLAUDE_CODE_SETTINGS_BACKUP_FOLDER_NAME, CLAUDE_CODE_SETTINGS_BACKUP_KEPT_COUNT, CREST_APP_IDENTIFIER,
    OLD_CLAUDE_CODE_SETTINGS_BACKUP_EXTENSION,
};
use crate::utc_file_name_timestamp::format_utc_file_name_timestamp;

const ROAMING_APP_DATA_FOLDER_VARIABLE: &str = "APPDATA";

/// Inside the folder where Tauri keeps Crest's own settings, found without Tauri so the
/// uninstaller keeps a copy too.
pub fn claude_code_settings_backup_folder() -> Result<PathBuf, String> {
    let roaming_app_data_folder =
        std::env::var_os(ROAMING_APP_DATA_FOLDER_VARIABLE).ok_or("the roaming app data folder is unknown")?;
    Ok(PathBuf::from(roaming_app_data_folder).join(CREST_APP_IDENTIFIER).join(CLAUDE_CODE_SETTINGS_BACKUP_FOLDER_NAME))
}

/// Skipped when the text equals the newest copy, so switching back and forth doesn't push the
/// user's own version out of the kept ones. A copy already made in the same second stays: it
/// holds the older state.
pub fn keep_claude_code_settings_copy(backup_folder: &Path, settings_text: &str, copy_time: SystemTime) -> Result<(), String> {
    fs::create_dir_all(backup_folder).map_err(|error| format!("could not create {}: {error}", backup_folder.display()))?;
    let newest_copy_text = list_settings_copies(backup_folder)?.last().and_then(|newest_copy| fs::read_to_string(newest_copy).ok());
    if newest_copy_text.as_deref() == Some(settings_text) {
        return Ok(());
    }
    let copy_path = backup_folder.join(name_settings_copy(copy_time));
    match OpenOptions::new().write(true).create_new(true).open(&copy_path) {
        Ok(mut copy_file) => {
            copy_file.write_all(settings_text.as_bytes()).map_err(|error| format!("could not keep a copy: {error}"))?
        }
        Err(open_error) if open_error.kind() == ErrorKind::AlreadyExists => {}
        Err(open_error) => return Err(format!("could not keep a copy: {open_error}")),
    }
    remove_older_settings_copies(backup_folder)
}

/// Named after its last change, so it sorts among the copies by its age; removed from next to
/// Claude Code's file only once it's in the backup folder.
pub fn move_in_old_crest_backup(settings_file_path: &Path, backup_folder: &Path) -> Result<(), String> {
    let old_backup_path = settings_file_path.with_extension(OLD_CLAUDE_CODE_SETTINGS_BACKUP_EXTENSION);
    let last_change_time = match fs::metadata(&old_backup_path) {
        Ok(old_backup_metadata) => old_backup_metadata.modified().unwrap_or_else(|_| SystemTime::now()),
        Err(metadata_error) if metadata_error.kind() == ErrorKind::NotFound => return Ok(()),
        Err(metadata_error) => return Err(format!("could not read {}: {metadata_error}", old_backup_path.display())),
    };
    fs::create_dir_all(backup_folder).map_err(|error| format!("could not create {}: {error}", backup_folder.display()))?;
    let moved_copy_path = backup_folder.join(name_settings_copy(last_change_time));
    if !moved_copy_path.exists() {
        fs::copy(&old_backup_path, &moved_copy_path).map_err(|error| format!("could not move the old copy: {error}"))?;
    }
    fs::remove_file(&old_backup_path).map_err(|error| format!("could not remove the old copy: {error}"))?;
    remove_older_settings_copies(backup_folder)
}

fn name_settings_copy(copy_time: SystemTime) -> String {
    format!(
        "{CLAUDE_CODE_SETTINGS_BACKUP_FILE_NAME_PREFIX}{}{CLAUDE_CODE_SETTINGS_BACKUP_FILE_NAME_SUFFIX}",
        format_utc_file_name_timestamp(copy_time)
    )
}

/// Oldest first: the names hold fixed-width UTC times.
fn list_settings_copies(backup_folder: &Path) -> Result<Vec<PathBuf>, String> {
    let folder_entries =
        fs::read_dir(backup_folder).map_err(|error| format!("could not list {}: {error}", backup_folder.display()))?;
    let mut settings_copies: Vec<PathBuf> = folder_entries
        .filter_map(Result::ok)
        .map(|folder_entry| folder_entry.path())
        .filter(|entry_path| {
            entry_path.file_name().and_then(|file_name| file_name.to_str()).is_some_and(|file_name| {
                file_name.starts_with(CLAUDE_CODE_SETTINGS_BACKUP_FILE_NAME_PREFIX)
                    && file_name.ends_with(CLAUDE_CODE_SETTINGS_BACKUP_FILE_NAME_SUFFIX)
            })
        })
        .collect();
    settings_copies.sort();
    Ok(settings_copies)
}

fn remove_older_settings_copies(backup_folder: &Path) -> Result<(), String> {
    let settings_copies = list_settings_copies(backup_folder)?;
    let removed_copy_count = settings_copies.len().saturating_sub(CLAUDE_CODE_SETTINGS_BACKUP_KEPT_COUNT);
    for removed_copy in &settings_copies[..removed_copy_count] {
        fs::remove_file(removed_copy).map_err(|error| format!("could not remove an old copy: {error}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use serde_json::Value;

    use super::*;

    fn create_empty_test_folder(test_name: &str) -> PathBuf {
        let test_folder = std::env::temp_dir().join(format!("crest-settings-backups-test-{test_name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&test_folder);
        fs::create_dir_all(&test_folder).unwrap();
        test_folder
    }

    fn moment(seconds_since_epoch: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(seconds_since_epoch)
    }

    fn copy_names(backup_folder: &Path) -> Vec<String> {
        list_settings_copies(backup_folder)
            .unwrap()
            .iter()
            .map(|copy_path| copy_path.file_name().unwrap().to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn each_change_keeps_a_dated_copy_but_not_twice_the_same_text() {
        let backup_folder = create_empty_test_folder("dated").join("backups");
        keep_claude_code_settings_copy(&backup_folder, "first", moment(1_793_697_307)).unwrap();
        keep_claude_code_settings_copy(&backup_folder, "first", moment(1_793_697_400)).unwrap();
        keep_claude_code_settings_copy(&backup_folder, "second", moment(1_793_697_500)).unwrap();
        assert_eq!(
            copy_names(&backup_folder),
            ["settings.json.2026-11-03_09-15-07-UTC.json", "settings.json.2026-11-03_09-18-20-UTC.json"]
        );
        let _ = fs::remove_dir_all(backup_folder.parent().unwrap());
    }

    #[test]
    fn a_copy_made_in_the_same_second_keeps_the_older_state() {
        let backup_folder = create_empty_test_folder("same-second");
        keep_claude_code_settings_copy(&backup_folder, "older", moment(1_793_697_307)).unwrap();
        keep_claude_code_settings_copy(&backup_folder, "newer", moment(1_793_697_307)).unwrap();
        let kept_copies = list_settings_copies(&backup_folder).unwrap();
        assert_eq!(kept_copies.len(), 1);
        assert_eq!(fs::read_to_string(&kept_copies[0]).unwrap(), "older");
        let _ = fs::remove_dir_all(&backup_folder);
    }

    #[test]
    fn only_the_newest_copies_are_kept_and_other_files_are_left_alone() {
        let backup_folder = create_empty_test_folder("newest");
        fs::write(backup_folder.join("notes.txt"), "not a copy").unwrap();
        for copy_number in 0..CLAUDE_CODE_SETTINGS_BACKUP_KEPT_COUNT as u64 + 3 {
            keep_claude_code_settings_copy(&backup_folder, &format!("version {copy_number}"), moment(1_793_697_307 + copy_number))
                .unwrap();
        }
        let kept_copies = list_settings_copies(&backup_folder).unwrap();
        assert_eq!(kept_copies.len(), CLAUDE_CODE_SETTINGS_BACKUP_KEPT_COUNT);
        assert_eq!(fs::read_to_string(&kept_copies[0]).unwrap(), "version 3");
        assert!(backup_folder.join("notes.txt").exists());
        let _ = fs::remove_dir_all(&backup_folder);
    }

    #[test]
    fn the_old_single_copy_moves_in_once_under_the_time_of_its_last_change() {
        let test_folder = create_empty_test_folder("old-copy");
        let settings_file_path = test_folder.join("settings.json");
        let old_backup_path = test_folder.join("settings.json.crest-backup");
        fs::write(&old_backup_path, "the user's file from before").unwrap();
        fs::File::options().write(true).open(&old_backup_path).unwrap().set_modified(moment(1_793_697_307)).unwrap();
        let backup_folder = test_folder.join("backups");
        move_in_old_crest_backup(&settings_file_path, &backup_folder).unwrap();
        assert!(!old_backup_path.exists());
        assert_eq!(copy_names(&backup_folder), ["settings.json.2026-11-03_09-15-07-UTC.json"]);
        move_in_old_crest_backup(&settings_file_path, &backup_folder).unwrap();
        assert_eq!(copy_names(&backup_folder).len(), 1);
        let _ = fs::remove_dir_all(&test_folder);
    }

    #[test]
    fn the_identifier_matches_the_one_tauri_uses_for_crests_folder() {
        let tauri_configuration: Value = serde_json::from_str(include_str!("../../../tauri.conf.json")).unwrap();
        assert_eq!(tauri_configuration["identifier"], CREST_APP_IDENTIFIER);
    }
}
