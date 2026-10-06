//! Windows: `FindFirstChangeNotificationW` gives a handle that becomes signaled when a file in the
//! folder changes size or is written; one thread waits on it and on a "stop" event, sleeping
//! in between.

use std::path::Path;
use std::thread::{self, JoinHandle};

use windows::core::HSTRING;
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::Storage::FileSystem::{
    FindCloseChangeNotification, FindFirstChangeNotificationW, FindNextChangeNotification, FILE_NOTIFY_CHANGE_LAST_WRITE,
    FILE_NOTIFY_CHANGE_SIZE,
};
use windows::Win32::System::Threading::{CreateEventW, SetEvent, WaitForMultipleObjects, INFINITE};

use crate::folder_change_watching::FolderChangeWatch;

const FOLDER_WATCH_THREAD_NAME: &str = "crest-folder-watch";

pub struct ChangeNotificationFolderWatch {
    /// Handles as plain numbers: Rust doesn't let raw handles cross threads.
    stop_event_handle_value: isize,
    watch_thread: Option<JoinHandle<()>>,
}

impl FolderChangeWatch for ChangeNotificationFolderWatch {
    fn start_watching(watched_folder: &Path, on_folder_change: Box<dyn Fn() + Send>) -> Result<Self, String> {
        // Created here rather than on the thread, so a missing folder fails at once.
        // SAFETY: both handles are closed by the watch thread when it ends.
        let change_handle = unsafe {
            FindFirstChangeNotificationW(
                &HSTRING::from(watched_folder),
                false,
                FILE_NOTIFY_CHANGE_SIZE | FILE_NOTIFY_CHANGE_LAST_WRITE,
            )
        }
        .map_err(|error| format!("watching {} failed: {error}", watched_folder.display()))?;
        let stop_event = match unsafe { CreateEventW(None, true, false, None) } {
            Ok(stop_event) => stop_event,
            Err(event_error) => {
                let _ = unsafe { FindCloseChangeNotification(change_handle) };
                return Err(event_error.to_string());
            }
        };
        let (change_handle_value, stop_event_handle_value) = (change_handle.0 as isize, stop_event.0 as isize);
        let watch_thread = thread::Builder::new()
            .name(FOLDER_WATCH_THREAD_NAME.to_string())
            .spawn(move || wait_for_folder_changes(change_handle_value, stop_event_handle_value, on_folder_change))
            .map_err(|error| error.to_string())?;
        Ok(Self { stop_event_handle_value, watch_thread: Some(watch_thread) })
    }
}

impl Drop for ChangeNotificationFolderWatch {
    fn drop(&mut self) {
        // SAFETY: the event stays open until the watch thread closes it after waking up.
        let _ = unsafe { SetEvent(HANDLE(self.stop_event_handle_value as *mut _)) };
        if let Some(watch_thread) = self.watch_thread.take() {
            let _ = watch_thread.join();
        }
    }
}

fn wait_for_folder_changes(change_handle_value: isize, stop_event_handle_value: isize, on_folder_change: Box<dyn Fn() + Send>) {
    let change_handle = HANDLE(change_handle_value as *mut _);
    let stop_event = HANDLE(stop_event_handle_value as *mut _);
    loop {
        // SAFETY: both handles stay open for the whole loop.
        let signaled_handle = unsafe { WaitForMultipleObjects(&[change_handle, stop_event], false, INFINITE) };
        if signaled_handle != WAIT_OBJECT_0 {
            break;
        }
        on_folder_change();
        if unsafe { FindNextChangeNotification(change_handle) }.is_err() {
            break;
        }
    }
    // SAFETY: each handle is closed once, here, after the last wait.
    unsafe {
        let _ = FindCloseChangeNotification(change_handle);
        let _ = CloseHandle(stop_event);
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::mpsc;
    use std::time::Duration;

    use super::*;

    #[test]
    fn writing_a_file_in_the_folder_is_reported_and_dropping_the_watch_ends_it() {
        let watched_folder = std::env::temp_dir().join(format!("crest-folder-watch-test-{}", std::process::id()));
        std::fs::create_dir_all(&watched_folder).unwrap();
        let transcript_path = watched_folder.join("session.jsonl");
        std::fs::write(&transcript_path, "first line\n").unwrap();
        let (change_sender, change_receiver) = mpsc::channel();
        let folder_watch = ChangeNotificationFolderWatch::start_watching(
            &watched_folder,
            Box::new(move || {
                let _ = change_sender.send(());
            }),
        )
        .unwrap();
        let mut transcript_file = std::fs::OpenOptions::new().append(true).open(&transcript_path).unwrap();
        transcript_file.write_all(b"second line\n").unwrap();
        transcript_file.flush().unwrap();
        drop(transcript_file);
        assert!(change_receiver.recv_timeout(Duration::from_secs(2)).is_ok());
        drop(folder_watch);
        std::fs::remove_dir_all(&watched_folder).unwrap();
    }
}
