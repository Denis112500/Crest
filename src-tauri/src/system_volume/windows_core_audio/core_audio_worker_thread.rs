use std::sync::mpsc::{Receiver, Sender};

use windows::core::GUID;
use windows::Win32::Media::Audio::{IMMDeviceEnumerator, IMMNotificationClient, MMDeviceEnumerator};
use windows::Win32::System::Com::{CoCreateGuid, CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED};

use crate::system_volume::system_volume_platform::SystemVolumeListener;
use crate::system_volume::windows_core_audio::core_audio_worker_message::CoreAudioWorkerMessage;
use crate::system_volume::windows_core_audio::default_output_change_callback::DefaultOutputChangeCallback;
use crate::system_volume::windows_core_audio::default_output_follower::DefaultOutputFollower;

/// Runs for the whole life of the app on its own thread and owns every Core Audio object.
/// It sleeps until a callback or the slider sends a message, so it costs no CPU while the
/// volume stays the same.
pub fn run_core_audio_worker_thread(
    system_volume_listener: SystemVolumeListener,
    worker_message_sender: Sender<CoreAudioWorkerMessage>,
    worker_message_receiver: Receiver<CoreAudioWorkerMessage>,
) {
    // The multithreaded apartment, like the SMTC worker: Core Audio calls the callbacks on its
    // own threads, and no window message loop is needed.
    // SAFETY: called once, first thing on this new thread, before any other COM call on it.
    if let Err(initialization_error) = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.ok() {
        eprintln!("Crest system volume: could not initialize COM: {initialization_error}");
        return;
    }
    // Without these the page keeps "no output device" (the state before any reading) and greys
    // the speaker out.
    let Some((device_enumerator, crest_event_context)) = create_device_enumerator_and_event_context() else {
        return;
    };
    let default_output_change_callback: IMMNotificationClient =
        DefaultOutputChangeCallback::new(worker_message_sender.clone()).into();
    // SAFETY: on this thread, which initialised COM. Registered for the app's whole life, so it
    // is never unregistered.
    if let Err(register_error) =
        unsafe { device_enumerator.RegisterEndpointNotificationCallback(&default_output_change_callback) }
    {
        // Not fatal: the volume still works, only a change of output device goes unnoticed.
        eprintln!("Crest system volume: could not watch for a new output device: {register_error}");
    }
    let mut default_output_follower =
        DefaultOutputFollower::new(device_enumerator, worker_message_sender, crest_event_context, system_volume_listener);
    default_output_follower.follow_default_output();

    while let Ok(worker_message) = worker_message_receiver.recv() {
        match worker_message {
            CoreAudioWorkerMessage::VolumeNotified { endpoint_generation, reported_system_volume, is_crest_own_change } => {
                default_output_follower.handle_volume_notification(
                    endpoint_generation,
                    reported_system_volume,
                    is_crest_own_change,
                )
            }
            CoreAudioWorkerMessage::DefaultOutputDeviceChanged => default_output_follower.follow_default_output(),
            CoreAudioWorkerMessage::LevelRequested(requested_level) => default_output_follower.set_level(requested_level),
        }
    }
}

fn create_device_enumerator_and_event_context() -> Option<(IMMDeviceEnumerator, GUID)> {
    // SAFETY: COM is initialised on this thread.
    let device_enumerator: IMMDeviceEnumerator = match unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL) }
    {
        Ok(device_enumerator) => device_enumerator,
        Err(create_error) => {
            eprintln!("Crest system volume: Windows' audio devices are unavailable: {create_error}");
            return None;
        }
    };
    // A fresh value per run is enough: it only has to differ from every other app's context.
    // SAFETY: no preconditions.
    match unsafe { CoCreateGuid() } {
        Ok(crest_event_context) => Some((device_enumerator, crest_event_context)),
        Err(guid_error) => {
            eprintln!("Crest system volume: could not create an event context: {guid_error}");
            None
        }
    }
}
