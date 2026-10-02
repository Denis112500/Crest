use std::cell::RefCell;
use std::ffi::c_void;

use windows::Win32::Foundation::HWND;

use crate::fullscreen_detection::fullscreen_app_watcher::FullscreenAppChangeListener;
use crate::fullscreen_detection::windows_shell_appbar::front_window_fullscreen_check::is_fullscreen_app_in_front_of_pill;

pub struct AppbarWatcherThreadState {
    /// The pill window's handle as a plain number: Rust doesn't let raw handles cross
    /// threads, and this thread only uses it to ask which monitor the pill is on.
    pub pill_window_handle_value: isize,
    /// Explorer broadcasts this after it restarts; its number is assigned at runtime.
    pub taskbar_created_message: u32,
    pub last_reported_fullscreen_state: Option<bool>,
    pub fullscreen_app_change_listener: FullscreenAppChangeListener,
}

impl AppbarWatcherThreadState {
    pub fn check_front_window_and_report(&mut self) {
        let pill_window = HWND(self.pill_window_handle_value as *mut c_void);
        self.report_fullscreen_state(is_fullscreen_app_in_front_of_pill(pill_window));
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
