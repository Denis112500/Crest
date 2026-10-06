use crate::activity_core::{ActivityPresence, ActivityPublisher, ActivitySource, ActivityUpdate};
use crate::activity_sources::music::session_loss_grace_period::SessionLossGracePeriod;
use crate::backend_constants::{MUSIC_ACTIVITY_DISPLAY_PRIORITY, MUSIC_SESSION_LOSS_GRACE_PERIOD};
use crate::ipc_channel_names::{
    MUSIC_ACTIVITY_KIND, MUSIC_NEXT_TRACK_ACTION, MUSIC_PREVIOUS_TRACK_ACTION, MUSIC_TOGGLE_PLAY_PAUSE_ACTION,
};
use crate::media::{MediaPlaybackState, MediaSessionSnapshot, MediaSource, MediaTransportCommand};

/// The music plugin: turns what the OS media source sees into pill activity, and the
/// pill's buttons into player commands. It doesn't know which OS it runs on; that's the
/// `MediaSource`'s job.
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

    fn perform_activity_action(&self, activity_action: &str) -> Result<(), String> {
        let media_transport_command = media_transport_command_for_action(activity_action)
            .ok_or_else(|| format!("the music activity has no action \"{activity_action}\""))?;
        self.media_source.send_media_transport_command(media_transport_command)
    }
}

fn media_transport_command_for_action(activity_action: &str) -> Option<MediaTransportCommand> {
    match activity_action {
        MUSIC_TOGGLE_PLAY_PAUSE_ACTION => Some(MediaTransportCommand::TogglePlayPause),
        MUSIC_NEXT_TRACK_ACTION => Some(MediaTransportCommand::NextTrack),
        MUSIC_PREVIOUS_TRACK_ACTION => Some(MediaTransportCommand::PreviousTrack),
        _ => None,
    }
}

fn convert_to_music_activity_update(media_snapshot: &MediaSessionSnapshot) -> Option<ActivityUpdate> {
    let activity_payload = serde_json::to_value(media_snapshot)
        .inspect_err(|serialization_error| eprintln!("Crest music: could not serialize the track: {serialization_error}"))
        .ok()?;
    Some(ActivityUpdate {
        display_priority: MUSIC_ACTIVITY_DISPLAY_PRIORITY,
        activity_presence: if media_snapshot.playback_state == MediaPlaybackState::Playing {
            ActivityPresence::Ongoing
        } else {
            ActivityPresence::Lingering
        },
        // A new title or artist means a new track, which makes the pill peek.
        attention_key: format!("{}\n{}", media_snapshot.track_title, media_snapshot.track_artist),
        activity_payload,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_every_button_action_to_a_player_command() {
        assert_eq!(
            media_transport_command_for_action(MUSIC_TOGGLE_PLAY_PAUSE_ACTION),
            Some(MediaTransportCommand::TogglePlayPause)
        );
        assert_eq!(media_transport_command_for_action(MUSIC_NEXT_TRACK_ACTION), Some(MediaTransportCommand::NextTrack));
        assert_eq!(
            media_transport_command_for_action(MUSIC_PREVIOUS_TRACK_ACTION),
            Some(MediaTransportCommand::PreviousTrack)
        );
        assert_eq!(media_transport_command_for_action("dance"), None);
    }
}
