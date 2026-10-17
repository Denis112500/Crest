//! The default output device's volume control with Crest's change callback registered on it.
//! Dropping it unregisters the callback first, as the docs require (otherwise both leak).

use windows::core::{Result as WindowsResult, GUID, HRESULT};
use windows::Win32::Foundation::ERROR_NOT_FOUND;
use windows::Win32::Media::Audio::Endpoints::{IAudioEndpointVolume, IAudioEndpointVolumeCallback};
use windows::Win32::Media::Audio::{eConsole, eRender, IMMDeviceEnumerator};
use windows::Win32::System::Com::CLSCTX_ALL;

use crate::system_volume::system_volume_level::SystemVolumeLevel;
use crate::system_volume::system_volume_reading::SystemVolumeState;

pub struct WatchedOutputEndpoint {
    endpoint_volume: IAudioEndpointVolume,
    volume_change_callback: IAudioEndpointVolumeCallback,
}

impl WatchedOutputEndpoint {
    /// `Ok(None)` when Windows has no output device at all (E_NOTFOUND, verified from docs).
    pub fn open_default_output(
        device_enumerator: &IMMDeviceEnumerator,
        volume_change_callback: IAudioEndpointVolumeCallback,
    ) -> WindowsResult<Option<Self>> {
        // The console role is the one the taskbar's speaker controls; Windows 11 gives all three
        // roles to the same output device.
        // SAFETY: called on the worker thread, which initialised COM.
        let default_output_device = match unsafe { device_enumerator.GetDefaultAudioEndpoint(eRender, eConsole) } {
            Ok(default_output_device) => default_output_device,
            Err(endpoint_error) if endpoint_error.code() == HRESULT::from_win32(ERROR_NOT_FOUND.0) => return Ok(None),
            Err(endpoint_error) => return Err(endpoint_error),
        };
        // SAFETY: as above; no activation parameters are needed for the volume control.
        let endpoint_volume: IAudioEndpointVolume = unsafe { default_output_device.Activate(CLSCTX_ALL, None)? };
        // SAFETY: as above; Core Audio keeps its own reference to the callback until unregistered.
        unsafe { endpoint_volume.RegisterControlChangeNotify(&volume_change_callback)? };
        Ok(Some(Self { endpoint_volume, volume_change_callback }))
    }

    pub fn read_system_volume(&self) -> WindowsResult<SystemVolumeState> {
        // SAFETY: plain reads on the worker thread that owns the object.
        let (level_scalar, is_muted) = unsafe {
            (self.endpoint_volume.GetMasterVolumeLevelScalar()?, self.endpoint_volume.GetMute()?.as_bool())
        };
        Ok(SystemVolumeState { level: SystemVolumeLevel::from_reported_scalar(level_scalar), is_muted })
    }

    /// Moving the slider while muted brings the sound back, like Windows' own slider. Both
    /// changes carry Crest's event context, so their notifications are recognised as Crest's.
    pub fn set_level(&self, requested_level: SystemVolumeLevel, crest_event_context: &GUID) -> WindowsResult<()> {
        // SAFETY: calls on the worker thread that owns the object; the context outlives them.
        unsafe {
            if self.endpoint_volume.GetMute()?.as_bool() {
                self.endpoint_volume.SetMute(false, crest_event_context)?;
            }
            self.endpoint_volume.SetMasterVolumeLevelScalar(requested_level.as_scalar(), crest_event_context)
        }
    }
}

impl Drop for WatchedOutputEndpoint {
    fn drop(&mut self) {
        // SAFETY: on the worker thread, never inside a Core Audio callback. A device that is
        // already gone may refuse; there is nothing else to clean up then.
        let _ = unsafe { self.endpoint_volume.UnregisterControlChangeNotify(&self.volume_change_callback) };
    }
}
