//! What the window procedure needs to remember. A window procedure is a plain function
//! Windows calls, with no `self`, so its state lives in a thread-local variable.

use std::cell::RefCell;
use std::ffi::c_void;

use windows::Win32::Foundation::HWND;

use crate::fullscreen_detection::fullscreen_app_watcher::FullscreenAppChangeListener;
use crate::fullscreen_detection::windows_shell_appbar::front_window_fullscreen_check::{
    locate_front_window, FrontWindowPlacement,
};

pub struct AppbarWatcherThreadState {
    /// The pill window's handle as a plain number: Rust doesn't let raw handles cross
    /// threads, and this thread only uses it to ask which monitor the pill is on.
    pub pill_window_handle_value: isize,
    /// The watcher's own window, which owns the settle timer.
    pub watcher_window_handle_value: isize,
    /// Explorer broadcasts this after it restarts; its number is assigned at runtime.
    pub taskbar_created_message: u32,
    pub last_reported_fullscreen_state: Option<bool>,
    pub fullscreen_app_change_listener: FullscreenAppChangeListener,
    /// The running settle timer was started by the shell's "closed", which outranks a mere
    /// front-window change: then a front window on another monitor means "not fullscreen".
    pub is_settle_check_after_shell_closed: bool,
}

impl AppbarWatcherThreadState {
    pub fn locate_front_window_now(&self) -> FrontWindowPlacement {
        locate_front_window(HWND(self.pill_window_handle_value as *mut c_void))
    }

    /// For the shell's signals, startup, Explorer restarts and "the pill moved": the front window
    /// decides alone, and one on another monitor means "not fullscreen".
    pub fn check_front_window_and_report(&mut self) {
        let front_window_placement = self.locate_front_window_now();
        self.report_fullscreen_state(front_window_placement == FrontWindowPlacement::CoversPillMonitor);
    }

    /// For a front-window change only: one on another monitor leaves the answer as it is, since
    /// a game on the pill's monitor is still on screen (alt-tab to the second monitor and back
    /// sends no shell signal, 2026-10-19).
    pub fn check_front_window_after_change_and_report(&mut self) {
        match self.locate_front_window_now() {
            FrontWindowPlacement::CoversPillMonitor => self.report_fullscreen_state(true),
            FrontWindowPlacement::LeavesPillMonitorUncovered => self.report_fullscreen_state(false),
            FrontWindowPlacement::OnAnotherMonitor => {}
        }
    }

    pub fn report_fullscreen_state(&mut self, is_fullscreen_app_in_front: bool) {
        if self.last_reported_fullscreen_state != Some(is_fullscreen_app_in_front) {
            self.last_reported_fullscreen_state = Some(is_fullscreen_app_in_front);
            (self.fullscreen_app_change_listener)(is_fullscreen_app_in_front);
        }
    }
}

thread_local! {
    /// Windows calls the window procedure as a plain function on the watcher thread; this
    /// is how that function reaches the thread's state.
    pub static APPBAR_WATCHER_THREAD_STATE: RefCell<Option<AppbarWatcherThreadState>> = const { RefCell::new(None) };
}

pub fn with_appbar_watcher_thread_state(state_action: impl FnOnce(&mut AppbarWatcherThreadState)) {
    APPBAR_WATCHER_THREAD_STATE.with_borrow_mut(|watcher_thread_state| {
        if let Some(watcher_thread_state) = watcher_thread_state.as_mut() {
            state_action(watcher_thread_state);
        }
    });
}
