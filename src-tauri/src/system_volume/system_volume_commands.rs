//! The pill page's two system-volume commands (only the pill's capability grants them).

use std::sync::Arc;

use tauri::State;

use crate::system_volume::system_volume_announcer::SystemVolumeAnnouncer;
use crate::system_volume::system_volume_level::SystemVolumeLevel;
use crate::system_volume::system_volume_platform::SystemVolumePlatform;
use crate::system_volume::system_volume_reading::SystemVolumeState;

/// Events sent before the page listens are lost, so the page asks once at startup.
/// `None` = no output device (or not read yet; the first reading follows as an event).
#[tauri::command]
pub fn get_current_system_volume(
    system_volume_announcer: State<'_, Arc<SystemVolumeAnnouncer>>,
) -> Option<SystemVolumeState> {
    system_volume_announcer.current_system_volume()
}

/// `level` is 0..1; the result arrives on the page through the slider it moved, not as an event.
#[tauri::command]
pub fn set_system_volume_level(
    level: f64,
    system_volume_platform: State<'_, Box<dyn SystemVolumePlatform>>,
) -> Result<(), String> {
    system_volume_platform.request_system_volume_level(SystemVolumeLevel::from_requested_scalar(level)?)
}
