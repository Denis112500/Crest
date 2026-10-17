//! The system volume (all sound, like the speaker icon in the taskbar) for the music tile's
//! slider. The OS-independent rules live here; each OS brings its own implementation
//! (Windows: Core Audio).

mod system_volume_announcer;
pub mod system_volume_commands;
mod system_volume_level;
mod system_volume_platform;
mod system_volume_reading;
mod system_volume_startup;

#[cfg(target_os = "windows")]
mod windows_core_audio;

pub use system_volume_startup::start_watching_system_volume;

#[cfg(target_os = "windows")]
use windows_core_audio::CoreAudioSystemVolume as CurrentPlatformSystemVolume;
