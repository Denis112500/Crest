// The named-pipe side (option B): one pipe instance per hook process, each answered on its
// own thread so a PermissionRequest waiting for its delay doesn't hold up status events.
//
// Wire format, both little-endian: hook → listener = u64 milliseconds since the hook process
// started, u32 length, event JSON; listener → hook = u32 length, reply JSON (may be empty).

use std::fs::File;
use std::io::{Read, Write};
use std::os::windows::io::FromRawHandle;
use std::thread;

use windows::core::HSTRING;
use windows::Win32::Foundation::{CloseHandle, ERROR_PIPE_CONNECTED};
use windows::Win32::Storage::FileSystem::PIPE_ACCESS_DUPLEX;
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE,
    PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};

use crate::hook_event_fields::summarize_hook_event;
use crate::permission_reply::build_reply_to_hook_event;
use crate::probe_log_line::print_probe_line;
use crate::user_only_pipe_security::UserOnlyPipeSecurity;
use crate::ListenOptions;

pub const PROBE_PIPE_NAME: &str = r"\\.\pipe\crest-claude-code-hook-probe";
const PIPE_BUFFER_BYTES: u32 = 64 * 1024;
const LARGEST_ACCEPTED_EVENT_BYTES: usize = 4 * 1024 * 1024;

pub fn run_pipe_listener(listen_options: ListenOptions) -> Result<(), String> {
    let pipe_security = UserOnlyPipeSecurity::for_current_user()?;
    print_probe_line(
        "pipe",
        &format!("listening on {PROBE_PIPE_NAME}, only user {} may open it", pipe_security.current_user_sid),
    );
    let pipe_name = HSTRING::from(PROBE_PIPE_NAME);
    loop {
        let pipe_instance = unsafe {
            CreateNamedPipeW(
                &pipe_name,
                PIPE_ACCESS_DUPLEX,
                PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
                PIPE_UNLIMITED_INSTANCES,
                PIPE_BUFFER_BYTES,
                PIPE_BUFFER_BYTES,
                0,
                Some(pipe_security.attributes()),
            )
        };
        if pipe_instance.is_invalid() {
            return Err(format!("creating the pipe failed: {}", windows::core::Error::from_thread()));
        }
        // ERROR_PIPE_CONNECTED: the hook connected between create and connect, which is fine.
        if let Err(connect_error) = unsafe { ConnectNamedPipe(pipe_instance, None) } {
            if connect_error.code() != ERROR_PIPE_CONNECTED.to_hresult() {
                print_probe_line("pipe", &format!("a connection failed: {connect_error}"));
                let _ = unsafe { CloseHandle(pipe_instance) };
                continue;
            }
        }
        let connected_pipe = unsafe { File::from_raw_handle(pipe_instance.0) };
        thread::spawn(move || {
            if let Err(problem) = answer_hook_process(connected_pipe, listen_options) {
                print_probe_line("pipe", &format!("a hook process went away mid-exchange: {problem}"));
            }
        });
    }
}

fn answer_hook_process(mut connected_pipe: File, listen_options: ListenOptions) -> std::io::Result<()> {
    let mut header_bytes = [0u8; 12];
    connected_pipe.read_exact(&mut header_bytes)?;
    let milliseconds_since_hook_started = u64::from_le_bytes(header_bytes[..8].try_into().unwrap());
    let event_length = u32::from_le_bytes(header_bytes[8..].try_into().unwrap()) as usize;
    if event_length > LARGEST_ACCEPTED_EVENT_BYTES {
        return Err(std::io::Error::other(format!("event of {event_length} bytes refused")));
    }
    let mut event_bytes = vec![0u8; event_length];
    connected_pipe.read_exact(&mut event_bytes)?;
    let hook_event_json = String::from_utf8_lossy(&event_bytes);
    let hook_event_summary = summarize_hook_event(&hook_event_json);
    print_probe_line(
        "pipe",
        &format!(
            "{}  {}  (hook process started {milliseconds_since_hook_started} ms before)",
            hook_event_summary.hook_event_name, hook_event_summary.details
        ),
    );
    let reply_json = build_reply_to_hook_event(
        &hook_event_summary.hook_event_name,
        listen_options.permission_answer,
        listen_options.answer_delay,
    );
    connected_pipe.write_all(&(reply_json.len() as u32).to_le_bytes())?;
    connected_pipe.write_all(reply_json.as_bytes())?;
    if hook_event_summary.hook_event_name == "PermissionRequest" {
        print_probe_line("pipe", &format!("answered PermissionRequest: {}", listen_options.permission_answer.label()));
    }
    // Waiting for the hook to close its end makes sure it read the reply before this end closes.
    let _ = connected_pipe.read_to_end(&mut Vec::new());
    Ok(())
}
