use std::thread;

use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

use crate::backend_constants::{
    SETTINGS_WINDOW_LABEL, SETTINGS_WINDOW_LOGICAL_HEIGHT, SETTINGS_WINDOW_LOGICAL_WIDTH, SETTINGS_WINDOW_PAGE_PATH,
};

const SETTINGS_WINDOW_TITLE: &str = "Crest Settings";

/// Shows the settings window: brings it to the front if it's already open, otherwise
/// creates it. The one way in for every entry point (tray, launching Crest a second time,
/// later a settings button), so they all behave the same.
pub fn open_or_focus_settings_window<R: Runtime>(crest_app: &AppHandle<R>) {
    if let Some(open_settings_window) = crest_app.get_webview_window(SETTINGS_WINDOW_LABEL) {
        let focus_result = open_settings_window.unminimize().and_then(|()| open_settings_window.set_focus());
        if let Err(focus_error) = focus_result {
            eprintln!("Crest: could not bring the settings window to the front: {focus_error}");
        }
        return;
    }
    // Creating a webview from a menu or other event handler deadlocks on Windows (Tauri docs,
    // "Known issues" of WebviewWindowBuilder::new), so it's built on a thread of its own.
    let window_creating_app = crest_app.clone();
    thread::spawn(move || {
        let creation_result = WebviewWindowBuilder::new(
            &window_creating_app,
            SETTINGS_WINDOW_LABEL,
            WebviewUrl::App(SETTINGS_WINDOW_PAGE_PATH.into()),
        )
        .title(SETTINGS_WINDOW_TITLE)
        .inner_size(SETTINGS_WINDOW_LOGICAL_WIDTH, SETTINGS_WINDOW_LOGICAL_HEIGHT)
        .resizable(false)
        .maximizable(false)
        .center()
        .focused(true)
        .build();
        if let Err(creation_error) = creation_result {
            eprintln!("Crest: could not open the settings window: {creation_error}");
        }
    });
}
