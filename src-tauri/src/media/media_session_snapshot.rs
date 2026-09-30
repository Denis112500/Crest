use serde::Serialize;

/// Everything the pill needs to know about the media that is playing, independent of the OS.
/// Sent to the frontend as JSON (field names in camelCase, like TypeScript).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSessionSnapshot {
    pub source_app_identifier: String,
    pub track_title: String,
    pub track_artist: String,
    pub album_title: String,
    /// A `data:image/...;base64,...` URL, so the webview can show it without file or network access.
    pub album_art_data_url: Option<String>,
    pub playback_state: MediaPlaybackState,
    /// `None` when the player doesn't report a track duration.
    pub timeline: Option<MediaTimeline>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaPlaybackState {
    Playing,
    Paused,
    /// Between tracks, or while the player is buffering.
    Changing,
    Stopped,
}

/// Players don't push the position continuously (verified with YouTube Music: only on
/// play, pause, seek and track change), so the current position is extrapolated from
/// the last reported position and the moment it was reported.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaTimeline {
    pub track_duration_milliseconds: i64,
    pub reported_position_milliseconds: i64,
    pub position_reported_at_unix_milliseconds: i64,
}
