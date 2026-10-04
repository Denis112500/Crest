use std::time::Instant;

use crate::media::media_player_filter::MediaPlayerFilter;

/// What the selector needs to know about one media session, whatever the OS.
pub struct MediaSessionCandidate<'candidate> {
    pub source_app_identifier: &'candidate str,
    pub is_playing: bool,
    /// When this session last reported a change; `None` if it hasn't since Crest started.
    pub last_activity: Option<Instant>,
}

/// Picks the session the pill should show: only apps allowed by the filter, a playing
/// session over a paused one, and among equals the most recently active.
/// Returns the index into `candidates`.
pub fn select_preferred_media_session(
    candidates: &[MediaSessionCandidate],
    media_player_filter: &MediaPlayerFilter,
) -> Option<usize> {
    candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| media_player_filter.allows_app_identifier(candidate.source_app_identifier))
        .max_by_key(|(_, candidate)| (candidate.is_playing, candidate.last_activity))
        .map(|(candidate_index, _)| candidate_index)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    const YOUTUBE_MUSIC_IN_BRAVE: &str = "Brave._crx_cinhimbnkkghhklpknlkffjgod";
    const SPOTIFY: &str = "Spotify.exe";

    fn youtube_music_filter() -> MediaPlayerFilter {
        MediaPlayerFilter::OnlyListedPlayers(vec!["_CRX_cinhimbnkkghhklpknlkffjgod".to_string()])
    }

    #[test]
    fn ignores_sessions_from_apps_outside_the_filter() {
        let candidates = [
            MediaSessionCandidate { source_app_identifier: SPOTIFY, is_playing: true, last_activity: None },
            MediaSessionCandidate { source_app_identifier: YOUTUBE_MUSIC_IN_BRAVE, is_playing: false, last_activity: None },
        ];
        assert_eq!(select_preferred_media_session(&candidates, &youtube_music_filter()), Some(1));
    }

    #[test]
    fn returns_none_when_no_session_is_allowed() {
        let candidates = [MediaSessionCandidate { source_app_identifier: SPOTIFY, is_playing: true, last_activity: None }];
        assert_eq!(select_preferred_media_session(&candidates, &youtube_music_filter()), None);
        assert_eq!(select_preferred_media_session(&[], &youtube_music_filter()), None);
    }

    #[test]
    fn prefers_playing_over_more_recently_active_paused() {
        let earlier = Instant::now();
        let later = earlier + Duration::from_secs(5);
        let candidates = [
            MediaSessionCandidate { source_app_identifier: SPOTIFY, is_playing: false, last_activity: Some(later) },
            MediaSessionCandidate { source_app_identifier: YOUTUBE_MUSIC_IN_BRAVE, is_playing: true, last_activity: Some(earlier) },
        ];
        assert_eq!(select_preferred_media_session(&candidates, &MediaPlayerFilter::EveryPlayer), Some(1));
    }

    #[test]
    fn prefers_most_recently_active_among_paused_sessions() {
        let earlier = Instant::now();
        let later = earlier + Duration::from_secs(5);
        let candidates = [
            MediaSessionCandidate { source_app_identifier: YOUTUBE_MUSIC_IN_BRAVE, is_playing: false, last_activity: Some(later) },
            MediaSessionCandidate { source_app_identifier: SPOTIFY, is_playing: false, last_activity: Some(earlier) },
        ];
        assert_eq!(select_preferred_media_session(&candidates, &MediaPlayerFilter::EveryPlayer), Some(0));
    }
}
