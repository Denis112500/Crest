//! The trait each OS implements: watch the system volume, and set its level.

use crate::system_volume::system_volume_level::SystemVolumeLevel;
use crate::system_volume::system_volume_reading::SystemVolumeReading;

/// Called once when watching starts, then after every change of level, mute or output device.
pub type SystemVolumeListener = Box<dyn Fn(SystemVolumeReading) + Send + 'static>;

/// One operating system's way of reading and setting the system volume.
/// Windows: Core Audio (the default output device's endpoint volume). Linux, later: PipeWire.
/// `Sync` because Tauri keeps it for the "set level" command, which any thread may run.
pub trait SystemVolumePlatform: Send + Sync {
    /// Starts watching in the background and returns immediately.
    fn start_watching_system_volume(&mut self, system_volume_listener: SystemVolumeListener) -> Result<(), String>;

    /// Asks the OS to set the level (and to unmute, like Windows' own slider). Returns as soon as
    /// the request is queued.
    fn request_system_volume_level(&self, requested_level: SystemVolumeLevel) -> Result<(), String>;
}
