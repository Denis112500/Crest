//! Holds a new track's album art back for a moment and shows "loading": some players first
//! send a placeholder (Brave: its logo) and the real cover a moment later.

use std::time::Instant;

use crate::backend_constants::ALBUM_ART_SETTLE_WINDOW;
use crate::media::media_session_snapshot::MediaSessionSnapshot;

/// App, title, artist and album: what makes a snapshot "a different track".
type TrackIdentity = (String, String, String, String);

/// Some players publish a placeholder thumbnail when a track starts and the real cover a
/// moment later (Brave: its own logo, then the cover). So a new track's art is held back
/// for a short settle window and the pill shows "loading" instead. This doesn't look for
/// any particular logo, so it keeps working when the placeholder changes.
#[derive(Default)]
pub struct AlbumArtSettleGate {
    current_track_identity: Option<TrackIdentity>,
    settle_deadline: Option<Instant>,
}

impl AlbumArtSettleGate {
    pub fn hold_back_unsettled_album_art(&mut self, media_session_snapshot: &mut MediaSessionSnapshot, now: Instant) {
        let track_identity = (
            media_session_snapshot.source_app_identifier.clone(),
            media_session_snapshot.track_title.clone(),
            media_session_snapshot.track_artist.clone(),
            media_session_snapshot.album_title.clone(),
        );
        if self.current_track_identity.as_ref() != Some(&track_identity) {
            self.current_track_identity = Some(track_identity);
            self.settle_deadline = Some(now + ALBUM_ART_SETTLE_WINDOW);
        }
        match self.settle_deadline {
            Some(settle_deadline) if now < settle_deadline => {
                media_session_snapshot.album_art_data_url = None;
                media_session_snapshot.is_album_art_loading = true;
            }
            _ => self.settle_deadline = None,
        }
    }

    /// When the art of the current track may be shown; the caller must publish again then,
    /// because the player may send nothing new at that moment.
    pub fn pending_settle_deadline(&self) -> Option<Instant> {
        self.settle_deadline
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::media::media_session_snapshot::{MediaControlAvailability, MediaPlaybackState};

    fn snapshot_of_track(track_title: &str, album_art_data_url: &str) -> MediaSessionSnapshot {
        MediaSessionSnapshot {
            source_app_identifier: "player".to_string(),
            track_title: track_title.to_string(),
            track_artist: "artist".to_string(),
            album_title: String::new(),
            album_art_data_url: Some(album_art_data_url.to_string()),
            is_album_art_loading: false,
            playback_state: MediaPlaybackState::Playing,
            timeline: None,
            available_controls: MediaControlAvailability {
                can_toggle_play_pause: true,
                can_skip_to_next_track: true,
                can_skip_to_previous_track: true,
            },
        }
    }

    #[test]
    fn a_new_track_shows_loading_instead_of_its_first_thumbnail() {
        let mut album_art_settle_gate = AlbumArtSettleGate::default();
        let mut placeholder_snapshot = snapshot_of_track("first", "data:placeholder");
        album_art_settle_gate.hold_back_unsettled_album_art(&mut placeholder_snapshot, Instant::now());
        assert_eq!(placeholder_snapshot.album_art_data_url, None);
        assert!(placeholder_snapshot.is_album_art_loading);
    }

    #[test]
    fn the_art_appears_once_the_settle_window_has_passed() {
        let mut album_art_settle_gate = AlbumArtSettleGate::default();
        let track_started_at = Instant::now();
        album_art_settle_gate
            .hold_back_unsettled_album_art(&mut snapshot_of_track("first", "data:placeholder"), track_started_at);
        let mut cover_arrived_snapshot = snapshot_of_track("first", "data:cover");
        album_art_settle_gate.hold_back_unsettled_album_art(
            &mut cover_arrived_snapshot,
            track_started_at + Duration::from_millis(100),
        );
        assert!(cover_arrived_snapshot.is_album_art_loading);

        let mut settled_snapshot = snapshot_of_track("first", "data:cover");
        album_art_settle_gate
            .hold_back_unsettled_album_art(&mut settled_snapshot, track_started_at + ALBUM_ART_SETTLE_WINDOW);
        assert_eq!(settled_snapshot.album_art_data_url.as_deref(), Some("data:cover"));
        assert!(!settled_snapshot.is_album_art_loading);
        assert_eq!(album_art_settle_gate.pending_settle_deadline(), None);
    }

    #[test]
    fn every_track_change_starts_a_new_settle_window() {
        let mut album_art_settle_gate = AlbumArtSettleGate::default();
        let first_track_started_at = Instant::now();
        album_art_settle_gate
            .hold_back_unsettled_album_art(&mut snapshot_of_track("first", "data:a"), first_track_started_at);
        let second_track_started_at = first_track_started_at + Duration::from_secs(5);
        let mut second_track_snapshot = snapshot_of_track("second", "data:placeholder");
        album_art_settle_gate.hold_back_unsettled_album_art(&mut second_track_snapshot, second_track_started_at);
        assert!(second_track_snapshot.is_album_art_loading);
        assert_eq!(
            album_art_settle_gate.pending_settle_deadline(),
            Some(second_track_started_at + ALBUM_ART_SETTLE_WINDOW)
        );
    }
}
