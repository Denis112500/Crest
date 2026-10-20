use crate::system_volume::system_volume_level::SystemVolumeLevel;
use crate::system_volume::system_volume_reading::SystemVolumeState;

/// Everything that wakes the Core Audio worker thread. Core Audio's callbacks (on Windows'
/// own threads) and the "set level" command (on Tauri's thread) only send one of these; every
/// Core Audio object is used on the worker thread alone.
pub enum CoreAudioWorkerMessage {
    VolumeNotified {
        /// Which opened output device sent it, so a late notification from the previous
        /// device can be told apart.
        endpoint_generation: u64,
        reported_system_volume: SystemVolumeState,
        is_crest_own_change: bool,
    },
    DefaultOutputDeviceChanged,
    LevelRequested(SystemVolumeLevel),
}
