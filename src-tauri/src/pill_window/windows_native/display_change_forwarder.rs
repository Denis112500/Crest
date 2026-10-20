// Windows sends WM_DISPLAYCHANGE to every top-level window when a monitor's resolution or the
// monitor layout changes: a monitor plugged in or unplugged, or the main monitor switching to
// its real resolution shortly after login (the pill was then placed for a 1920-wide screen on
// a 2560-wide one, 2026-10-20). A second subclass on the pill window passes the news on, so the
// pill is placed again from the monitors as they are now.

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::WM_DISPLAYCHANGE;

use crate::pill_window::pill_window_platform::DisplayChangeListener;

/// Tells this subclass apart from the frame-painting blocker on the same window.
const DISPLAY_CHANGE_FORWARDER_SUBCLASS_ID: usize = 2;

pub fn forward_display_changes(
    pill_window_native_handle: HWND,
    display_change_listener: DisplayChangeListener,
) -> Result<(), String> {
    // Handed to Windows as the subclass's reference data and never freed: the subclass lasts as
    // long as the pill window, which lasts as long as Crest.
    let listener_address = Box::into_raw(Box::new(display_change_listener)) as usize;
    // SAFETY: called on the thread that created the window (Tauri's setup), as subclassing
    // requires; the procedure below lives for the whole program.
    let was_subclassed = unsafe {
        SetWindowSubclass(
            pill_window_native_handle,
            Some(pass_display_change_on),
            DISPLAY_CHANGE_FORWARDER_SUBCLASS_ID,
            listener_address,
        )
    };
    if was_subclassed.as_bool() {
        return Ok(());
    }
    // SAFETY: Windows refused the subclass, so nothing else holds the address made above.
    drop(unsafe { Box::from_raw(listener_address as *mut DisplayChangeListener) });
    Err("Windows refused to hook the pill window's display changes".to_string())
}

unsafe extern "system" fn pass_display_change_on(
    window: HWND,
    message: u32,
    message_word_parameter: WPARAM,
    message_long_parameter: LPARAM,
    _subclass_id: usize,
    subclass_reference_data: usize,
) -> LRESULT {
    if message == WM_DISPLAYCHANGE {
        // SAFETY: the address set in `forward_display_changes`, which is never freed.
        let display_change_listener = &*(subclass_reference_data as *const DisplayChangeListener);
        display_change_listener();
    }
    DefSubclassProc(window, message, message_word_parameter, message_long_parameter)
}
