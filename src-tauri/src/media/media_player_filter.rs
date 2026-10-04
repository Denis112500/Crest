/// Which players the pill may show. "Every player" is its own case instead of an empty list,
/// so removing the last player from the list means "none", as the settings window says.
#[derive(Clone, Debug, PartialEq)]
pub enum MediaPlayerFilter {
    EveryPlayer,
    /// Only the listed players (see `is_app_identifier_in_list`). Empty: none.
    OnlyListedPlayers(Vec<String>),
}

impl MediaPlayerFilter {
    pub fn allows_app_identifier(&self, app_identifier: &str) -> bool {
        match self {
            Self::EveryPlayer => true,
            Self::OnlyListedPlayers(listed_app_identifiers) => is_app_identifier_in_list(app_identifier, listed_app_identifiers),
        }
    }
}

/// Chromium browsers name an installed web app "<Browser>._crx_<app id>"; an entry that starts
/// at `_crx_` is that app in any browser.
const BROWSER_INDEPENDENT_WEB_APP_PREFIX: &str = "_crx_";

/// A list entry matches its exact app ID (ignoring case, browsers aren't consistent). Not
/// "contains": the Brave tab player is "Brave", and "contains" would also match every Brave web
/// app ("Brave._crx_…"), measured 2026-10-04. The one exception is a browser-independent web-app
/// entry ("_crx_<app id>"), which matches that app whichever browser runs it.
pub fn is_app_identifier_in_list(app_identifier: &str, listed_app_identifiers: &[String]) -> bool {
    let lowercase_app_identifier = app_identifier.to_lowercase();
    listed_app_identifiers.iter().any(|listed_app_identifier| {
        let lowercase_listed_app_identifier = listed_app_identifier.to_lowercase();
        if lowercase_listed_app_identifier.starts_with(BROWSER_INDEPENDENT_WEB_APP_PREFIX) {
            lowercase_app_identifier.ends_with(&lowercase_listed_app_identifier)
        } else {
            lowercase_app_identifier == lowercase_listed_app_identifier
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const YOUTUBE_MUSIC_IN_BRAVE: &str = "Brave._crx_cinhimbnkkghhklpknlkffjgod";

    #[test]
    fn an_empty_list_allows_no_player_and_every_player_allows_all() {
        assert!(!MediaPlayerFilter::OnlyListedPlayers(Vec::new()).allows_app_identifier("Spotify.exe"));
        assert!(MediaPlayerFilter::EveryPlayer.allows_app_identifier("Spotify.exe"));
    }

    #[test]
    fn a_web_app_entry_matches_that_app_in_any_browser_ignoring_case() {
        let youtube_music_filter = MediaPlayerFilter::OnlyListedPlayers(vec!["_CRX_cinhimbnkkghhklpknlkffjgod".to_string()]);
        assert!(youtube_music_filter.allows_app_identifier(YOUTUBE_MUSIC_IN_BRAVE));
        assert!(youtube_music_filter.allows_app_identifier("Chrome._crx_cinhimbnkkghhklpknlkffjgod"));
        assert!(!youtube_music_filter.allows_app_identifier("Brave"));
    }

    #[test]
    fn the_brave_tab_player_does_not_match_brave_web_apps() {
        let brave_tabs_filter = MediaPlayerFilter::OnlyListedPlayers(vec!["Brave".to_string()]);
        assert!(brave_tabs_filter.allows_app_identifier("brave"));
        assert!(!brave_tabs_filter.allows_app_identifier(YOUTUBE_MUSIC_IN_BRAVE));
    }
}
