mod backend_constants;
mod pill_window;

use tauri::Manager;

use backend_constants::PILL_WINDOW_LABEL;
use pill_window::{CurrentPlatformPillWindow, PillWindowPlatform};

pub fn run_crest_app() {
    tauri::Builder::default()
        .setup(|crest_app| {
            let pill_window = crest_app
                .get_webview_window(PILL_WINDOW_LABEL)
                .ok_or("the pill window from tauri.conf.json was not created")?;
            CurrentPlatformPillWindow::prepare_pill_window_as_overlay(&pill_window)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            pill_window::pill_window_commands::place_pill_window_at_top_center,
            pill_window::pill_window_commands::reveal_pill_window,
        ])
        .run(tauri::generate_context!())
        .expect("Crest failed to start the Tauri application");
}
