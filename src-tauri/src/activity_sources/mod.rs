//! The activity plugins. Each one turns something happening on the PC into pill activity.

pub mod claude_code;
mod music;

pub use music::MusicActivitySource;
