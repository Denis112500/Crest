//! The settings window's commands for the Claude Code integration: show its state, switch it
//! on or off, and switch answering permission requests in the pill on or off. Each change
//! carries the fingerprint of the preview the user confirmed (`claude_code_switch_change_preview.rs`).

use std::sync::PoisonError;

use serde::Serialize;
use tauri::State;

use crate::activity_core::{SharedActivityArbiter, SharedActivitySourceRegistry};
use crate::activity_sources::claude_code::{
    change_claude_code_permission_answering, claude_code_settings_file_path, turn_claude_code_integration_off,
    turn_claude_code_integration_on, ClaudeCodePermissionAnsweringSwitch, ClaudeCodeSettingsChangeOutcome,
};
use crate::settings_window::claude_code_switch_change_preview::{
    describe_claude_code_switch_change, ClaudeCodeSwitchChange, ClaudeCodeSwitchChangePreview,
};
use crate::user_settings_store::{CrestUserSettings, CrestUserSettingsStore};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCodeIntegrationSetting {
    is_integration_on: bool,
    is_permission_answering_on: bool,
    /// Where Claude Code's settings file is, shown so the user knows which file changes.
    claude_code_settings_file_path: String,
}

#[derive(Serialize)]
#[serde(tag = "outcome", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum ClaudeCodeSwitchChangeOutcome {
    /// The switches show this state.
    Changed { current_setting: ClaudeCodeIntegrationSetting },
    /// Nothing changed, because Claude Code's file isn't the one the user confirmed: the card
    /// shows this preview of it to confirm again.
    SettingsFileChangedSincePreview { current_preview: ClaudeCodeSwitchChangePreview },
}

#[tauri::command]
pub fn read_claude_code_integration_setting(
    settings_store: State<'_, CrestUserSettingsStore>,
) -> Result<ClaudeCodeIntegrationSetting, String> {
    describe_claude_code_integration(&settings_store.read_current_settings())
}

/// Saves the choice only once Claude Code's settings were changed, so the switch never shows
/// "on" while the hooks are missing.
#[tauri::command]
pub fn change_claude_code_integration_setting(
    settings_store: State<'_, CrestUserSettingsStore>,
    shared_activity_source_registry: State<'_, SharedActivitySourceRegistry>,
    shared_activity_arbiter: State<'_, SharedActivityArbiter>,
    permission_answering_switch: State<'_, ClaudeCodePermissionAnsweringSwitch>,
    should_integration_be_on: bool,
    expected_settings_file_fingerprint: String,
) -> Result<ClaudeCodeSwitchChangeOutcome, String> {
    let mut activity_source_registry = shared_activity_source_registry.lock().unwrap_or_else(PoisonError::into_inner);
    let change_outcome = if should_integration_be_on {
        turn_claude_code_integration_on(
            &mut activity_source_registry,
            &shared_activity_arbiter,
            &permission_answering_switch,
            Some(&expected_settings_file_fingerprint),
        )?
    } else {
        let switch_off_result =
            turn_claude_code_integration_off(&mut activity_source_registry, Some(&expected_settings_file_fingerprint));
        // Saved as off even if Claude Code's file couldn't be cleaned up: Crest stopped listening.
        if switch_off_result != Ok(ClaudeCodeSettingsChangeOutcome::FileChangedSincePreview) {
            settings_store.change_and_save(|settings| settings.is_claude_code_integration_on = false)?;
        }
        switch_off_result?
    };
    if change_outcome == ClaudeCodeSettingsChangeOutcome::FileChangedSincePreview {
        let switch_change =
            if should_integration_be_on { ClaudeCodeSwitchChange::IntegrationOn } else { ClaudeCodeSwitchChange::IntegrationOff };
        return report_settings_file_changed_since_preview(&settings_store, &permission_answering_switch, switch_change);
    }
    if should_integration_be_on {
        settings_store.change_and_save(|settings| settings.is_claude_code_integration_on = true)?;
    }
    report_switch_changed(&settings_store)
}

/// Like the integration switch: saved only once Claude Code's settings were changed.
#[tauri::command]
pub fn change_claude_code_permission_answering_setting(
    settings_store: State<'_, CrestUserSettingsStore>,
    permission_answering_switch: State<'_, ClaudeCodePermissionAnsweringSwitch>,
    should_permission_answering_be_on: bool,
    expected_settings_file_fingerprint: String,
) -> Result<ClaudeCodeSwitchChangeOutcome, String> {
    let is_integration_on = settings_store.read_current_settings().is_claude_code_integration_on;
    let change_outcome = change_claude_code_permission_answering(
        &permission_answering_switch,
        should_permission_answering_be_on,
        is_integration_on,
        Some(&expected_settings_file_fingerprint),
    )?;
    if change_outcome == ClaudeCodeSettingsChangeOutcome::FileChangedSincePreview {
        let switch_change = if should_permission_answering_be_on {
            ClaudeCodeSwitchChange::AnsweringOn
        } else {
            ClaudeCodeSwitchChange::AnsweringOff
        };
        return report_settings_file_changed_since_preview(&settings_store, &permission_answering_switch, switch_change);
    }
    settings_store.change_and_save(|settings| settings.is_claude_code_permission_answering_on = should_permission_answering_be_on)?;
    report_switch_changed(&settings_store)
}

fn report_switch_changed(settings_store: &CrestUserSettingsStore) -> Result<ClaudeCodeSwitchChangeOutcome, String> {
    Ok(ClaudeCodeSwitchChangeOutcome::Changed {
        current_setting: describe_claude_code_integration(&settings_store.read_current_settings())?,
    })
}

fn report_settings_file_changed_since_preview(
    settings_store: &CrestUserSettingsStore,
    permission_answering_switch: &ClaudeCodePermissionAnsweringSwitch,
    switch_change: ClaudeCodeSwitchChange,
) -> Result<ClaudeCodeSwitchChangeOutcome, String> {
    Ok(ClaudeCodeSwitchChangeOutcome::SettingsFileChangedSincePreview {
        current_preview: describe_claude_code_switch_change(
            &settings_store.read_current_settings(),
            permission_answering_switch,
            switch_change,
        )?,
    })
}

fn describe_claude_code_integration(current_settings: &CrestUserSettings) -> Result<ClaudeCodeIntegrationSetting, String> {
    Ok(ClaudeCodeIntegrationSetting {
        is_integration_on: current_settings.is_claude_code_integration_on,
        is_permission_answering_on: current_settings.is_claude_code_permission_answering_on,
        claude_code_settings_file_path: claude_code_settings_file_path()?.display().to_string(),
    })
}
