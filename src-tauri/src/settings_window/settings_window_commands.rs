use tauri::{AppHandle, Emitter, State, WebviewWindow};

use crate::backend_constants::PILL_WINDOW_LABEL;
use crate::ipc_channel_names::PILL_DISPLAY_CHANGED_EVENT;
use crate::launch_at_login::{is_launch_at_login_enabled, set_launch_at_login};
use crate::pill_window::{describe_pill_display_options, read_connected_displays, PillDisplayOption};
use crate::user_settings_store::CrestUserSettingsStore;

// Only the settings window may call these (capabilities/settings_window.json); the pill page
// has no permission for them.

#[tauri::command]
pub fn read_launch_at_login_setting(crest_app: AppHandle) -> bool {
    is_launch_at_login_enabled(&crest_app)
}

/// Returns what Windows reports afterwards, which the switch then shows.
#[tauri::command]
pub fn change_launch_at_login_setting(crest_app: AppHandle, should_launch_at_login: bool) -> bool {
    set_launch_at_login(&crest_app, should_launch_at_login)
}

#[tauri::command]
pub fn list_pill_display_options(
    settings_window: WebviewWindow,
    settings_store: State<'_, CrestUserSettingsStore>,
) -> Result<Vec<PillDisplayOption>, String> {
    let connected_displays = read_connected_displays(&settings_window)?;
    let chosen_display_name = settings_store.read_current_settings().pill_display_name;
    Ok(describe_pill_display_options(&connected_displays, chosen_display_name.as_deref()))
}

/// Saves the choice, then asks the pill page to place itself again: the page knows the
/// pill's size and its current interactive area, Rust knows the monitor.
#[tauri::command]
pub fn choose_pill_display(
    crest_app: AppHandle,
    settings_store: State<'_, CrestUserSettingsStore>,
    display_name: Option<String>,
) -> Result<(), String> {
    settings_store.change_and_save(|settings| settings.pill_display_name = display_name)?;
    crest_app.emit_to(PILL_WINDOW_LABEL, PILL_DISPLAY_CHANGED_EVENT, ()).map_err(|error| error.to_string())
}
