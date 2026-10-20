//! Core Audio calls this when audio devices come, go or change roles. Only a new default
//! output device matters to Crest: the slider follows whatever the taskbar's speaker controls.

use std::sync::mpsc::Sender;

use windows::core::{implement, Result as WindowsResult, PCWSTR};
use windows::Win32::Foundation::PROPERTYKEY;
use windows::Win32::Media::Audio::{
    eConsole, eRender, EDataFlow, ERole, IMMNotificationClient, IMMNotificationClient_Impl, DEVICE_STATE,
};

use crate::system_volume::windows_core_audio::core_audio_worker_message::CoreAudioWorkerMessage;

/// Holds only a channel: the docs forbid blocking, (un)registering callbacks or releasing a
/// device's last reference inside these methods, so the worker thread does all of that.
#[implement(IMMNotificationClient)]
pub struct DefaultOutputChangeCallback {
    worker_message_sender: Sender<CoreAudioWorkerMessage>,
}

impl DefaultOutputChangeCallback {
    pub fn new(worker_message_sender: Sender<CoreAudioWorkerMessage>) -> Self {
        Self { worker_message_sender }
    }
}

impl IMMNotificationClient_Impl for DefaultOutputChangeCallback_Impl {
    fn OnDefaultDeviceChanged(
        &self,
        data_flow: EDataFlow,
        device_role: ERole,
        _new_default_device_identifier: &PCWSTR,
    ) -> WindowsResult<()> {
        if is_watched_default_output_change(data_flow, device_role) {
            // A send fails only once the worker has stopped; nothing is left to tell then.
            let _ = self.worker_message_sender.send(CoreAudioWorkerMessage::DefaultOutputDeviceChanged);
        }
        Ok(())
    }

    fn OnDeviceStateChanged(&self, _device_identifier: &PCWSTR, _new_device_state: DEVICE_STATE) -> WindowsResult<()> {
        Ok(())
    }

    fn OnDeviceAdded(&self, _device_identifier: &PCWSTR) -> WindowsResult<()> {
        Ok(())
    }

    fn OnDeviceRemoved(&self, _device_identifier: &PCWSTR) -> WindowsResult<()> {
        Ok(())
    }

    fn OnPropertyValueChanged(&self, _device_identifier: &PCWSTR, _property_key: &PROPERTYKEY) -> WindowsResult<()> {
        Ok(())
    }
}

/// Windows calls once per role (console, multimedia, communications) when the default device
/// changes; the console output is the one Crest watches (`watched_output_endpoint.rs`), so one
/// change means one reopen.
fn is_watched_default_output_change(data_flow: EDataFlow, device_role: ERole) -> bool {
    data_flow == eRender && device_role == eConsole
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use windows::Win32::Media::Audio::{eCapture, eCommunications, eMultimedia};

    use super::*;

    fn messages_after_default_change(data_flow: EDataFlow, device_role: ERole) -> usize {
        let (worker_message_sender, worker_message_receiver) = mpsc::channel();
        let default_output_change_callback: IMMNotificationClient =
            DefaultOutputChangeCallback::new(worker_message_sender).into();
        // SAFETY: a null device identifier is what Windows sends when no device is left.
        unsafe { default_output_change_callback.OnDefaultDeviceChanged(data_flow, device_role, PCWSTR::null()) }
            .unwrap();
        worker_message_receiver.try_iter().count()
    }

    #[test]
    fn a_new_default_console_output_wakes_the_worker() {
        assert_eq!(messages_after_default_change(eRender, eConsole), 1);
    }

    #[test]
    fn the_other_two_roles_of_the_same_switch_are_ignored() {
        assert_eq!(messages_after_default_change(eRender, eMultimedia), 0);
        assert_eq!(messages_after_default_change(eRender, eCommunications), 0);
    }

    #[test]
    fn a_new_default_microphone_is_ignored() {
        assert_eq!(messages_after_default_change(eCapture, eConsole), 0);
    }
}
