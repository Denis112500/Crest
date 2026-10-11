//! Switching the Claude Code integration on and off: Claude Code's hooks and Crest's listening
//! source always change together. Used by the settings window and at startup.

use crate::activity_core::{ActivitySourceRegistry, SharedActivityArbiter};
use crate::activity_sources::claude_code::claude_code_activity_source::ClaudeCodeActivitySource;
use crate::activity_sources::claude_code::claude_code_permission_answering_switch::ClaudeCodePermissionAnsweringSwitch;
use crate::activity_sources::claude_code::claude_code_settings_file::{
    add_crest_hooks_to_claude_code_settings, remove_crest_hooks_from_claude_code_settings,
};
use crate::ipc_channel_names::CLAUDE_CODE_ACTIVITY_KIND;

/// Also used at every start while the integration is on: the hooks then point at the Crest
/// that is running now (another install folder, or a dev build).
pub fn turn_claude_code_integration_on(
    activity_source_registry: &mut ActivitySourceRegistry,
    shared_activity_arbiter: &SharedActivityArbiter,
    permission_answering_switch: &ClaudeCodePermissionAnsweringSwitch,
) -> Result<(), String> {
    let crest_executable_path = std::env::current_exe().map_err(|error| error.to_string())?;
    if !activity_source_registry.is_registered(CLAUDE_CODE_ACTIVITY_KIND) {
        let claude_code_source = ClaudeCodeActivitySource::new(permission_answering_switch.clone());
        activity_source_registry.start_and_register(Box::new(claude_code_source), shared_activity_arbiter)?;
    }
    let should_add_permission_hook = permission_answering_switch.is_answering_on();
    add_crest_hooks_to_claude_code_settings(&crest_executable_path, should_add_permission_hook).inspect_err(|_| {
        activity_source_registry.stop_and_unregister(CLAUDE_CODE_ACTIVITY_KIND);
    })
}

/// Stops listening even if Claude Code's settings can't be changed; the error then says why
/// its hooks are still there (they'd only find no Crest listening, which costs nothing).
pub fn turn_claude_code_integration_off(activity_source_registry: &mut ActivitySourceRegistry) -> Result<(), String> {
    activity_source_registry.stop_and_unregister(CLAUDE_CODE_ACTIVITY_KIND);
    remove_crest_hooks_from_claude_code_settings()
}

/// "Answer permission requests in the pill": Claude Code's hooks change first (only while the
/// integration is on), so the switch never says off while the permission hook is still there.
pub fn change_claude_code_permission_answering(
    permission_answering_switch: &ClaudeCodePermissionAnsweringSwitch,
    should_answering_be_on: bool,
    is_integration_on: bool,
) -> Result<(), String> {
    if is_integration_on {
        let crest_executable_path = std::env::current_exe().map_err(|error| error.to_string())?;
        add_crest_hooks_to_claude_code_settings(&crest_executable_path, should_answering_be_on)?;
    }
    permission_answering_switch.change(should_answering_be_on);
    Ok(())
}
