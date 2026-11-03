//! Switching the Claude Code integration on and off: Claude Code's hooks and Crest's listening
//! source always change together. Used by the settings window and at startup.

use crate::activity_core::{ActivitySourceRegistry, SharedActivityArbiter};
use crate::activity_sources::claude_code::claude_code_activity_source::ClaudeCodeActivitySource;
use crate::activity_sources::claude_code::claude_code_permission_answering_switch::ClaudeCodePermissionAnsweringSwitch;
use crate::activity_sources::claude_code::claude_code_settings_change::{
    change_claude_code_settings, ClaudeCodeSettingsChange, ClaudeCodeSettingsChangeOutcome,
};
use crate::ipc_channel_names::CLAUDE_CODE_ACTIVITY_KIND;

/// Also used at every start while the integration is on (with no fingerprint): the hooks then
/// point at the Crest that is running now (another install folder, or a dev build).
pub fn turn_claude_code_integration_on(
    activity_source_registry: &mut ActivitySourceRegistry,
    shared_activity_arbiter: &SharedActivityArbiter,
    permission_answering_switch: &ClaudeCodePermissionAnsweringSwitch,
    expected_settings_file_fingerprint: Option<&str>,
) -> Result<ClaudeCodeSettingsChangeOutcome, String> {
    let crest_executable_path = std::env::current_exe().map_err(|error| error.to_string())?;
    if !activity_source_registry.is_registered(CLAUDE_CODE_ACTIVITY_KIND) {
        let claude_code_source = ClaudeCodeActivitySource::new(permission_answering_switch.clone());
        activity_source_registry.start_and_register(Box::new(claude_code_source), shared_activity_arbiter)?;
    }
    let crest_hooks_change = ClaudeCodeSettingsChange::AddCrestHooks {
        crest_executable_path,
        should_add_permission_hook: permission_answering_switch.is_answering_on(),
    };
    let change_outcome = change_claude_code_settings(&crest_hooks_change, expected_settings_file_fingerprint);
    if change_outcome != Ok(ClaudeCodeSettingsChangeOutcome::Applied) {
        activity_source_registry.stop_and_unregister(CLAUDE_CODE_ACTIVITY_KIND);
    }
    change_outcome
}

/// Stops listening even if Claude Code's settings can't be changed; the error then says why
/// its hooks are still there (they'd only find no Crest listening, which costs nothing). A file
/// changed since the preview stops nothing: the user decides again on the new preview.
pub fn turn_claude_code_integration_off(
    activity_source_registry: &mut ActivitySourceRegistry,
    expected_settings_file_fingerprint: Option<&str>,
) -> Result<ClaudeCodeSettingsChangeOutcome, String> {
    let change_outcome =
        change_claude_code_settings(&ClaudeCodeSettingsChange::RemoveCrestHooks, expected_settings_file_fingerprint);
    if change_outcome != Ok(ClaudeCodeSettingsChangeOutcome::FileChangedSincePreview) {
        activity_source_registry.stop_and_unregister(CLAUDE_CODE_ACTIVITY_KIND);
    }
    change_outcome
}

/// "Answer permission requests in the pill": Claude Code's hooks change first (only while the
/// integration is on), so the switch never says off while the permission hook is still there.
pub fn change_claude_code_permission_answering(
    permission_answering_switch: &ClaudeCodePermissionAnsweringSwitch,
    should_answering_be_on: bool,
    is_integration_on: bool,
    expected_settings_file_fingerprint: Option<&str>,
) -> Result<ClaudeCodeSettingsChangeOutcome, String> {
    if is_integration_on {
        let crest_hooks_change = ClaudeCodeSettingsChange::AddCrestHooks {
            crest_executable_path: std::env::current_exe().map_err(|error| error.to_string())?,
            should_add_permission_hook: should_answering_be_on,
        };
        let change_outcome = change_claude_code_settings(&crest_hooks_change, expected_settings_file_fingerprint)?;
        if change_outcome == ClaudeCodeSettingsChangeOutcome::FileChangedSincePreview {
            return Ok(change_outcome);
        }
    }
    permission_answering_switch.change(should_answering_be_on);
    Ok(ClaudeCodeSettingsChangeOutcome::Applied)
}
