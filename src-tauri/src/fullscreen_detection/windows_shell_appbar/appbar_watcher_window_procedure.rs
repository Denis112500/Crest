//! Handles the messages Windows sends the appbar: fullscreen opened/closed, the short settle
//! timer, Explorer restarting (which forgets every appbar), and Crest's own "check again".

use std::ffi::c_void;

use windows::Win32::Foundation::{E_FAIL, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Shell::{SHAppBarMessage, ABM_NEW, ABN_FULLSCREENAPP, APPBARDATA};
use windows::Win32::UI::WindowsAndMessaging::{DefWindowProcW, KillTimer, SetTimer, WM_APP, WM_TIMER};

use crate::backend_constants::FULLSCREEN_APP_LEAVE_SETTLE_DELAY;
use crate::fullscreen_detection::windows_shell_appbar::appbar_watcher_thread_state::{
    with_appbar_watcher_thread_state, AppbarWatcherThreadState, APPBAR_WATCHER_THREAD_STATE,
};

/// Our own number for the shell's appbar notifications (WM_APP and up are free for private use).
const APPBAR_NOTIFICATION_MESSAGE: u32 = WM_APP + 1;
/// Posted by `FullscreenAppRecheckTrigger` from other threads (e.g. after the pill moved).
pub const FULLSCREEN_RECHECK_REQUEST_MESSAGE: u32 = WM_APP + 2;
const FULLSCREEN_LEAVE_SETTLE_TIMER_IDENTIFIER: usize = 1;

/// An appbar is normally a docked toolbar; without a following ABM_SETPOS it reserves no
/// screen space and only receives the shell's notifications.
pub fn register_as_appbar(watcher_window: HWND) -> windows::core::Result<()> {
    let mut appbar_registration = APPBARDATA {
        cbSize: size_of::<APPBARDATA>() as u32,
        hWnd: watcher_window,
        uCallbackMessage: APPBAR_NOTIFICATION_MESSAGE,
        ..Default::default()
    };
    // SAFETY: the structure lives for the whole call and describes our own window.
    if unsafe { SHAppBarMessage(ABM_NEW, &mut appbar_registration) } == 0 {
        return Err(windows::core::Error::new(E_FAIL, "the shell refused the appbar registration"));
    }
    Ok(())
}

/// Shared by the shell's "opened"/"closed" and Crest's own front-window checks: "in front"
/// takes effect at once, "gone" only after a short settle delay.
fn react_to_fullscreen_signal(watcher_window: HWND, is_fullscreen_app_signalled: bool) {
    // SAFETY: our own window, used on the thread that created it.
    unsafe {
        if is_fullscreen_app_signalled {
            let _ = KillTimer(Some(watcher_window), FULLSCREEN_LEAVE_SETTLE_TIMER_IDENTIFIER);
            with_appbar_watcher_thread_state(AppbarWatcherThreadState::check_front_window_and_report);
        } else {
            // A game switching display mode sends "closed" and "opened" 20 ms apart, and a game
            // coming back to the front is 1 pixel short of its monitor for ~20 ms (measured);
            // waiting a moment keeps the pill from flashing in between. A new timer replaces an
            // old one, and it checks the front window again when it fires.
            SetTimer(
                Some(watcher_window),
                FULLSCREEN_LEAVE_SETTLE_TIMER_IDENTIFIER,
                FULLSCREEN_APP_LEAVE_SETTLE_DELAY.as_millis() as u32,
                None,
            );
        }
    }
}

/// The front window changed while a fullscreen app is reported (see `front_window_change_hook.rs`).
pub fn recheck_after_front_window_change() {
    let mut watcher_window_and_answer = None;
    with_appbar_watcher_thread_state(|watcher_thread_state| {
        watcher_window_and_answer = Some((
            HWND(watcher_thread_state.watcher_window_handle_value as *mut c_void),
            watcher_thread_state.is_fullscreen_app_in_front_now(),
        ));
    });
    if let Some((watcher_window, is_fullscreen_app_in_front)) = watcher_window_and_answer {
        react_to_fullscreen_signal(watcher_window, is_fullscreen_app_in_front);
    }
}

pub unsafe extern "system" fn appbar_watcher_window_procedure(
    watcher_window: HWND,
    window_message: u32,
    message_wparam: WPARAM,
    message_lparam: LPARAM,
) -> LRESULT {
    if window_message == APPBAR_NOTIFICATION_MESSAGE {
        if message_wparam.0 as u32 == ABN_FULLSCREENAPP {
            react_to_fullscreen_signal(watcher_window, message_lparam.0 != 0);
        }
        return LRESULT(0);
    }
    if window_message == FULLSCREEN_RECHECK_REQUEST_MESSAGE {
        with_appbar_watcher_thread_state(AppbarWatcherThreadState::check_front_window_and_report);
        return LRESULT(0);
    }
    if window_message == WM_TIMER && message_wparam.0 == FULLSCREEN_LEAVE_SETTLE_TIMER_IDENTIFIER {
        let _ = KillTimer(Some(watcher_window), FULLSCREEN_LEAVE_SETTLE_TIMER_IDENTIFIER);
        with_appbar_watcher_thread_state(AppbarWatcherThreadState::check_front_window_and_report);
        return LRESULT(0);
    }
    let taskbar_created_message = APPBAR_WATCHER_THREAD_STATE
        .with_borrow(|watcher_thread_state| watcher_thread_state.as_ref().map_or(0, |state| state.taskbar_created_message));
    if taskbar_created_message != 0 && window_message == taskbar_created_message {
        // Explorer restarted (a crash or an update) and forgot every appbar.
        if let Err(registration_error) = register_as_appbar(watcher_window) {
            eprintln!("Crest: could not watch fullscreen apps again after Explorer restarted: {registration_error}");
        }
        with_appbar_watcher_thread_state(AppbarWatcherThreadState::check_front_window_and_report);
        return LRESULT(0);
    }
    DefWindowProcW(watcher_window, window_message, message_wparam, message_lparam)
}
