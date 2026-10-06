//! "Start with Windows": switching it, and repairing it after an update removed it.

mod launch_at_login_repair;
mod launch_at_login_switch;

#[cfg(target_os = "windows")]
mod windows_run_key_registration;

pub use launch_at_login_repair::repair_launch_at_login_after_update;
pub use launch_at_login_switch::{is_launch_at_login_enabled, set_launch_at_login};

#[cfg(target_os = "windows")]
use windows_run_key_registration::read_run_key_entry_state as read_launch_at_login_entry_state;
