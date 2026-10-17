//! Starts watching the system volume when Crest starts and gives Tauri what the two commands
//! need. Always on: it is local, and while nothing changes it only waits for the OS.

use std::sync::Arc;

use tauri::{App, Emitter, Manager};

use crate::backend_constants::PILL_WINDOW_LABEL;
use crate::ipc_channel_names::SYSTEM_VOLUME_CHANGED_EVENT;
use crate::system_volume::system_volume_announcer::SystemVolumeAnnouncer;
use crate::system_volume::system_volume_platform::SystemVolumePlatform;
use crate::system_volume::CurrentPlatformSystemVolume;

pub fn start_watching_system_volume(crest_app: &App) {
    let emit_app_handle = crest_app.handle().clone();
    let system_volume_announcer = Arc::new(SystemVolumeAnnouncer::new(Box::new(move |system_volume| {
        if let Err(emit_error) = emit_app_handle.emit_to(PILL_WINDOW_LABEL, SYSTEM_VOLUME_CHANGED_EVENT, system_volume) {
            eprintln!("Crest: could not send the system volume to the window: {emit_error}");
        }
    })));
    let announcer_for_listener = Arc::clone(&system_volume_announcer);
    let mut system_volume_platform = CurrentPlatformSystemVolume::default();
    // Not fatal: without it the page only gets "no output device" and greys the speaker out.
    if let Err(watch_error) = system_volume_platform.start_watching_system_volume(Box::new(move |system_volume_reading| {
        announcer_for_listener.handle_system_volume_reading(system_volume_reading);
    })) {
        eprintln!("Crest: could not start watching the system volume: {watch_error}");
    }
    crest_app.manage(system_volume_announcer);
    let boxed_system_volume_platform: Box<dyn SystemVolumePlatform> = Box::new(system_volume_platform);
    crest_app.manage(boxed_system_volume_platform);
}
