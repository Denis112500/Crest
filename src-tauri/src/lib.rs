mod activity_core;
mod activity_sources;
mod backend_constants;
mod ipc_channel_names;
mod media;
mod pill_window;
mod user_settings_file;

use std::sync::{Arc, Mutex};

use tauri::{Emitter, Manager};

use activity_core::{ActivityArbiter, ActivityPublisher, ActivitySource, SharedActivityArbiter};
use activity_sources::MusicActivitySource;
use backend_constants::PILL_WINDOW_LABEL;
use ipc_channel_names::PILL_PRESENTATION_CHANGED_EVENT;
use media::CurrentPlatformMediaSource;
use pill_window::{CurrentPlatformPillWindow, PillWindowPlatform};
use user_settings_file::load_crest_user_settings;

pub fn run_crest_app() {
    tauri::Builder::default()
        .setup(|crest_app| {
            let pill_window = crest_app
                .get_webview_window(PILL_WINDOW_LABEL)
                .ok_or("the pill window from tauri.conf.json was not created")?;
            CurrentPlatformPillWindow::prepare_pill_window_as_overlay(&pill_window)?;

            let crest_app_handle = crest_app.handle().clone();
            let shared_activity_arbiter: SharedActivityArbiter =
                Arc::new(Mutex::new(ActivityArbiter::new(Box::new(move |pill_presentation| {
                    if let Err(emit_error) =
                        crest_app_handle.emit_to(PILL_WINDOW_LABEL, PILL_PRESENTATION_CHANGED_EVENT, pill_presentation)
                    {
                        eprintln!("Crest: could not send the pill presentation to the window: {emit_error}");
                    }
                }))));
            crest_app.manage(Arc::clone(&shared_activity_arbiter));

            let crest_user_settings = load_crest_user_settings(&crest_app.path().app_config_dir()?);
            let media_source =
                CurrentPlatformMediaSource::new(crest_user_settings.allowed_media_app_identifier_fragments);
            let mut activity_sources: Vec<Box<dyn ActivitySource>> =
                vec![Box::new(MusicActivitySource::new(Box::new(media_source)))];
            for activity_source in &mut activity_sources {
                let activity_publisher =
                    ActivityPublisher::new(activity_source.activity_kind(), Arc::clone(&shared_activity_arbiter));
                activity_source.start_publishing(activity_publisher)?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            pill_window::pill_window_commands::place_pill_window_at_top_center,
            pill_window::pill_window_commands::reveal_pill_window,
            pill_window::pill_window_commands::set_pill_interactive_area,
            activity_core::pill_presentation_command::get_current_pill_presentation,
        ])
        .run(tauri::generate_context!())
        .expect("Crest failed to start the Tauri application");
}
