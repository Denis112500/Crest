mod album_art_settle_gate;
mod media_app_identifier_label;
mod media_player_filter;
mod media_player_filter_control;
mod media_session_selector;
mod media_session_snapshot;
mod media_source;
mod pending_transport_commands;

#[cfg(target_os = "windows")]
mod windows_smtc;

pub use media_app_identifier_label::describe_media_app_identifier;
pub use media_player_filter_control::MediaPlayerFilterControl;
pub use media_player_filter::{is_app_identifier_in_list, MediaPlayerFilter};
pub use media_session_snapshot::{MediaPlaybackState, MediaSessionSnapshot};
pub use media_source::{MediaSource, MediaTransportCommand};

#[cfg(target_os = "windows")]
pub use windows_smtc::SmtcMediaSource as CurrentPlatformMediaSource;
