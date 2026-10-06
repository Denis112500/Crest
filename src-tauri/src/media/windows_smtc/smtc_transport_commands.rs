//! Sends play/pause, next and previous to a player session.

use windows::Media::Control::GlobalSystemMediaTransportControlsSession as SmtcSession;

use crate::media::media_source::MediaTransportCommand;

/// Must be called from the SMTC worker thread: `.join()` waits for the player's answer.
/// A button press has nobody to return an error to, so failures are only reported.
pub fn send_smtc_transport_command(session: &SmtcSession, media_transport_command: MediaTransportCommand) {
    match request_smtc_transport_command(session, media_transport_command) {
        Ok(true) => {}
        // The player received the request but declined it (for example "previous" on the
        // first track of a queue).
        Ok(false) => eprintln!("Crest media: the player declined {media_transport_command:?}"),
        Err(command_error) => eprintln!("Crest media: could not send {media_transport_command:?}: {command_error}"),
    }
}

fn request_smtc_transport_command(
    session: &SmtcSession,
    media_transport_command: MediaTransportCommand,
) -> windows::core::Result<bool> {
    let command_request = match media_transport_command {
        MediaTransportCommand::TogglePlayPause => session.TryTogglePlayPauseAsync()?,
        MediaTransportCommand::NextTrack => session.TrySkipNextAsync()?,
        MediaTransportCommand::PreviousTrack => session.TrySkipPreviousAsync()?,
    };
    command_request.join()
}
