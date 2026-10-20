//! What the OS reports about the system volume, and the part of it the page shows.

use serde::Serialize;

use crate::system_volume::system_volume_level::SystemVolumeLevel;

/// The default output device's volume, as the page shows it (`{ level, isMuted }`).
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemVolumeState {
    pub level: SystemVolumeLevel,
    pub is_muted: bool,
}

/// One report from the OS.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SystemVolumeReading {
    /// `None` when there is no output device at all.
    pub system_volume: Option<SystemVolumeState>,
    /// The change came from Crest's own slider (recognised by the OS's event context).
    pub is_crest_own_change: bool,
}
