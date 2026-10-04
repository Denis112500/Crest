use tauri::{AppHandle, Emitter, State, WebviewWindow};

use crate::backend_constants::PILL_WINDOW_LABEL;
use crate::ipc_channel_names::PILL_DISPLAY_CHANGED_EVENT;
use crate::launch_at_login::{is_launch_at_login_enabled, set_launch_at_login};
use crate::media::{MediaPlayerFilter, MediaPlayerFilterControl};
use crate::pill_window::{describe_pill_display_options, read_connected_displays, PillDisplayOption};
use crate::settings_window::allowed_player_options::{describe_allowed_player_options, AllowedPlayerOptions};
use crate::user_settings_store::{CrestUserSettings, CrestUserSettingsStore};

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

#[tauri::command]
pub fn list_allowed_player_options(
    settings_store: State<'_, CrestUserSettingsStore>,
    player_filter_control: State<'_, MediaPlayerFilterControl>,
) -> AllowedPlayerOptions {
    describe_current_player_options(&settings_store.read_current_settings(), &player_filter_control)
}

#[tauri::command]
pub fn change_allowed_players(
    settings_store: State<'_, CrestUserSettingsStore>,
    player_filter_control: State<'_, MediaPlayerFilterControl>,
    allowed_app_identifier_fragments: Vec<String>,
) -> Result<AllowedPlayerOptions, String> {
    save_and_apply_player_settings(&settings_store, &player_filter_control, |settings| {
        settings.allowed_media_app_identifier_fragments = allowed_app_identifier_fragments;
    })
}

#[tauri::command]
pub fn change_show_every_player(
    settings_store: State<'_, CrestUserSettingsStore>,
    player_filter_control: State<'_, MediaPlayerFilterControl>,
    should_show_every_player: bool,
) -> Result<AllowedPlayerOptions, String> {
    save_and_apply_player_settings(&settings_store, &player_filter_control, |settings| {
        settings.show_every_media_player = Some(should_show_every_player);
    })
}

/// Saves the change, hands the resulting filter to the running media source (so the pill
/// follows at once) and returns the updated options, which the window shows.
fn save_and_apply_player_settings(
    settings_store: &CrestUserSettingsStore,
    player_filter_control: &MediaPlayerFilterControl,
    apply_change: impl FnOnce(&mut CrestUserSettings),
) -> Result<AllowedPlayerOptions, String> {
    settings_store.change_and_save(apply_change)?;
    let current_settings = settings_store.read_current_settings();
    player_filter_control.replace_media_player_filter(current_settings.media_player_filter())?;
    Ok(describe_current_player_options(&current_settings, player_filter_control))
}

fn describe_current_player_options(
    current_settings: &CrestUserSettings,
    player_filter_control: &MediaPlayerFilterControl,
) -> AllowedPlayerOptions {
    describe_allowed_player_options(
        current_settings.media_player_filter() == MediaPlayerFilter::EveryPlayer,
        &current_settings.allowed_media_app_identifier_fragments,
        &player_filter_control.list_open_player_app_identifiers(),
    )
}
