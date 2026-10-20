//! Starts the pointer watch when Crest starts: the rest detector reads where the closed pill is,
//! and a rest on it goes to the pill page as an event. Always on: while the mouse is still it
//! only waits for the OS.

use tauri::{App, Emitter, Manager};

use crate::backend_constants::PILL_WINDOW_LABEL;
use crate::ipc_channel_names::PILL_POINTER_RESTED_EVENT;
use crate::pointer_watching::pointer_rest_detector::PointerRestDetector;
use crate::pointer_watching::pointer_watch::PointerWatch;
use crate::pointer_watching::shared_pill_pointer_zones::SharedPillPointerZones;
use crate::pointer_watching::CurrentPlatformPointerWatch;

/// Before the pill page can ask for its placement, which fills the zones in.
pub fn start_watching_pointer_rest(crest_app: &App) {
    let shared_pill_pointer_zones = SharedPillPointerZones::default();
    crest_app.manage(shared_pill_pointer_zones.clone());
    let zones_for_detector = shared_pill_pointer_zones.clone();
    let emit_app_handle = crest_app.handle().clone();
    let pointer_rest_detector = PointerRestDetector::new(
        Box::new(move || zones_for_detector.read_rest_zone()),
        Box::new(move |screen_x, screen_y| {
            let Some(rest_point) = shared_pill_pointer_zones.convert_to_window_logical_point(screen_x, screen_y) else {
                return;
            };
            if let Err(emit_error) = emit_app_handle.emit_to(PILL_WINDOW_LABEL, PILL_POINTER_RESTED_EVENT, rest_point) {
                eprintln!("Crest: could not tell the pill the pointer rests on it: {emit_error}");
            }
        }),
    );
    // Not fatal: without it the closed pill only opens for news, not for the pointer.
    if let Err(watch_error) = CurrentPlatformPointerWatch::start_watching_pointer(Box::new(pointer_rest_detector)) {
        eprintln!("Crest: could not start watching the pointer: {watch_error}");
    }
}
