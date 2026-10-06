//! The music plugin: what's playing in any media player, through the OS media source.

mod music_activity_source;
mod session_loss_grace_period;

pub use music_activity_source::MusicActivitySource;
