// Milestone (c) only: shows in the terminal what the media source sees.
// Milestone (d) replaces this with events sent to the frontend.

use std::time::{SystemTime, UNIX_EPOCH};

use crate::media::{MediaPlaybackState, MediaSessionSnapshot};

pub fn print_media_snapshot_to_console(media_snapshot: Option<MediaSessionSnapshot>) {
    let Some(media_snapshot) = media_snapshot else {
        println!("[media] no allowed media session");
        return;
    };
    let playback_state_label = match media_snapshot.playback_state {
        MediaPlaybackState::Playing => "PLAYING",
        MediaPlaybackState::Paused => "PAUSED",
        MediaPlaybackState::Changing => "CHANGING",
        MediaPlaybackState::Stopped => "STOPPED",
    };
    let album_art_description = match &media_snapshot.album_art_data_url {
        Some(album_art_data_url) => {
            let image_mime_type = album_art_data_url.trim_start_matches("data:").split(';').next().unwrap_or("?");
            format!("{image_mime_type}, {} KB as data URL", album_art_data_url.len() / 1024)
        }
        None => "none".to_string(),
    };
    let timeline_description = match media_snapshot.timeline {
        Some(timeline) => {
            let now_unix_milliseconds = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|since_epoch| since_epoch.as_millis() as i64)
                .unwrap_or(timeline.position_reported_at_unix_milliseconds);
            format!(
                "{} / {} (reported {} ms ago)",
                format_milliseconds_as_clock(timeline.reported_position_milliseconds),
                format_milliseconds_as_clock(timeline.track_duration_milliseconds),
                now_unix_milliseconds - timeline.position_reported_at_unix_milliseconds,
            )
        }
        None => "none".to_string(),
    };
    println!(
        "[media] {playback_state_label} \"{}\" by \"{}\" on \"{}\"\n        app: {}\n        art: {album_art_description}\n        time: {timeline_description}",
        media_snapshot.track_title,
        media_snapshot.track_artist,
        media_snapshot.album_title,
        media_snapshot.source_app_identifier,
    );
}

fn format_milliseconds_as_clock(total_milliseconds: i64) -> String {
    let total_seconds = total_milliseconds / 1000;
    format!("{}:{:02}", total_seconds / 60, total_seconds % 60)
}
