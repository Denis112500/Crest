//! Crest's end of the pipe: a thread that waits for a hook to connect (sleeping, no CPU), reads
//! its event, hands it over with the connection as a reply to send later, and waits for the
//! next one at once. A permission request can wait minutes for a click; every other hook keeps
//! getting through meanwhile, each on its own pipe instance. A hook that finds the pipe busy
//! between two connections retries.

use std::fs::{File, OpenOptions};
use std::os::windows::io::FromRawHandle;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::thread::{self, JoinHandle};

use windows::core::HSTRING;
use windows::Win32::Foundation::{CloseHandle, ERROR_PIPE_CONNECTED, HANDLE};
use windows::Win32::Storage::FileSystem::{FILE_FLAG_FIRST_PIPE_INSTANCE, PIPE_ACCESS_DUPLEX};
use windows::Win32::System::Pipes::{
    ConnectNamedPipe, CreateNamedPipeW, PIPE_READMODE_BYTE, PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_BYTE,
    PIPE_UNLIMITED_INSTANCES, PIPE_WAIT,
};

use crate::backend_constants::{
    CLAUDE_CODE_HOOK_PIPE_BUFFER_BYTES, CLAUDE_CODE_HOOK_PIPE_BUSY_RETRY_COUNT, CLAUDE_CODE_HOOK_PIPE_BUSY_RETRY_DELAY,
};
use crate::claude_code_hook_channel::length_prefixed_message::read_length_prefixed_message;
use crate::claude_code_hook_channel::windows_named_pipe::named_pipe_pending_hook_reply::NamedPipePendingHookReply;
use crate::claude_code_hook_channel::windows_named_pipe::windows_user_identity::UserOnlyPipeAccess;
use crate::claude_code_hook_channel::{HookEventHandler, RunningHookEventServer};

const HOOK_EVENT_SERVER_THREAD_NAME: &str = "crest-claude-code-hooks";

pub struct NamedPipeHookEventServer {
    pipe_name: String,
    stop_requested: Arc<AtomicBool>,
    server_thread: Option<JoinHandle<()>>,
}

impl NamedPipeHookEventServer {
    /// Returns once the pipe exists, or with the reason it couldn't be created (e.g. another
    /// program already holds the name), so switching the integration on fails visibly.
    pub fn start(pipe_name: String, current_user_sid: String, hook_event_handler: HookEventHandler) -> Result<Self, String> {
        let stop_requested = Arc::new(AtomicBool::new(false));
        let (startup_result_sender, startup_result_receiver) = mpsc::channel();
        let thread_pipe_name = pipe_name.clone();
        let thread_stop_requested = Arc::clone(&stop_requested);
        let server_thread = thread::Builder::new()
            .name(HOOK_EVENT_SERVER_THREAD_NAME.to_string())
            .spawn(move || {
                serve_hook_events(&thread_pipe_name, &current_user_sid, &thread_stop_requested, hook_event_handler, startup_result_sender)
            })
            .map_err(|error| error.to_string())?;
        startup_result_receiver.recv().map_err(|error| error.to_string())??;
        Ok(Self { pipe_name, stop_requested, server_thread: Some(server_thread) })
    }
}

impl RunningHookEventServer for NamedPipeHookEventServer {
    /// The thread sleeps inside ConnectNamedPipe, so after raising the flag Crest connects to
    /// its own pipe once to wake it up.
    fn stop(mut self: Box<Self>) {
        self.stop_requested.store(true, Ordering::SeqCst);
        for _ in 0..CLAUDE_CODE_HOOK_PIPE_BUSY_RETRY_COUNT {
            if OpenOptions::new().read(true).write(true).open(&self.pipe_name).is_ok() {
                break;
            }
            thread::sleep(CLAUDE_CODE_HOOK_PIPE_BUSY_RETRY_DELAY);
        }
        if let Some(server_thread) = self.server_thread.take() {
            let _ = server_thread.join();
        }
    }
}

fn serve_hook_events(
    pipe_name: &str,
    current_user_sid: &str,
    stop_requested: &AtomicBool,
    hook_event_handler: HookEventHandler,
    startup_result_sender: mpsc::Sender<Result<(), String>>,
) {
    let pipe_access = match UserOnlyPipeAccess::for_user(current_user_sid) {
        Ok(pipe_access) => pipe_access,
        Err(access_error) => {
            let _ = startup_result_sender.send(Err(access_error));
            return;
        }
    };
    let mut is_first_instance = true;
    loop {
        let pipe_instance = match create_pipe_instance(pipe_name, &pipe_access, is_first_instance) {
            Ok(pipe_instance) => pipe_instance,
            Err(creation_error) => {
                if is_first_instance {
                    let _ = startup_result_sender.send(Err(creation_error));
                } else {
                    eprintln!("Crest: Claude Code's events stopped arriving: {creation_error}");
                }
                return;
            }
        };
        if is_first_instance {
            let _ = startup_result_sender.send(Ok(()));
            is_first_instance = false;
        }
        // SAFETY: our own pipe handle; ERROR_PIPE_CONNECTED means the hook was quicker than us.
        if let Err(connect_error) = unsafe { ConnectNamedPipe(pipe_instance, None) } {
            if connect_error.code() != ERROR_PIPE_CONNECTED.to_hresult() {
                let _ = unsafe { CloseHandle(pipe_instance) };
                continue;
            }
        }
        if stop_requested.load(Ordering::SeqCst) {
            let _ = unsafe { CloseHandle(pipe_instance) };
            return;
        }
        // SAFETY: the handle is ours alone from here on; the File closes it.
        let mut connected_pipe = unsafe { File::from_raw_handle(pipe_instance.0) };
        match read_length_prefixed_message(&mut connected_pipe) {
            Ok(hook_event_bytes) => hook_event_handler(
                &String::from_utf8_lossy(&hook_event_bytes),
                Box::new(NamedPipePendingHookReply::new(connected_pipe)),
            ),
            Err(read_error) => eprintln!("Crest: a Claude Code hook went away mid-exchange: {read_error}"),
        }
    }
}

fn create_pipe_instance(pipe_name: &str, pipe_access: &UserOnlyPipeAccess, is_first_instance: bool) -> Result<HANDLE, String> {
    // The first instance must be new: if a program already holds the name, it could be posing
    // as Crest, so Crest refuses instead of sharing the name with it.
    let open_mode = if is_first_instance { PIPE_ACCESS_DUPLEX | FILE_FLAG_FIRST_PIPE_INSTANCE } else { PIPE_ACCESS_DUPLEX };
    // SAFETY: the name and the access rule outlive the call.
    let pipe_instance = unsafe {
        CreateNamedPipeW(
            &HSTRING::from(pipe_name),
            open_mode,
            PIPE_TYPE_BYTE | PIPE_READMODE_BYTE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
            PIPE_UNLIMITED_INSTANCES,
            CLAUDE_CODE_HOOK_PIPE_BUFFER_BYTES,
            CLAUDE_CODE_HOOK_PIPE_BUFFER_BYTES,
            0,
            Some(pipe_access.security_attributes()),
        )
    };
    if pipe_instance.is_invalid() {
        return Err(format!("creating the pipe failed: {}", windows::core::Error::from_thread()));
    }
    Ok(pipe_instance)
}
