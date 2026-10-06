//! Starts the watcher's own thread, creates the invisible appbar window and runs its message
//! loop, which sleeps until Windows has something to say.

use std::thread;

use tauri::WebviewWindow;
use windows::core::w;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DispatchMessageW, GetMessageW, RegisterClassW, RegisterWindowMessageW, TranslateMessage, MSG,
    WNDCLASSW, WS_EX_TOOLWINDOW, WS_POPUP,
};

use crate::fullscreen_detection::fullscreen_app_watcher::{FullscreenAppChangeListener, FullscreenAppWatcher};
use crate::fullscreen_detection::windows_shell_appbar::appbar_watcher_thread_state::{
    with_appbar_watcher_thread_state, AppbarWatcherThreadState, APPBAR_WATCHER_THREAD_STATE,
};
use crate::fullscreen_detection::windows_shell_appbar::appbar_watcher_window_procedure::{
    appbar_watcher_window_procedure, register_as_appbar,
};

const FULLSCREEN_WATCHER_THREAD_NAME: &str = "crest-fullscreen-watcher";

/// Uses the shell's own fullscreen detection (the one that hides the taskbar): an appbar
/// receives ABN_FULLSCREENAPP when a fullscreen app opens and when it closes. Measured with
/// `dev-tools/fullscreen_event_watcher` (notes.md, 2026-10-02).
pub struct AppbarFullscreenAppWatcher;

impl FullscreenAppWatcher for AppbarFullscreenAppWatcher {
    fn start_watching_fullscreen_apps(
        pill_window: &WebviewWindow,
        fullscreen_app_change_listener: FullscreenAppChangeListener,
    ) -> Result<(), String> {
        let pill_window_handle_value = pill_window.hwnd().map_err(|error| error.to_string())?.0 as isize;
        thread::Builder::new()
            .name(FULLSCREEN_WATCHER_THREAD_NAME.to_string())
            .spawn(move || {
                if let Err(watch_error) = run_appbar_watcher_thread(pill_window_handle_value, fullscreen_app_change_listener)
                {
                    eprintln!("Crest: fullscreen apps can't be detected, the pill stays over them: {watch_error}");
                }
            })
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}

fn run_appbar_watcher_thread(
    pill_window_handle_value: isize,
    fullscreen_app_change_listener: FullscreenAppChangeListener,
) -> windows::core::Result<()> {
    // SAFETY: the watcher window is created and used only on this thread, which lives as
    // long as the app; the message structure outlives every call that uses it.
    unsafe {
        let crest_program = GetModuleHandleW(None)?;
        let watcher_window_class_name = w!("CrestFullscreenAppWatcher");
        let watcher_window_class = WNDCLASSW {
            lpfnWndProc: Some(appbar_watcher_window_procedure),
            hInstance: crest_program.into(),
            lpszClassName: watcher_window_class_name,
            ..Default::default()
        };
        if RegisterClassW(&watcher_window_class) == 0 {
            return Err(windows::core::Error::from_thread());
        }
        // Never shown. A real top-level window, because message-only windows don't receive
        // Explorer's TaskbarCreated broadcast.
        let watcher_window = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            watcher_window_class_name,
            w!("Crest fullscreen app watcher"),
            WS_POPUP,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(crest_program.into()),
            None,
        )?;
        APPBAR_WATCHER_THREAD_STATE.set(Some(AppbarWatcherThreadState {
            pill_window_handle_value,
            taskbar_created_message: RegisterWindowMessageW(w!("TaskbarCreated")),
            last_reported_fullscreen_state: None,
            fullscreen_app_change_listener,
        }));
        register_as_appbar(watcher_window)?;
        // The shell only reports changes, and a game may already be in front at startup.
        with_appbar_watcher_thread_state(AppbarWatcherThreadState::check_front_window_and_report);

        // GetMessageW sleeps until Windows has something for this thread: no CPU meanwhile.
        let mut queued_message = MSG::default();
        while GetMessageW(&mut queued_message, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&queued_message);
            DispatchMessageW(&queued_message);
        }
    }
    Ok(())
}
