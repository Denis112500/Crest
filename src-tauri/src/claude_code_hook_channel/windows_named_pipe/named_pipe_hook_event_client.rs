//! The hook's end of the pipe: connect, make sure the other end is a Crest run by the same
//! user, send the event, read the reply.

use std::fs::{File, OpenOptions};
use std::os::windows::io::AsRawHandle;
use std::thread;

use windows::Win32::Foundation::{ERROR_PIPE_BUSY, HANDLE};
use windows::Win32::System::Pipes::GetNamedPipeServerProcessId;

use crate::backend_constants::{CLAUDE_CODE_HOOK_PIPE_BUSY_RETRY_COUNT, CLAUDE_CODE_HOOK_PIPE_BUSY_RETRY_DELAY};
use crate::claude_code_hook_channel::length_prefixed_message::{read_length_prefixed_message, write_length_prefixed_message};
use crate::claude_code_hook_channel::windows_named_pipe::windows_user_identity::read_user_sid_of_process;

/// `None` when no Crest of this user listens: the pipe doesn't exist (Crest isn't running or
/// the integration is off), it stays busy, or another user's program holds the name.
pub fn forward_hook_event_through_pipe(pipe_name: &str, current_user_sid: &str, hook_event_json: &[u8]) -> Option<Vec<u8>> {
    let mut crest_pipe = open_crest_pipe(pipe_name)?;
    // Tool inputs can contain file contents and commands; they only go to our own user's Crest.
    let mut server_process_id = 0u32;
    // SAFETY: the handle belongs to the open pipe for the whole call.
    unsafe { GetNamedPipeServerProcessId(HANDLE(crest_pipe.as_raw_handle()), &mut server_process_id) }.ok()?;
    if read_user_sid_of_process(server_process_id).ok()? != current_user_sid {
        return None;
    }
    write_length_prefixed_message(&mut crest_pipe, hook_event_json).ok()?;
    read_length_prefixed_message(&mut crest_pipe).ok()
}

fn open_crest_pipe(pipe_name: &str) -> Option<File> {
    for _ in 0..CLAUDE_CODE_HOOK_PIPE_BUSY_RETRY_COUNT {
        match OpenOptions::new().read(true).write(true).open(pipe_name) {
            Ok(crest_pipe) => return Some(crest_pipe),
            Err(open_error) if open_error.raw_os_error() == Some(ERROR_PIPE_BUSY.0 as i32) => {
                thread::sleep(CLAUDE_CODE_HOOK_PIPE_BUSY_RETRY_DELAY)
            }
            Err(_) => return None,
        }
    }
    None
}
