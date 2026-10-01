use std::time::Instant;

use crate::backend_constants::PENDING_MEDIA_TRANSPORT_COMMAND_LIFETIME;
use crate::media::media_source::MediaTransportCommand;

/// Button presses that arrived while the player's session was gone. Browsers drop the
/// session for about half a second on every track change, so pressing "next" twice quickly
/// used to lose the second press. Presses are kept in order and handed out once the session
/// is back, except those that have become too old to still mean anything.
#[derive(Default)]
pub struct PendingTransportCommands {
    held_commands: Vec<(MediaTransportCommand, Instant)>,
}

impl PendingTransportCommands {
    pub fn hold(&mut self, media_transport_command: MediaTransportCommand, pressed_at: Instant) {
        self.held_commands.push((media_transport_command, pressed_at));
    }

    /// Empties the list; only presses younger than the lifetime are returned, oldest first.
    pub fn take_still_relevant(&mut self, now: Instant) -> Vec<MediaTransportCommand> {
        self.held_commands
            .drain(..)
            .filter(|(_, pressed_at)| now.duration_since(*pressed_at) < PENDING_MEDIA_TRANSPORT_COMMAND_LIFETIME)
            .map(|(media_transport_command, _)| media_transport_command)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn presses_during_the_gap_are_handed_out_in_order() {
        let mut pending_transport_commands = PendingTransportCommands::default();
        let first_press_at = Instant::now();
        pending_transport_commands.hold(MediaTransportCommand::NextTrack, first_press_at);
        pending_transport_commands
            .hold(MediaTransportCommand::TogglePlayPause, first_press_at + Duration::from_millis(200));
        assert_eq!(
            pending_transport_commands.take_still_relevant(first_press_at + Duration::from_millis(500)),
            vec![MediaTransportCommand::NextTrack, MediaTransportCommand::TogglePlayPause]
        );
        assert!(pending_transport_commands.take_still_relevant(first_press_at + Duration::from_secs(1)).is_empty());
    }

    #[test]
    fn presses_older_than_the_lifetime_are_dropped() {
        let mut pending_transport_commands = PendingTransportCommands::default();
        let pressed_at = Instant::now();
        pending_transport_commands.hold(MediaTransportCommand::PreviousTrack, pressed_at);
        assert!(pending_transport_commands
            .take_still_relevant(pressed_at + PENDING_MEDIA_TRANSPORT_COMMAND_LIFETIME)
            .is_empty());
    }
}
