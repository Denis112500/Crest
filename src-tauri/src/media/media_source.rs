use crate::media::media_player_filter_control::MediaPlayerFilterControl;
use crate::media::media_session_snapshot::MediaSessionSnapshot;

/// Called with the preferred media session every time something about it changes,
/// or with `None` when no allowed session exists.
pub type MediaSnapshotListener = Box<dyn Fn(Option<MediaSessionSnapshot>) + Send + 'static>;

/// What the pill's buttons can ask the player to do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MediaTransportCommand {
    TogglePlayPause,
    NextTrack,
    PreviousTrack,
}

/// One operating system's way of watching and controlling media players.
/// Windows: System Media Transport Controls (SMTC). Linux, later: MPRIS over D-Bus.
/// `Send` because the app keeps it where commands from any thread can reach it.
pub trait MediaSource: Send {
    /// Starts watching in the background and returns immediately.
    fn start_watching_media_sessions(
        &mut self,
        media_snapshot_listener: MediaSnapshotListener,
    ) -> Result<(), String>;

    /// Asks the player of the session currently shown to act. Returns as soon as the
    /// request is queued; the effect arrives later as an ordinary snapshot update.
    fn send_media_transport_command(&self, media_transport_command: MediaTransportCommand) -> Result<(), String>;

    /// A handle for the settings window, created before the source is handed to its
    /// activity: lists the open players and changes the filter while the source runs.
    fn create_player_filter_control(&self) -> MediaPlayerFilterControl;
}
