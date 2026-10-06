// The `forward` mode, run by Claude Code as a command hook for every event: stdin → pipe →
// answer → stdout. It always exits with code 0 so a missing listener is never an error in
// Claude Code; no output means "no decision".

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::{ERROR_PIPE_BUSY, FILETIME};
use windows::Win32::System::SystemInformation::GetSystemTimeAsFileTime;
use windows::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};

use crate::probe_pipe_listener::PROBE_PIPE_NAME;

/// All pipe instances are briefly busy only between two connections; a few short retries cover it.
const PIPE_BUSY_RETRY_COUNT: u32 = 25;
const PIPE_BUSY_RETRY_DELAY: Duration = Duration::from_millis(20);
const FILETIME_TICKS_PER_MILLISECOND: u64 = 10_000;

pub fn forward_hook_event_to_listener() {
    let milliseconds_since_start = milliseconds_since_this_process_started();
    let mut hook_event_bytes = Vec::new();
    if std::io::stdin().read_to_end(&mut hook_event_bytes).is_err() {
        return;
    }
    let Some(mut listener_pipe) = open_listener_pipe() else {
        return;
    };
    let mut request_bytes = Vec::with_capacity(12 + hook_event_bytes.len());
    request_bytes.extend_from_slice(&milliseconds_since_start.to_le_bytes());
    request_bytes.extend_from_slice(&(hook_event_bytes.len() as u32).to_le_bytes());
    request_bytes.extend_from_slice(&hook_event_bytes);
    if listener_pipe.write_all(&request_bytes).is_err() {
        return;
    }
    let mut reply_length_bytes = [0u8; 4];
    if listener_pipe.read_exact(&mut reply_length_bytes).is_err() {
        return;
    }
    let mut reply_bytes = vec![0u8; u32::from_le_bytes(reply_length_bytes) as usize];
    if listener_pipe.read_exact(&mut reply_bytes).is_err() {
        return;
    }
    let _ = std::io::stdout().write_all(&reply_bytes);
}

/// None when no listener runs (the pipe doesn't exist) or it refuses this user.
fn open_listener_pipe() -> Option<File> {
    for _ in 0..PIPE_BUSY_RETRY_COUNT {
        match OpenOptions::new().read(true).write(true).open(PROBE_PIPE_NAME) {
            Ok(listener_pipe) => return Some(listener_pipe),
            Err(open_error) if open_error.raw_os_error() == Some(ERROR_PIPE_BUSY.0 as i32) => {
                thread::sleep(PIPE_BUSY_RETRY_DELAY)
            }
            Err(_) => return None,
        }
    }
    None
}

/// How long Windows took to start this process before it could send; the cost a command
/// hook adds to every event.
fn milliseconds_since_this_process_started() -> u64 {
    let mut creation_time = FILETIME::default();
    let mut exit_time = FILETIME::default();
    let mut kernel_time = FILETIME::default();
    let mut user_time = FILETIME::default();
    let times_result = unsafe {
        GetProcessTimes(GetCurrentProcess(), &mut creation_time, &mut exit_time, &mut kernel_time, &mut user_time)
    };
    if times_result.is_err() {
        return 0;
    }
    let current_time = unsafe { GetSystemTimeAsFileTime() };
    let as_ticks = |file_time: FILETIME| (u64::from(file_time.dwHighDateTime) << 32) | u64::from(file_time.dwLowDateTime);
    as_ticks(current_time).saturating_sub(as_ticks(creation_time)) / FILETIME_TICKS_PER_MILLISECOND
}
