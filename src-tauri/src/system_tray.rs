use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::App;

use crate::settings_window::open_or_focus_settings_window;

const TRAY_TOOLTIP_TEXT: &str = "Crest";
const OPEN_SETTINGS_MENU_ITEM_IDENTIFIER: &str = "open-settings";
const OPEN_SETTINGS_MENU_ITEM_LABEL: &str = "Settings…";
const QUIT_MENU_ITEM_IDENTIFIER: &str = "quit-crest";
const QUIT_MENU_ITEM_LABEL: &str = "Quit Crest";

/// The notification-area icon. The pill has no window chrome and no taskbar button, so
/// this is the way to the settings window and to quitting.
pub fn create_crest_tray_icon(crest_app: &App) -> tauri::Result<()> {
    let open_settings_menu_item = MenuItem::with_id(
        crest_app,
        OPEN_SETTINGS_MENU_ITEM_IDENTIFIER,
        OPEN_SETTINGS_MENU_ITEM_LABEL,
        true,
        None::<&str>,
    )?;
    let quit_menu_item =
        MenuItem::with_id(crest_app, QUIT_MENU_ITEM_IDENTIFIER, QUIT_MENU_ITEM_LABEL, true, None::<&str>)?;
    let tray_menu = Menu::with_items(
        crest_app,
        &[&open_settings_menu_item, &PredefinedMenuItem::separator(crest_app)?, &quit_menu_item],
    )?;
    let mut tray_icon_builder = TrayIconBuilder::new()
        .tooltip(TRAY_TOOLTIP_TEXT)
        .menu(&tray_menu)
        .on_menu_event(move |crest_app_handle, menu_event| match menu_event.id().as_ref() {
            OPEN_SETTINGS_MENU_ITEM_IDENTIFIER => open_or_focus_settings_window(crest_app_handle),
            QUIT_MENU_ITEM_IDENTIFIER => crest_app_handle.exit(0),
            _ => {}
        });
    // The app icon from `bundle.icon` in tauri.conf.json, embedded at compile time.
    if let Some(crest_app_icon) = crest_app.default_window_icon() {
        tray_icon_builder = tray_icon_builder.icon(crest_app_icon.clone());
    }
    // Tauri keeps the built tray icon alive for the app's lifetime.
    tray_icon_builder.build(crest_app)?;
    Ok(())
}
