use windows::Foundation::{DateTime, TimeSpan};
use windows::Media::Control::{
    GlobalSystemMediaTransportControlsSession as SmtcSession,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as SmtcPlaybackStatus,
};

use crate::backend_constants::{WINRT_TICKS_FROM_1601_TO_UNIX_EPOCH, WINRT_TICKS_PER_MILLISECOND};
use crate::media::media_session_snapshot::{
    MediaControlAvailability, MediaPlaybackState, MediaSessionSnapshot, MediaTimeline,
};
use crate::media::windows_smtc::smtc_thumbnail_reader::read_thumbnail_as_data_url;

/// Must be called from the SMTC worker thread: `.join()` blocks until Windows answers.
pub fn read_media_session_snapshot(
    session: &SmtcSession,
    source_app_identifier: &str,
) -> windows::core::Result<MediaSessionSnapshot> {
    let media_properties = session.TryGetMediaPropertiesAsync()?.join()?;
    Ok(MediaSessionSnapshot {
        source_app_identifier: source_app_identifier.to_string(),
        track_title: media_properties.Title()?.to_string_lossy(),
        track_artist: media_properties.Artist()?.to_string_lossy(),
        album_title: media_properties.AlbumTitle()?.to_string_lossy(),
        album_art_data_url: media_properties
            .Thumbnail()
            .ok()
            .and_then(|thumbnail_reference| read_thumbnail_as_data_url(&thumbnail_reference)),
        is_album_art_loading: false,
        playback_state: read_playback_state(session),
        timeline: read_media_timeline(session),
        available_controls: read_control_availability(session),
    })
}

/// A session that can't report its status is treated as stopped rather than failing the read.
pub fn read_playback_state(session: &SmtcSession) -> MediaPlaybackState {
    let playback_status = session.GetPlaybackInfo().and_then(|playback_info| playback_info.PlaybackStatus());
    match playback_status {
        Ok(SmtcPlaybackStatus::Playing) => MediaPlaybackState::Playing,
        Ok(SmtcPlaybackStatus::Paused) => MediaPlaybackState::Paused,
        Ok(SmtcPlaybackStatus::Changing) => MediaPlaybackState::Changing,
        _ => MediaPlaybackState::Stopped,
    }
}

/// The player's permissions arrive with every `PlaybackInfoChanged` event. If they can't be
/// read, all buttons stay usable: a press the player doesn't accept is simply declined.
fn read_control_availability(session: &SmtcSession) -> MediaControlAvailability {
    let all_controls_available = MediaControlAvailability {
        can_toggle_play_pause: true,
        can_skip_to_next_track: true,
        can_skip_to_previous_track: true,
    };
    let Ok(playback_controls) = session.GetPlaybackInfo().and_then(|playback_info| playback_info.Controls()) else {
        return all_controls_available;
    };
    MediaControlAvailability {
        can_toggle_play_pause: playback_controls.IsPlayPauseToggleEnabled().unwrap_or(true),
        can_skip_to_next_track: playback_controls.IsNextEnabled().unwrap_or(true),
        can_skip_to_previous_track: playback_controls.IsPreviousEnabled().unwrap_or(true),
    }
}

fn read_media_timeline(session: &SmtcSession) -> Option<MediaTimeline> {
    let timeline_properties = session.GetTimelineProperties().ok()?;
    let track_duration_milliseconds = winrt_time_span_to_milliseconds(timeline_properties.EndTime().ok()?)
        - winrt_time_span_to_milliseconds(timeline_properties.StartTime().ok()?);
    // Live streams and players without duration report zero.
    if track_duration_milliseconds <= 0 {
        return None;
    }
    Some(MediaTimeline {
        track_duration_milliseconds,
        reported_position_milliseconds: winrt_time_span_to_milliseconds(timeline_properties.Position().ok()?),
        position_reported_at_unix_milliseconds: winrt_date_time_to_unix_milliseconds(
            timeline_properties.LastUpdatedTime().ok()?,
        ),
    })
}

fn winrt_time_span_to_milliseconds(time_span: TimeSpan) -> i64 {
    time_span.Duration / WINRT_TICKS_PER_MILLISECOND
}

fn winrt_date_time_to_unix_milliseconds(date_time: DateTime) -> i64 {
    (date_time.UniversalTime - WINRT_TICKS_FROM_1601_TO_UNIX_EPOCH) / WINRT_TICKS_PER_MILLISECOND
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_winrt_ticks_to_milliseconds() {
        assert_eq!(winrt_time_span_to_milliseconds(TimeSpan { Duration: 1_792_348_290 }), 179_234);
    }

    #[test]
    fn converts_winrt_date_to_unix_time() {
        // 2026-09-30T00:00:00Z is 1790726400000 ms after the Unix epoch.
        let winrt_date = DateTime { UniversalTime: 1_790_726_400_000 * 10_000 + 116_444_736_000_000_000 };
        assert_eq!(winrt_date_time_to_unix_milliseconds(winrt_date), 1_790_726_400_000);
    }
}
