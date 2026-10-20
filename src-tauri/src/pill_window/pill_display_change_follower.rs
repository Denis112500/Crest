//! Places the pill again whenever the monitors change (a resolution switch after login, a
//! monitor plugged in or unplugged): the page already does this on `pill-display-changed`.

use tauri::{App, Emitter, WebviewWindow};

use crate::backend_constants::PILL_WINDOW_LABEL;
use crate::ipc_channel_names::PILL_DISPLAY_CHANGED_EVENT;
use crate::pill_window::pill_window_platform::PillWindowPlatform;
use crate::pill_window::CurrentPlatformPillWindow;

pub fn follow_display_changes(crest_app: &App, pill_window: &WebviewWindow) {
    let emit_app_handle = crest_app.handle().clone();
    // Not fatal: without it the pill keeps its place until Crest restarts, as before.
    if let Err(watch_error) = CurrentPlatformPillWindow::watch_display_changes(
        pill_window,
        Box::new(move || {
            if let Err(emit_error) = emit_app_handle.emit_to(PILL_WINDOW_LABEL, PILL_DISPLAY_CHANGED_EVENT, ()) {
                eprintln!("Crest: could not tell the pill the monitors changed: {emit_error}");
            }
        }),
    ) {
        eprintln!("Crest: could not watch for monitor changes: {watch_error}");
    }
}
