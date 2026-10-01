mod activity_core;
mod activity_sources;
mod backend_constants;
mod ipc_channel_names;
mod media;
mod pill_window;
mod system_tray;
mod user_settings_file;

use std::sync::{Arc, Mutex};

use tauri::{Emitter, Manager};

use activity_core::{ActivityArbiter, ActivitySourceRegistry, PillVisibilityController, SharedActivityArbiter};
use activity_sources::MusicActivitySource;
use backend_constants::PILL_WINDOW_LABEL;
use ipc_channel_names::{PILL_PRESENTATION_CHANGED_EVENT, PILL_VISIBILITY_CHANGED_EVENT};
use media::CurrentPlatformMediaSource;
use pill_window::{CurrentPlatformPillWindow, PillWindowPlatform};
use system_tray::create_crest_tray_icon;
use user_settings_file::load_crest_user_settings;

pub fn run_crest_app() {
    tauri::Builder::default()
        // Registered first, as the plugin requires, so a second copy exits before it creates
        // its own pill and tray icon. Two copies would stack two pills in the same spot. The
        // running copy has nothing to bring forward (the pill appears only with music), so
        // the second launch is simply ignored.
        .plugin(tauri_plugin_single_instance::init(|_running_crest_app, _second_launch_arguments, _second_launch_directory| {}))
        .setup(|crest_app| {
            let pill_window = crest_app
                .get_webview_window(PILL_WINDOW_LABEL)
                .ok_or("the pill window from tauri.conf.json was not created")?;
            CurrentPlatformPillWindow::prepare_pill_window_as_overlay(&pill_window)?;
            create_crest_tray_icon(crest_app)?;

            let visibility_app_handle = crest_app.handle().clone();
            let pill_visibility_controller = PillVisibilityController::new(Box::new(move |is_pill_visible| {
                if let Err(emit_error) =
                    visibility_app_handle.emit_to(PILL_WINDOW_LABEL, PILL_VISIBILITY_CHANGED_EVENT, is_pill_visible)
                {
                    eprintln!("Crest: could not send the pill visibility to the window: {emit_error}");
                }
            }));
            crest_app.manage(pill_visibility_controller.clone());

            let presentation_app_handle = crest_app.handle().clone();
            let shared_activity_arbiter: SharedActivityArbiter =
                Arc::new(Mutex::new(ActivityArbiter::new(Box::new(move |pill_presentation| {
                    if let Err(emit_error) = presentation_app_handle.emit_to(
                        PILL_WINDOW_LABEL,
                        PILL_PRESENTATION_CHANGED_EVENT,
                        pill_presentation,
                    ) {
                        eprintln!("Crest: could not send the pill presentation to the window: {emit_error}");
                    }
                    pill_visibility_controller.handle_presentation_change(pill_presentation);
                }))));
            crest_app.manage(Arc::clone(&shared_activity_arbiter));

            let crest_user_settings = load_crest_user_settings(&crest_app.path().app_config_dir()?);
            let media_source =
                CurrentPlatformMediaSource::new(crest_user_settings.allowed_media_app_identifier_fragments);
            let mut activity_source_registry = ActivitySourceRegistry::default();
            activity_source_registry
                .start_and_register(Box::new(MusicActivitySource::new(Box::new(media_source))), &shared_activity_arbiter)?;
            crest_app.manage(Mutex::new(activity_source_registry));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            pill_window::pill_window_commands::place_pill_window_at_top_center,
            pill_window::pill_window_commands::reveal_pill_window,
            pill_window::pill_window_commands::conceal_pill_window,
            pill_window::pill_window_commands::set_pill_interactive_area,
            activity_core::pill_presentation_command::get_current_pill_presentation,
            activity_core::pill_visibility_command::get_current_pill_visibility,
            activity_core::activity_action_command::perform_activity_action,
        ])
        .run(tauri::generate_context!())
        .expect("Crest failed to start the Tauri application");
}
