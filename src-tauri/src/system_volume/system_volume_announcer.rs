//! Hands the system volume to the pill page: keeps the latest state for the page's "get"
//! command, and sends an event only when the page has something new to show.

use std::sync::Mutex;

use crate::system_volume::system_volume_reading::{SystemVolumeReading, SystemVolumeState};

/// Sends the state to the page (`None` = no output device).
pub type SystemVolumeEmitter = Box<dyn Fn(Option<SystemVolumeState>) + Send + Sync + 'static>;

pub struct SystemVolumeAnnouncer {
    /// `None` until the OS reports, and whenever there is no output device.
    current_system_volume: Mutex<Option<SystemVolumeState>>,
    system_volume_emitter: SystemVolumeEmitter,
}

impl SystemVolumeAnnouncer {
    pub fn new(system_volume_emitter: SystemVolumeEmitter) -> Self {
        Self { current_system_volume: Mutex::new(None), system_volume_emitter }
    }

    pub fn current_system_volume(&self) -> Option<SystemVolumeState> {
        *self.current_system_volume.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn handle_system_volume_reading(&self, system_volume_reading: SystemVolumeReading) {
        let has_changed = {
            let mut current_system_volume =
                self.current_system_volume.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            let has_changed = *current_system_volume != system_volume_reading.system_volume;
            *current_system_volume = system_volume_reading.system_volume;
            has_changed
        };
        // Crest's own change is already on the page (the user moved the slider there). Sent back,
        // an echo of an earlier drag step would arrive after a later one and pull the slider back.
        if has_changed && !system_volume_reading.is_crest_own_change {
            (self.system_volume_emitter)(system_volume_reading.system_volume);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::system_volume::system_volume_level::SystemVolumeLevel;

    fn volume_state(level_scalar: f32, is_muted: bool) -> SystemVolumeState {
        SystemVolumeState { level: SystemVolumeLevel::from_reported_scalar(level_scalar), is_muted }
    }

    fn reading(system_volume: Option<SystemVolumeState>, is_crest_own_change: bool) -> SystemVolumeReading {
        SystemVolumeReading { system_volume, is_crest_own_change }
    }

    fn announcer_with_sent_list() -> (SystemVolumeAnnouncer, Arc<Mutex<Vec<Option<SystemVolumeState>>>>) {
        let sent_to_page: Arc<Mutex<Vec<Option<SystemVolumeState>>>> = Arc::default();
        let sent_to_page_for_emitter = Arc::clone(&sent_to_page);
        let system_volume_announcer = SystemVolumeAnnouncer::new(Box::new(move |system_volume| {
            sent_to_page_for_emitter.lock().unwrap().push(system_volume);
        }));
        (system_volume_announcer, sent_to_page)
    }

    #[test]
    fn a_change_from_elsewhere_is_sent_and_kept() {
        let (system_volume_announcer, sent_to_page) = announcer_with_sent_list();
        system_volume_announcer.handle_system_volume_reading(reading(Some(volume_state(0.5, false)), false));
        assert_eq!(*sent_to_page.lock().unwrap(), vec![Some(volume_state(0.5, false))]);
        assert_eq!(system_volume_announcer.current_system_volume(), Some(volume_state(0.5, false)));
    }

    #[test]
    fn the_same_state_again_is_not_sent_twice() {
        let (system_volume_announcer, sent_to_page) = announcer_with_sent_list();
        system_volume_announcer.handle_system_volume_reading(reading(Some(volume_state(0.5, false)), false));
        system_volume_announcer.handle_system_volume_reading(reading(Some(volume_state(0.5, false)), false));
        assert_eq!(sent_to_page.lock().unwrap().len(), 1);
    }

    #[test]
    fn crests_own_change_is_kept_but_not_sent_back() {
        let (system_volume_announcer, sent_to_page) = announcer_with_sent_list();
        system_volume_announcer.handle_system_volume_reading(reading(Some(volume_state(0.3, false)), true));
        assert!(sent_to_page.lock().unwrap().is_empty());
        assert_eq!(system_volume_announcer.current_system_volume(), Some(volume_state(0.3, false)));
    }

    #[test]
    fn a_key_press_back_to_the_old_level_after_crests_change_is_sent() {
        let (system_volume_announcer, sent_to_page) = announcer_with_sent_list();
        system_volume_announcer.handle_system_volume_reading(reading(Some(volume_state(0.5, false)), false));
        system_volume_announcer.handle_system_volume_reading(reading(Some(volume_state(0.3, false)), true));
        system_volume_announcer.handle_system_volume_reading(reading(Some(volume_state(0.5, false)), false));
        assert_eq!(*sent_to_page.lock().unwrap(), vec![Some(volume_state(0.5, false)), Some(volume_state(0.5, false))]);
    }

    #[test]
    fn a_mute_alone_is_a_change() {
        let (system_volume_announcer, sent_to_page) = announcer_with_sent_list();
        system_volume_announcer.handle_system_volume_reading(reading(Some(volume_state(0.5, false)), false));
        system_volume_announcer.handle_system_volume_reading(reading(Some(volume_state(0.5, true)), false));
        assert_eq!(sent_to_page.lock().unwrap().last(), Some(&Some(volume_state(0.5, true))));
    }

    #[test]
    fn a_lost_output_device_is_sent_once_and_its_return_again() {
        let (system_volume_announcer, sent_to_page) = announcer_with_sent_list();
        system_volume_announcer.handle_system_volume_reading(reading(Some(volume_state(0.5, false)), false));
        system_volume_announcer.handle_system_volume_reading(reading(None, false));
        system_volume_announcer.handle_system_volume_reading(reading(None, false));
        system_volume_announcer.handle_system_volume_reading(reading(Some(volume_state(0.8, false)), false));
        assert_eq!(
            *sent_to_page.lock().unwrap(),
            vec![Some(volume_state(0.5, false)), None, Some(volume_state(0.8, false))]
        );
    }

    #[test]
    fn no_output_device_at_start_sends_nothing() {
        // The page asked first and already got `None`.
        let (system_volume_announcer, sent_to_page) = announcer_with_sent_list();
        system_volume_announcer.handle_system_volume_reading(reading(None, false));
        assert!(sent_to_page.lock().unwrap().is_empty());
    }
}
