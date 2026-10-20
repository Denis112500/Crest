//! Watches the mouse pointer without taking it from anyone, so the closed pill (which lets
//! every click through) can open when the pointer rests on it. The OS-specific part sits behind
//! a trait, like the other platform code.

mod pointer_rest_detector;
mod pointer_rest_startup;
mod pointer_rest_zone;
mod pointer_watch;
mod shared_pill_pointer_zones;

#[cfg(target_os = "windows")]
mod windows_raw_input;

pub use pointer_rest_startup::start_watching_pointer_rest;
pub use pointer_rest_zone::PointerRestZone;
pub use shared_pill_pointer_zones::SharedPillPointerZones;

#[cfg(target_os = "windows")]
pub use windows_raw_input::RawInputPointerWatch as CurrentPlatformPointerWatch;
