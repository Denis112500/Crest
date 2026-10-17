//! Core Audio calls this when the watched output device's level or mute changes, from any
//! source: the volume keys, the taskbar slider, another app, or Crest itself.

use std::sync::mpsc::Sender;

use windows::core::{implement, Result as WindowsResult, GUID};
use windows::Win32::Media::Audio::Endpoints::{IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl};
use windows::Win32::Media::Audio::AUDIO_VOLUME_NOTIFICATION_DATA;

use crate::system_volume::system_volume_level::SystemVolumeLevel;
use crate::system_volume::system_volume_reading::SystemVolumeState;
use crate::system_volume::windows_core_audio::core_audio_worker_message::CoreAudioWorkerMessage;

/// Holds only a channel and two numbers, never a Core Audio object, so it can't block Core
/// Audio's thread or release a device from inside a callback.
#[implement(IAudioEndpointVolumeCallback)]
pub struct EndpointVolumeChangeCallback {
    worker_message_sender: Sender<CoreAudioWorkerMessage>,
    endpoint_generation: u64,
    /// Passed with every change Crest makes, and handed back by Core Audio in the notification.
    crest_event_context: GUID,
}

impl EndpointVolumeChangeCallback {
    pub fn new(
        worker_message_sender: Sender<CoreAudioWorkerMessage>,
        endpoint_generation: u64,
        crest_event_context: GUID,
    ) -> Self {
        Self { worker_message_sender, endpoint_generation, crest_event_context }
    }
}

impl IAudioEndpointVolumeCallback_Impl for EndpointVolumeChangeCallback_Impl {
    fn OnNotify(&self, volume_notification: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> WindowsResult<()> {
        // SAFETY: Core Audio passes a pointer that stays valid for the duration of this call;
        // `as_ref` turns a null pointer into `None`.
        let Some(volume_notification) = (unsafe { volume_notification.as_ref() }) else {
            return Ok(());
        };
        let reported_system_volume = SystemVolumeState {
            level: SystemVolumeLevel::from_reported_scalar(volume_notification.fMasterVolume),
            is_muted: volume_notification.bMuted.as_bool(),
        };
        // A send fails only once the worker has stopped; nothing is left to tell then.
        let _ = self.worker_message_sender.send(CoreAudioWorkerMessage::VolumeNotified {
            endpoint_generation: self.endpoint_generation,
            reported_system_volume,
            is_crest_own_change: volume_notification.guidEventContext == self.crest_event_context,
        });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use windows::core::BOOL;

    use super::*;

    const CREST_TEST_EVENT_CONTEXT: GUID = GUID::from_u128(0x6b1f_2c3a_0000_4000_8000_0000_0000_0001);

    fn notify_through_com(event_context: GUID, master_volume: f32, is_muted: bool) -> CoreAudioWorkerMessage {
        let (worker_message_sender, worker_message_receiver) = mpsc::channel();
        let volume_change_callback: IAudioEndpointVolumeCallback =
            EndpointVolumeChangeCallback::new(worker_message_sender, 7, CREST_TEST_EVENT_CONTEXT).into();
        let mut volume_notification = AUDIO_VOLUME_NOTIFICATION_DATA {
            guidEventContext: event_context,
            bMuted: BOOL::from(is_muted),
            fMasterVolume: master_volume,
            nChannels: 1,
            afChannelVolumes: [master_volume],
        };
        // SAFETY: the notification lives on this stack frame for the whole call.
        unsafe { volume_change_callback.OnNotify(&mut volume_notification) }.unwrap();
        worker_message_receiver.try_recv().unwrap()
    }

    #[test]
    fn crests_own_event_context_is_recognised() {
        let CoreAudioWorkerMessage::VolumeNotified { is_crest_own_change, .. } =
            notify_through_com(CREST_TEST_EVENT_CONTEXT, 0.5, false)
        else {
            panic!("expected a volume notification");
        };
        assert!(is_crest_own_change);
    }

    #[test]
    fn another_event_context_is_not_crests() {
        // GUID_NULL is what a change without a context (e.g. from another app) carries.
        let CoreAudioWorkerMessage::VolumeNotified { is_crest_own_change, .. } =
            notify_through_com(GUID::zeroed(), 0.5, false)
        else {
            panic!("expected a volume notification");
        };
        assert!(!is_crest_own_change);
    }

    #[test]
    fn the_notification_carries_rounded_level_mute_and_generation() {
        let CoreAudioWorkerMessage::VolumeNotified { endpoint_generation, reported_system_volume, .. } =
            notify_through_com(GUID::zeroed(), 0.639_999_9, true)
        else {
            panic!("expected a volume notification");
        };
        assert_eq!(endpoint_generation, 7);
        assert_eq!(reported_system_volume.level.as_scalar(), 0.64);
        assert!(reported_system_volume.is_muted);
    }
}
