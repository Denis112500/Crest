//! What the "Allowed players" card shows: the switch, the saved list, and the open players
//! that could be added.

use serde::Serialize;

use crate::backend_constants::DEFAULT_ALLOWED_MEDIA_APP_IDENTIFIER_FRAGMENT;
use crate::media::{describe_media_app_identifier, is_app_identifier_in_list};

/// One player as the settings window shows it: a readable name, and the ID (or part of one)
/// it's matched by, which is also what gets saved when it's added.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaPlayerDescription {
    pub app_identifier: String,
    pub label: String,
}

/// What the "Allowed players" card shows: the switch, the saved list, and the open players
/// that could be added to it.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowedPlayerOptions {
    pub show_every_player: bool,
    pub allowed_players: Vec<MediaPlayerDescription>,
    pub players_to_add: Vec<MediaPlayerDescription>,
}

/// Offers every open player the list doesn't match yet, whether or not "every player" is on,
/// so the list can be prepared before switching it off.
pub fn describe_allowed_player_options(
    show_every_player: bool,
    allowed_app_identifier_fragments: &[String],
    open_player_app_identifiers: &[String],
) -> AllowedPlayerOptions {
    let allowed_players = allowed_app_identifier_fragments.iter().map(|fragment| describe_media_player(fragment)).collect();
    let mut players_to_add: Vec<MediaPlayerDescription> = open_player_app_identifiers
        .iter()
        .filter(|open_app_identifier| !is_app_identifier_in_list(open_app_identifier, allowed_app_identifier_fragments))
        .map(|open_app_identifier| describe_media_player(&choose_identifier_to_save(open_app_identifier)))
        .collect();
    // Sorted by ID too, so duplicates (one app with several sessions, e.g. two playing tabs)
    // end up next to each other even when two apps share a label.
    players_to_add.sort_by(|first_player, second_player| {
        (&first_player.label, &first_player.app_identifier).cmp(&(&second_player.label, &second_player.app_identifier))
    });
    players_to_add.dedup_by(|first_player, second_player| first_player.app_identifier == second_player.app_identifier);
    AllowedPlayerOptions { show_every_player, allowed_players, players_to_add }
}

/// YouTube Music is saved by the part of its ID that's the same in every Chromium browser,
/// so the entry keeps working after switching from Brave to Chrome or Edge. Other players are
/// saved by their exact ID.
fn choose_identifier_to_save(open_app_identifier: &str) -> String {
    let youtube_music_fragment = [DEFAULT_ALLOWED_MEDIA_APP_IDENTIFIER_FRAGMENT.to_string()];
    if is_app_identifier_in_list(open_app_identifier, &youtube_music_fragment) {
        DEFAULT_ALLOWED_MEDIA_APP_IDENTIFIER_FRAGMENT.to_string()
    } else {
        open_app_identifier.to_string()
    }
}

fn describe_media_player(app_identifier: &str) -> MediaPlayerDescription {
    MediaPlayerDescription { app_identifier: app_identifier.to_string(), label: describe_media_app_identifier(app_identifier) }
}

#[cfg(test)]
mod tests {
    use super::*;

    const YOUTUBE_MUSIC_IN_BRAVE: &str = "Brave._crx_cinhimbnkkghhklpknlkffjgod";
    const SPOTIFY: &str = "Spotify.exe";

    #[test]
    fn offers_only_open_players_the_list_does_not_match_yet() {
        let player_options = describe_allowed_player_options(
            false,
            &[DEFAULT_ALLOWED_MEDIA_APP_IDENTIFIER_FRAGMENT.to_string()],
            &[YOUTUBE_MUSIC_IN_BRAVE.to_string(), SPOTIFY.to_string(), SPOTIFY.to_string()],
        );
        assert_eq!(player_options.allowed_players[0].label, "YouTube Music");
        assert_eq!(
            player_options.players_to_add,
            [MediaPlayerDescription { app_identifier: SPOTIFY.to_string(), label: "Spotify".to_string() }]
        );
    }

    #[test]
    fn offers_youtube_music_by_its_browser_independent_part() {
        let player_options = describe_allowed_player_options(false, &[], &[YOUTUBE_MUSIC_IN_BRAVE.to_string()]);
        assert!(player_options.allowed_players.is_empty());
        assert_eq!(player_options.players_to_add[0].app_identifier, DEFAULT_ALLOWED_MEDIA_APP_IDENTIFIER_FRAGMENT);
        assert_eq!(player_options.players_to_add[0].label, "YouTube Music");
    }
}
