//! The Windows `SystemVolumePlatform`: starts the Core Audio worker thread and passes the
//! slider's requests to it as messages.

use std::sync::mpsc::{self, Sender};
use std::thread;

use crate::system_volume::system_volume_level::SystemVolumeLevel;
use crate::system_volume::system_volume_platform::{SystemVolumeListener, SystemVolumePlatform};
use crate::system_volume::windows_core_audio::core_audio_worker_message::CoreAudioWorkerMessage;
use crate::system_volume::windows_core_audio::core_audio_worker_thread::run_core_audio_worker_thread;

const CORE_AUDIO_WORKER_THREAD_NAME: &str = "crest-system-volume";

/// Windows implementation of `SystemVolumePlatform`, through Core Audio: the same volume as the
/// taskbar's speaker slider.
#[derive(Default)]
pub struct CoreAudioSystemVolume {
    /// Set when watching starts; the channel's receiving end lives on the worker thread.
    worker_message_sender: Option<Sender<CoreAudioWorkerMessage>>,
}

impl SystemVolumePlatform for CoreAudioSystemVolume {
    fn start_watching_system_volume(&mut self, system_volume_listener: SystemVolumeListener) -> Result<(), String> {
        if self.worker_message_sender.is_some() {
            return Err("system volume watching has already started".to_string());
        }
        let (worker_message_sender, worker_message_receiver) = mpsc::channel();
        let sender_for_callbacks = worker_message_sender.clone();
        thread::Builder::new()
            .name(CORE_AUDIO_WORKER_THREAD_NAME.to_string())
            .spawn(move || run_core_audio_worker_thread(system_volume_listener, sender_for_callbacks, worker_message_receiver))
            .map_err(|spawn_error| spawn_error.to_string())?;
        self.worker_message_sender = Some(worker_message_sender);
        Ok(())
    }

    fn request_system_volume_level(&self, requested_level: SystemVolumeLevel) -> Result<(), String> {
        self.worker_message_sender
            .as_ref()
            .ok_or("system volume watching hasn't started")?
            .send(CoreAudioWorkerMessage::LevelRequested(requested_level))
            .map_err(|_| "the system volume worker thread has stopped".to_string())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::Duration;

    use super::*;

    /// Reads this PC's real output device (never changes it). Ignored by default, since a build
    /// machine may have no sound device: `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn reads_this_pcs_system_volume() {
        let (reading_sender, reading_receiver) = mpsc::channel();
        let mut core_audio_system_volume = CoreAudioSystemVolume::default();
        core_audio_system_volume
            .start_watching_system_volume(Box::new(move |system_volume_reading| {
                let _ = reading_sender.send(system_volume_reading);
            }))
            .unwrap();
        let first_reading = reading_receiver.recv_timeout(Duration::from_secs(2)).unwrap();
        println!("{first_reading:?}");
        assert!(first_reading.system_volume.is_some(), "no output device found");
        assert!(!first_reading.is_crest_own_change);
    }
}
