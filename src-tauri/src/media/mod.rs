mod media_session_selector;
mod media_session_snapshot;
mod media_source;

#[cfg(target_os = "windows")]
mod windows_smtc;

pub use media_session_snapshot::{MediaPlaybackState, MediaSessionSnapshot};
pub use media_source::MediaSource;

#[cfg(target_os = "windows")]
pub use windows_smtc::SmtcMediaSource as CurrentPlatformMediaSource;
