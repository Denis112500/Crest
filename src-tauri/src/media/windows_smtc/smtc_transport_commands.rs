use windows::Media::Control::GlobalSystemMediaTransportControlsSession as SmtcSession;

use crate::media::media_source::MediaTransportCommand;

/// Must be called from the SMTC worker thread: `.join()` waits for the player's answer.
/// `Ok(false)` means the player received the request but declined it (for example
/// "previous" on the first track of a queue).
pub fn send_smtc_transport_command(
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
