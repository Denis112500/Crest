//! Readable names for players' app IDs in the settings window ("YouTube Music", "Brave tabs").

use crate::backend_constants::DEFAULT_ALLOWED_MEDIA_APP_IDENTIFIER_FRAGMENT;

const YOUTUBE_MUSIC_LABEL: &str = "YouTube Music";
/// A normal browser tab plays as the browser's own player, e.g. "Brave" (measured 2026-10-04).
const BRAVE_TAB_PLAYER_IDENTIFIER: &str = "Brave";
const BRAVE_TAB_PLAYER_LABEL: &str = "Brave tabs";
/// Chromium browsers name an installed web app "<Browser>._crx_<app id>".
const BROWSER_WEB_APP_MARKER: &str = "._crx_";
const WINDOWS_EXECUTABLE_SUFFIX: &str = ".exe";
/// Store apps are "<Package>_<publisher hash>!<App>"; the package name is the readable part.
const STORE_APP_ENTRY_SEPARATOR: char = '!';
const STORE_APP_PUBLISHER_SEPARATOR: char = '_';

/// A readable name for a player's app ID (or a saved part of one) for the settings window.
/// The rules cover the IDs Windows reports; anything unknown is shown as it is, and the
/// window shows the raw ID under the name anyway.
pub fn describe_media_app_identifier(app_identifier: &str) -> String {
    let lowercase_app_identifier = app_identifier.to_lowercase();
    if lowercase_app_identifier.contains(&DEFAULT_ALLOWED_MEDIA_APP_IDENTIFIER_FRAGMENT.to_lowercase()) {
        return YOUTUBE_MUSIC_LABEL.to_string();
    }
    if app_identifier.eq_ignore_ascii_case(BRAVE_TAB_PLAYER_IDENTIFIER) {
        return BRAVE_TAB_PLAYER_LABEL.to_string();
    }
    if let Some(browser_name_end) = app_identifier.find(BROWSER_WEB_APP_MARKER) {
        return format!("{} web app", &app_identifier[..browser_name_end]);
    }
    if lowercase_app_identifier.ends_with(WINDOWS_EXECUTABLE_SUFFIX) {
        return app_identifier[..app_identifier.len() - WINDOWS_EXECUTABLE_SUFFIX.len()].to_string();
    }
    if let Some((store_package, _store_app_entry)) = app_identifier.split_once(STORE_APP_ENTRY_SEPARATOR) {
        if let Some((store_package_name, _publisher_hash)) = store_package.split_once(STORE_APP_PUBLISHER_SEPARATOR) {
            return store_package_name.to_string();
        }
    }
    app_identifier.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_youtube_music_in_any_browser_and_as_a_saved_fragment() {
        assert_eq!(describe_media_app_identifier("Brave._crx_cinhimbnkkghhklpknlkffjgod"), "YouTube Music");
        assert_eq!(describe_media_app_identifier("_crx_cinhimbnkkghhklpknlkffjgod"), "YouTube Music");
    }

    #[test]
    fn names_other_web_apps_desktop_programs_and_store_apps() {
        assert_eq!(describe_media_app_identifier("Brave._crx_abcdefghijklmnop"), "Brave web app");
        assert_eq!(describe_media_app_identifier("Brave"), "Brave tabs");
        assert_eq!(describe_media_app_identifier("Spotify.exe"), "Spotify");
        assert_eq!(
            describe_media_app_identifier("Microsoft.ZuneMusic_8wekyb3d8bbwe!Microsoft.ZuneMusic"),
            "Microsoft.ZuneMusic"
        );
    }

    #[test]
    fn shows_an_unknown_identifier_as_it_is() {
        assert_eq!(describe_media_app_identifier("SomePlayer"), "SomePlayer");
    }
}
