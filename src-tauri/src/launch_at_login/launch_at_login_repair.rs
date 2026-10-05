use tauri::{AppHandle, Runtime};

use crate::launch_at_login::launch_at_login_switch::set_launch_at_login;
use crate::launch_at_login::read_launch_at_login_entry_state;
use crate::user_settings_store::CrestUserSettingsStore;

/// What Windows' start-at-login entry for Crest looks like right now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunEntryState {
    Missing,
    /// It names a program that's gone, e.g. a Crest that was uninstalled or moved: Windows
    /// would start nothing at login.
    StartsMissingProgram,
    StartsExistingProgram,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LaunchAtLoginRepair {
    /// Point the entry at the running Crest (and remember the choice). Crest's installer deletes
    /// the entry when an update uninstalls the old version first (its preselected choice).
    RegisterAgain,
    /// No choice saved yet (versions before 0.3.0 didn't save it) and a working entry exists:
    /// adopt it as the user's choice, so a later update can be repaired.
    RememberAsWanted,
    NothingToDo,
}

/// Only ever repairs towards what the user chose. It never removes an entry; an entry Task
/// Manager disabled still counts as there, so that choice is left alone; and an entry that
/// starts another existing Crest (e.g. a test copy) isn't taken over.
pub fn decide_launch_at_login_repair(
    saved_launch_at_login_choice: Option<bool>,
    run_entry_state: RunEntryState,
) -> LaunchAtLoginRepair {
    match (saved_launch_at_login_choice, run_entry_state) {
        (Some(true), RunEntryState::Missing | RunEntryState::StartsMissingProgram) => LaunchAtLoginRepair::RegisterAgain,
        // Someone switched it on at some point; it only points at a Crest that's gone.
        (None, RunEntryState::StartsMissingProgram) => LaunchAtLoginRepair::RegisterAgain,
        (None, RunEntryState::StartsExistingProgram) => LaunchAtLoginRepair::RememberAsWanted,
        _ => LaunchAtLoginRepair::NothingToDo,
    }
}

/// Runs once at startup. Skipped in development builds: they would register
/// `target\debug\crest.exe`, which can't start at login without the Vite dev server.
pub fn repair_launch_at_login_after_update<R: Runtime>(crest_app: &AppHandle<R>, settings_store: &CrestUserSettingsStore) {
    if cfg!(debug_assertions) {
        return;
    }
    let saved_launch_at_login_choice = settings_store.read_current_settings().should_launch_at_login;
    let run_entry_state = read_launch_at_login_entry_state(&crest_app.package_info().name);
    match decide_launch_at_login_repair(saved_launch_at_login_choice, run_entry_state) {
        LaunchAtLoginRepair::RegisterAgain => {
            set_launch_at_login(crest_app, true);
            remember_launch_at_login_as_wanted(settings_store);
        }
        LaunchAtLoginRepair::RememberAsWanted => remember_launch_at_login_as_wanted(settings_store),
        LaunchAtLoginRepair::NothingToDo => {}
    }
}

fn remember_launch_at_login_as_wanted(settings_store: &CrestUserSettingsStore) {
    if let Err(save_error) = settings_store.change_and_save(|settings| settings.should_launch_at_login = Some(true)) {
        eprintln!("Crest: could not save that it starts with Windows: {save_error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_again_when_wanted_and_missing_or_pointing_at_a_removed_crest() {
        assert_eq!(decide_launch_at_login_repair(Some(true), RunEntryState::Missing), LaunchAtLoginRepair::RegisterAgain);
        assert_eq!(
            decide_launch_at_login_repair(Some(true), RunEntryState::StartsMissingProgram),
            LaunchAtLoginRepair::RegisterAgain
        );
        assert_eq!(
            decide_launch_at_login_repair(Some(true), RunEntryState::StartsExistingProgram),
            LaunchAtLoginRepair::NothingToDo
        );
    }

    #[test]
    fn leaves_the_entry_alone_when_the_user_switched_it_off_in_crest() {
        assert_eq!(decide_launch_at_login_repair(Some(false), RunEntryState::Missing), LaunchAtLoginRepair::NothingToDo);
        assert_eq!(
            decide_launch_at_login_repair(Some(false), RunEntryState::StartsExistingProgram),
            LaunchAtLoginRepair::NothingToDo
        );
    }

    #[test]
    fn handles_entries_from_versions_that_did_not_save_the_choice() {
        assert_eq!(
            decide_launch_at_login_repair(None, RunEntryState::StartsExistingProgram),
            LaunchAtLoginRepair::RememberAsWanted
        );
        assert_eq!(
            decide_launch_at_login_repair(None, RunEntryState::StartsMissingProgram),
            LaunchAtLoginRepair::RegisterAgain
        );
        assert_eq!(decide_launch_at_login_repair(None, RunEntryState::Missing), LaunchAtLoginRepair::NothingToDo);
    }
}
