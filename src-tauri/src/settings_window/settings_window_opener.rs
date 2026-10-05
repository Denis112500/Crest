use std::thread;

use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

use crate::backend_constants::{
    SETTINGS_WINDOW_LABEL, SETTINGS_WINDOW_LOGICAL_HEIGHT, SETTINGS_WINDOW_LOGICAL_WIDTH, SETTINGS_WINDOW_PAGE_PATH,
    SETTINGS_WINDOW_TITLE_BAR_ALLOWANCE_LOGICAL_PIXELS,
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
    let settings_window_logical_height = fit_settings_window_height(read_main_display_work_area_logical_height(crest_app));
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
        .inner_size(SETTINGS_WINDOW_LOGICAL_WIDTH, settings_window_logical_height)
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

/// The main display's height without the taskbar, in logical pixels; `None` if unknown.
fn read_main_display_work_area_logical_height<R: Runtime>(crest_app: &AppHandle<R>) -> Option<f64> {
    let main_display = crest_app.primary_monitor().ok().flatten()?;
    Some(f64::from(main_display.work_area().size.height) / main_display.scale_factor())
}

/// The window can't be resized, so it must never be taller than the screen: on a 1080p laptop
/// at 150 % scaling the whole screen is only 720 logical pixels tall, and the bottom cards
/// would be out of reach. A shorter window scrolls its page instead.
fn fit_settings_window_height(work_area_logical_height: Option<f64>) -> f64 {
    match work_area_logical_height {
        Some(work_area_logical_height) => SETTINGS_WINDOW_LOGICAL_HEIGHT
            .min(work_area_logical_height - SETTINGS_WINDOW_TITLE_BAR_ALLOWANCE_LOGICAL_PIXELS),
        None => SETTINGS_WINDOW_LOGICAL_HEIGHT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_full_height_on_a_tall_screen() {
        assert_eq!(fit_settings_window_height(Some(1392.0)), SETTINGS_WINDOW_LOGICAL_HEIGHT);
        assert_eq!(fit_settings_window_height(None), SETTINGS_WINDOW_LOGICAL_HEIGHT);
    }

    #[test]
    fn shrinks_to_fit_a_1080p_laptop_at_150_percent() {
        // 1080 px minus a 48 px taskbar, at 150 % scaling: 688 logical pixels of work area.
        let fitted_height = fit_settings_window_height(Some(688.0));
        assert!(fitted_height + SETTINGS_WINDOW_TITLE_BAR_ALLOWANCE_LOGICAL_PIXELS <= 688.0);
    }
}
