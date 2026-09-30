use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::App;

const TRAY_TOOLTIP_TEXT: &str = "Crest";
const QUIT_MENU_ITEM_IDENTIFIER: &str = "quit-crest";
const QUIT_MENU_ITEM_LABEL: &str = "Quit Crest";

/// The notification-area icon. The pill has no window chrome and no taskbar button, so
/// this is the one place to quit the app.
pub fn create_crest_tray_icon(crest_app: &App) -> tauri::Result<()> {
    let quit_menu_item =
        MenuItem::with_id(crest_app, QUIT_MENU_ITEM_IDENTIFIER, QUIT_MENU_ITEM_LABEL, true, None::<&str>)?;
    let tray_menu = Menu::with_items(crest_app, &[&quit_menu_item])?;
    let mut tray_icon_builder = TrayIconBuilder::new()
        .tooltip(TRAY_TOOLTIP_TEXT)
        .menu(&tray_menu)
        .on_menu_event(|crest_app_handle, menu_event| {
            if menu_event.id().as_ref() == QUIT_MENU_ITEM_IDENTIFIER {
                crest_app_handle.exit(0);
            }
        });
    // The app icon from `bundle.icon` in tauri.conf.json, embedded at compile time.
    if let Some(crest_app_icon) = crest_app.default_window_icon() {
        tray_icon_builder = tray_icon_builder.icon(crest_app_icon.clone());
    }
    // Tauri keeps the built tray icon alive for the app's lifetime.
    tray_icon_builder.build(crest_app)?;
    Ok(())
}
