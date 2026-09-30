use crate::media::media_session_snapshot::MediaSessionSnapshot;

/// Called with the preferred media session every time something about it changes,
/// or with `None` when no allowed session exists.
pub type MediaSnapshotListener = Box<dyn Fn(Option<MediaSessionSnapshot>) + Send + 'static>;

/// One operating system's way of watching media players.
/// Windows: System Media Transport Controls (SMTC). Linux, later: MPRIS over D-Bus.
pub trait MediaSource {
    /// Starts watching in the background and returns immediately.
    fn start_watching_media_sessions(
        &mut self,
        media_snapshot_listener: MediaSnapshotListener,
    ) -> Result<(), String>;
}
