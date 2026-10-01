use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::time::Instant;

use windows::Media::Control::GlobalSystemMediaTransportControlsSessionManager as SmtcSessionManager;

use crate::media::album_art_settle_gate::AlbumArtSettleGate;
use crate::media::media_session_selector::{select_preferred_media_session, MediaSessionCandidate};
use crate::media::media_session_snapshot::{MediaPlaybackState, MediaSessionSnapshot};
use crate::media::media_source::{MediaSnapshotListener, MediaTransportCommand};
use crate::media::pending_transport_commands::PendingTransportCommands;
use crate::media::windows_smtc::smtc_event_subscriptions::SmtcSessionEventSubscription;
use crate::media::windows_smtc::smtc_snapshot_reader::{read_media_session_snapshot, read_playback_state};
use crate::media::windows_smtc::smtc_tracked_session::{find_session_of_app, TrackedSmtcSession};
use crate::media::windows_smtc::smtc_transport_commands::send_smtc_transport_command;
use crate::media::windows_smtc::smtc_worker_message::SmtcWorkerMessage;

/// Owns everything the SMTC worker thread knows: which sessions exist, when each was last
/// active, and what was last sent to the listener (so unchanged snapshots aren't resent).
pub struct SmtcSessionTracker {
    session_manager: SmtcSessionManager,
    worker_message_sender: Sender<SmtcWorkerMessage>,
    allowed_app_identifier_fragments: Vec<String>,
    tracked_sessions: Vec<TrackedSmtcSession>,
    last_activity_by_source_app: HashMap<String, Instant>,
    album_art_settle_gate: AlbumArtSettleGate,
    last_published_snapshot: Option<Option<MediaSessionSnapshot>>,
    /// The app the buttons control: the one shown last. Kept while its session is briefly
    /// gone between tracks, so a press in that gap can still be delivered to it.
    button_target_source_app_identifier: Option<String>,
    pending_transport_commands: PendingTransportCommands,
    media_snapshot_listener: MediaSnapshotListener,
}

impl SmtcSessionTracker {
    pub fn new(
        session_manager: SmtcSessionManager,
        worker_message_sender: Sender<SmtcWorkerMessage>,
        allowed_app_identifier_fragments: Vec<String>,
        media_snapshot_listener: MediaSnapshotListener,
    ) -> Self {
        Self {
            session_manager,
            worker_message_sender,
            allowed_app_identifier_fragments,
            tracked_sessions: Vec::new(),
            last_activity_by_source_app: HashMap::new(),
            album_art_settle_gate: AlbumArtSettleGate::default(),
            last_published_snapshot: None,
            button_target_source_app_identifier: None,
            pending_transport_commands: PendingTransportCommands::default(),
            media_snapshot_listener,
        }
    }

    /// Re-subscribes to every current session. Replacing the list drops the old
    /// subscriptions, which unsubscribes from sessions that have closed.
    pub fn refresh_session_list(&mut self) {
        let current_sessions = match self.session_manager.GetSessions() {
            Ok(current_sessions) => current_sessions,
            Err(read_error) => {
                eprintln!("Crest media: could not list media sessions: {read_error}");
                return;
            }
        };
        self.tracked_sessions = current_sessions
            .into_iter()
            .filter_map(|session| {
                let source_app_identifier = session.SourceAppUserModelId().ok()?.to_string_lossy();
                let event_subscription =
                    SmtcSessionEventSubscription::subscribe(session, &self.worker_message_sender).ok()?;
                Some(TrackedSmtcSession { source_app_identifier, event_subscription })
            })
            .collect();

        // A session that closed and comes back (browsers do this on every track change)
        // counts as freshly active.
        let tracked_sessions = &self.tracked_sessions;
        self.last_activity_by_source_app.retain(|source_app_identifier, _| {
            tracked_sessions.iter().any(|tracked| &tracked.source_app_identifier == source_app_identifier)
        });
        for tracked_session in &self.tracked_sessions {
            self.last_activity_by_source_app
                .entry(tracked_session.source_app_identifier.clone())
                .or_insert_with(Instant::now);
        }
        let button_target_session =
            find_session_of_app(&self.tracked_sessions, self.button_target_source_app_identifier.as_deref());
        if let Some(button_target_session) = button_target_session {
            for pending_command in self.pending_transport_commands.take_still_relevant(Instant::now()) {
                send_smtc_transport_command(button_target_session, pending_command);
            }
        }
    }

    pub fn record_session_activity(&mut self, source_app_identifier: String) {
        self.last_activity_by_source_app.insert(source_app_identifier, Instant::now());
    }

    pub fn publish_preferred_session_if_changed(&mut self) {
        let mut preferred_snapshot = self.read_preferred_session_snapshot();
        if let Some(snapshot) = preferred_snapshot.as_mut() {
            self.album_art_settle_gate.hold_back_unsettled_album_art(snapshot, Instant::now());
        }
        if let Some(snapshot) = preferred_snapshot.as_ref() {
            self.button_target_source_app_identifier = Some(snapshot.source_app_identifier.clone());
        }
        if self.last_published_snapshot.as_ref() != Some(&preferred_snapshot) {
            (self.media_snapshot_listener)(preferred_snapshot.clone());
            self.last_published_snapshot = Some(preferred_snapshot);
        }
    }

    pub fn pending_album_art_settle_deadline(&self) -> Option<Instant> {
        self.album_art_settle_gate.pending_settle_deadline()
    }

    pub fn send_transport_command_to_shown_session(&mut self, media_transport_command: MediaTransportCommand) {
        match find_session_of_app(&self.tracked_sessions, self.button_target_source_app_identifier.as_deref()) {
            Some(button_target_session) => send_smtc_transport_command(button_target_session, media_transport_command),
            None => self.pending_transport_commands.hold(media_transport_command, Instant::now()),
        }
    }

    fn read_preferred_session_snapshot(&self) -> Option<MediaSessionSnapshot> {
        let candidates: Vec<MediaSessionCandidate> = self
            .tracked_sessions
            .iter()
            .map(|tracked_session| MediaSessionCandidate {
                source_app_identifier: &tracked_session.source_app_identifier,
                is_playing: read_playback_state(tracked_session.event_subscription.session())
                    == MediaPlaybackState::Playing,
                last_activity: self.last_activity_by_source_app.get(&tracked_session.source_app_identifier).copied(),
            })
            .collect();
        let preferred_session_index =
            select_preferred_media_session(&candidates, &self.allowed_app_identifier_fragments)?;
        let preferred_session = &self.tracked_sessions[preferred_session_index];
        read_media_session_snapshot(
            preferred_session.event_subscription.session(),
            &preferred_session.source_app_identifier,
        )
        .inspect_err(|read_error| eprintln!("Crest media: could not read the media session: {read_error}"))
        .ok()
    }
}
