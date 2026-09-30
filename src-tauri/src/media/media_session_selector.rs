use std::time::Instant;

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
    allowed_app_identifier_fragments: &[String],
) -> Option<usize> {
    candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| {
            is_app_identifier_allowed(candidate.source_app_identifier, allowed_app_identifier_fragments)
        })
        .max_by_key(|(_, candidate)| (candidate.is_playing, candidate.last_activity))
        .map(|(candidate_index, _)| candidate_index)
}

/// An empty filter allows every app; otherwise the app ID must contain one of the
/// fragments, ignoring case (browsers aren't consistent: "Brave", "chrome.exe", "MSEdge").
fn is_app_identifier_allowed(source_app_identifier: &str, allowed_app_identifier_fragments: &[String]) -> bool {
    let lowercase_app_identifier = source_app_identifier.to_lowercase();
    allowed_app_identifier_fragments.is_empty()
        || allowed_app_identifier_fragments
            .iter()
            .any(|allowed_fragment| lowercase_app_identifier.contains(&allowed_fragment.to_lowercase()))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    const YOUTUBE_MUSIC_IN_BRAVE: &str = "Brave._crx_cinhimbnkkghhklpknlkffjgod";
    const SPOTIFY: &str = "Spotify.exe";

    fn youtube_music_filter() -> Vec<String> {
        vec!["_CRX_cinhimbnkkghhklpknlkffjgod".to_string()]
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
        assert_eq!(select_preferred_media_session(&candidates, &[]), Some(1));
    }

    #[test]
    fn prefers_most_recently_active_among_paused_sessions() {
        let earlier = Instant::now();
        let later = earlier + Duration::from_secs(5);
        let candidates = [
            MediaSessionCandidate { source_app_identifier: YOUTUBE_MUSIC_IN_BRAVE, is_playing: false, last_activity: Some(later) },
            MediaSessionCandidate { source_app_identifier: SPOTIFY, is_playing: false, last_activity: Some(earlier) },
        ];
        assert_eq!(select_preferred_media_session(&candidates, &[]), Some(0));
    }
}
