//! Windows' system volume: Core Audio, the API behind the taskbar's speaker slider
//! (`IAudioEndpointVolume` of the default output device).

mod core_audio_system_volume;
mod core_audio_worker_message;
mod core_audio_worker_thread;
mod default_output_change_callback;
mod default_output_follower;
mod endpoint_volume_change_callback;
mod watched_output_endpoint;

pub use core_audio_system_volume::CoreAudioSystemVolume;
