//! The settings window's way into the running media source: list open players, change the
//! filter. OS-independent, so the settings code never sees Windows details.

use std::sync::{Arc, Mutex, PoisonError};

use crate::media::media_player_filter::MediaPlayerFilter;

/// Lets the settings window see which players are open and change which ones the pill
/// shows, while the media source keeps running. Each platform's media source creates one;
/// nothing here knows which OS it is.
pub struct MediaPlayerFilterControl {
    /// Written by the media source whenever the OS reports its list of players changed.
    open_player_app_identifiers: Arc<Mutex<Vec<String>>>,
    media_player_filter_sender: Box<dyn Fn(MediaPlayerFilter) -> Result<(), String> + Send + Sync>,
}

impl MediaPlayerFilterControl {
    pub fn new(
        open_player_app_identifiers: Arc<Mutex<Vec<String>>>,
        media_player_filter_sender: impl Fn(MediaPlayerFilter) -> Result<(), String> + Send + Sync + 'static,
    ) -> Self {
        Self { open_player_app_identifiers, media_player_filter_sender: Box::new(media_player_filter_sender) }
    }

    pub fn list_open_player_app_identifiers(&self) -> Vec<String> {
        self.open_player_app_identifiers.lock().unwrap_or_else(PoisonError::into_inner).clone()
    }

    /// Takes effect right away: the media source picks the shown session again with the new
    /// filter.
    pub fn replace_media_player_filter(&self, media_player_filter: MediaPlayerFilter) -> Result<(), String> {
        (self.media_player_filter_sender)(media_player_filter)
    }
}
