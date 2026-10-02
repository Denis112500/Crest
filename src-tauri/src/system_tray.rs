use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::App;

use crate::launch_at_login::{is_launch_at_login_enabled, toggle_launch_at_login};

const TRAY_TOOLTIP_TEXT: &str = "Crest";
const LAUNCH_AT_LOGIN_MENU_ITEM_IDENTIFIER: &str = "launch-at-login";
const LAUNCH_AT_LOGIN_MENU_ITEM_LABEL: &str = "Start with Windows";
const QUIT_MENU_ITEM_IDENTIFIER: &str = "quit-crest";
const QUIT_MENU_ITEM_LABEL: &str = "Quit Crest";

/// The notification-area icon. The pill has no window chrome and no taskbar button, so
/// this is the place for Crest's few settings and for quitting.
pub fn create_crest_tray_icon(crest_app: &App) -> tauri::Result<()> {
    let launch_at_login_menu_item = CheckMenuItem::with_id(
        crest_app,
        LAUNCH_AT_LOGIN_MENU_ITEM_IDENTIFIER,
        LAUNCH_AT_LOGIN_MENU_ITEM_LABEL,
        true,
        is_launch_at_login_enabled(crest_app),
        None::<&str>,
    )?;
    let quit_menu_item =
        MenuItem::with_id(crest_app, QUIT_MENU_ITEM_IDENTIFIER, QUIT_MENU_ITEM_LABEL, true, None::<&str>)?;
    let tray_menu = Menu::with_items(
        crest_app,
        &[&launch_at_login_menu_item, &PredefinedMenuItem::separator(crest_app)?, &quit_menu_item],
    )?;
    let mut tray_icon_builder = TrayIconBuilder::new()
        .tooltip(TRAY_TOOLTIP_TEXT)
        .menu(&tray_menu)
        .on_menu_event(move |crest_app_handle, menu_event| match menu_event.id().as_ref() {
            LAUNCH_AT_LOGIN_MENU_ITEM_IDENTIFIER => {
                // Windows already flipped the checkmark on click; overwrite it with what the
                // OS really reports, in case the switch failed.
                let is_enabled_now = toggle_launch_at_login(crest_app_handle);
                if let Err(menu_error) = launch_at_login_menu_item.set_checked(is_enabled_now) {
                    eprintln!("Crest: could not update the tray checkmark: {menu_error}");
                }
            }
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
