use crate::activity_core::{ActivityPublisher, ActivitySource, ActivityUpdate};
use crate::activity_sources::music::session_loss_grace_period::SessionLossGracePeriod;
use crate::backend_constants::{MUSIC_ACTIVITY_DISPLAY_PRIORITY, MUSIC_SESSION_LOSS_GRACE_PERIOD};
use crate::ipc_channel_names::MUSIC_ACTIVITY_KIND;
use crate::media::{MediaPlaybackState, MediaSessionSnapshot, MediaSource};

/// The music plugin: turns what the OS media source sees into pill activity.
/// It doesn't know which OS it runs on; that's the `MediaSource`'s job.
pub struct MusicActivitySource {
    media_source: Box<dyn MediaSource>,
}

impl MusicActivitySource {
    pub fn new(media_source: Box<dyn MediaSource>) -> Self {
        Self { media_source }
    }
}

impl ActivitySource for MusicActivitySource {
    fn activity_kind(&self) -> &'static str {
        MUSIC_ACTIVITY_KIND
    }

    fn start_publishing(&mut self, activity_publisher: ActivityPublisher) -> Result<(), String> {
        let session_loss_grace_period = SessionLossGracePeriod::new(MUSIC_SESSION_LOSS_GRACE_PERIOD);
        self.media_source.start_watching_media_sessions(Box::new(move |media_snapshot| {
            match media_snapshot {
                Some(media_snapshot) => session_loss_grace_period.report_session_present(|| {
                    if let Some(music_activity_update) = convert_to_music_activity_update(&media_snapshot) {
                        activity_publisher.publish_activity_update(music_activity_update);
                    }
                }),
                None => {
                    let publisher_for_lost_session = activity_publisher.clone();
                    session_loss_grace_period
                        .report_session_lost(move || publisher_for_lost_session.withdraw_activity());
                }
            }
        }))
    }
}

fn convert_to_music_activity_update(media_snapshot: &MediaSessionSnapshot) -> Option<ActivityUpdate> {
    let activity_payload = serde_json::to_value(media_snapshot)
        .inspect_err(|serialization_error| eprintln!("Crest music: could not serialize the track: {serialization_error}"))
        .ok()?;
    Some(ActivityUpdate {
        display_priority: MUSIC_ACTIVITY_DISPLAY_PRIORITY,
        is_ongoing: media_snapshot.playback_state == MediaPlaybackState::Playing,
        // A new title or artist means a new track, which makes the pill peek.
        attention_key: format!("{}\n{}", media_snapshot.track_title, media_snapshot.track_artist),
        activity_payload,
    })
}
