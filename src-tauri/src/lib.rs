mod backend_constants;
mod media;
mod media_console_preview;
mod pill_window;
mod user_settings_file;

use tauri::Manager;

use backend_constants::PILL_WINDOW_LABEL;
use media::{CurrentPlatformMediaSource, MediaSource};
use media_console_preview::print_media_snapshot_to_console;
use pill_window::{CurrentPlatformPillWindow, PillWindowPlatform};
use user_settings_file::load_crest_user_settings;

pub fn run_crest_app() {
    tauri::Builder::default()
        .setup(|crest_app| {
            let pill_window = crest_app
                .get_webview_window(PILL_WINDOW_LABEL)
                .ok_or("the pill window from tauri.conf.json was not created")?;
            CurrentPlatformPillWindow::prepare_pill_window_as_overlay(&pill_window)?;

            let crest_user_settings = load_crest_user_settings(&crest_app.path().app_config_dir()?);
            let mut media_source =
                CurrentPlatformMediaSource::new(crest_user_settings.allowed_media_app_identifier_fragments);
            media_source.start_watching_media_sessions(Box::new(print_media_snapshot_to_console))?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            pill_window::pill_window_commands::place_pill_window_at_top_center,
            pill_window::pill_window_commands::reveal_pill_window,
        ])
        .run(tauri::generate_context!())
        .expect("Crest failed to start the Tauri application");
}
