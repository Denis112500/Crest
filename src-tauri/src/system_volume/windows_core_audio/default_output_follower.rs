//! Keeps Crest on the current default output device: opens it, reopens it when Windows switches
//! to another one (or the device disappears), and reports its volume to the listener.

use std::sync::mpsc::Sender;

use windows::core::GUID;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolumeCallback;
use windows::Win32::Media::Audio::{IMMDeviceEnumerator, AUDCLNT_E_DEVICE_INVALIDATED};

use crate::system_volume::system_volume_level::SystemVolumeLevel;
use crate::system_volume::system_volume_platform::SystemVolumeListener;
use crate::system_volume::system_volume_reading::{SystemVolumeReading, SystemVolumeState};
use crate::system_volume::windows_core_audio::core_audio_worker_message::CoreAudioWorkerMessage;
use crate::system_volume::windows_core_audio::endpoint_volume_change_callback::EndpointVolumeChangeCallback;
use crate::system_volume::windows_core_audio::watched_output_endpoint::WatchedOutputEndpoint;

pub struct DefaultOutputFollower {
    device_enumerator: IMMDeviceEnumerator,
    worker_message_sender: Sender<CoreAudioWorkerMessage>,
    crest_event_context: GUID,
    system_volume_listener: SystemVolumeListener,
    /// `None` while Windows has no output device.
    watched_output_endpoint: Option<WatchedOutputEndpoint>,
    /// Counts the devices opened; notifications from an earlier one are ignored.
    endpoint_generation: u64,
}

impl DefaultOutputFollower {
    pub fn new(
        device_enumerator: IMMDeviceEnumerator,
        worker_message_sender: Sender<CoreAudioWorkerMessage>,
        crest_event_context: GUID,
        system_volume_listener: SystemVolumeListener,
    ) -> Self {
        Self {
            device_enumerator,
            worker_message_sender,
            crest_event_context,
            system_volume_listener,
            watched_output_endpoint: None,
            endpoint_generation: 0,
        }
    }

    /// Lets go of the device watched so far, opens the current default and reports its volume.
    pub fn follow_default_output(&mut self) {
        // Unregisters the old callback before the new one is registered.
        self.watched_output_endpoint = None;
        self.endpoint_generation += 1;
        let volume_change_callback: IAudioEndpointVolumeCallback = EndpointVolumeChangeCallback::new(
            self.worker_message_sender.clone(),
            self.endpoint_generation,
            self.crest_event_context,
        )
        .into();
        self.watched_output_endpoint =
            match WatchedOutputEndpoint::open_default_output(&self.device_enumerator, volume_change_callback) {
                Ok(watched_output_endpoint) => watched_output_endpoint,
                Err(open_error) => {
                    eprintln!("Crest system volume: could not open the default output device: {open_error}");
                    None
                }
            };
        let current_system_volume = self.watched_output_endpoint.as_ref().and_then(|watched_output_endpoint| {
            watched_output_endpoint
                .read_system_volume()
                .inspect_err(|read_error| eprintln!("Crest system volume: could not read the volume: {read_error}"))
                .ok()
        });
        self.report(current_system_volume, false);
    }

    pub fn handle_volume_notification(
        &self,
        endpoint_generation: u64,
        reported_system_volume: SystemVolumeState,
        is_crest_own_change: bool,
    ) {
        if endpoint_generation == self.endpoint_generation {
            self.report(Some(reported_system_volume), is_crest_own_change);
        }
    }

    pub fn set_level(&mut self, requested_level: SystemVolumeLevel) {
        // No output device: the page greys the speaker out, so a request can only be a late one.
        let Some(watched_output_endpoint) = &self.watched_output_endpoint else {
            return;
        };
        match watched_output_endpoint.set_level(requested_level, &self.crest_event_context) {
            Ok(()) => {}
            // The device went away before Windows told us; follow whatever is default now.
            Err(set_error) if set_error.code() == AUDCLNT_E_DEVICE_INVALIDATED => self.follow_default_output(),
            Err(set_error) => eprintln!("Crest system volume: could not set the level: {set_error}"),
        }
    }

    fn report(&self, system_volume: Option<SystemVolumeState>, is_crest_own_change: bool) {
        (self.system_volume_listener)(SystemVolumeReading { system_volume, is_crest_own_change });
    }
}
