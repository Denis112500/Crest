// Rule for the pill window on Windows: never call Tauri's `show()`, `hide()` or any
// setter that changes a window flag (always-on-top, ignore-cursor, resizable, ...).
// Tauri's window layer (tao 0.37) rewrites the whole extended style from its own
// flags on every such change, which would erase WS_EX_TOOLWINDOW, and its `show()`
// uses SW_SHOW, which activates the window and steals focus (tao PR #1358, unmerged).

use tauri::WebviewWindow;
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetWindowLongPtrW, ShowWindow, GWL_EXSTYLE, SW_SHOWNOACTIVATE,
    WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};

use crate::pill_window::pill_window_platform::PillWindowPlatform;

pub struct WindowsPillWindowPlatform;

impl PillWindowPlatform for WindowsPillWindowPlatform {
    fn prepare_pill_window_as_overlay(pill_window: &WebviewWindow) -> Result<(), String> {
        let pill_window_native_handle = pill_window.hwnd().map_err(|error| error.to_string())?;
        // WS_EX_TOOLWINDOW keeps the window out of both the taskbar and Alt+Tab, which
        // Tauri's `skipTaskbar` doesn't reliably do (tauri#10422). WS_EX_APPWINDOW would
        // force a taskbar button back, so it's removed. WS_EX_NOACTIVATE is already set
        // by `focusable: false`; it's repeated so this function alone guarantees the result.
        let extended_style_bits_to_add = (WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0) as isize;
        let extended_style_bits_to_remove = WS_EX_APPWINDOW.0 as isize;
        // SAFETY: the handle belongs to a live window owned by this process, and Tauri
        // runs setup and synchronous commands on the thread that created the window.
        unsafe {
            let current_extended_style = GetWindowLongPtrW(pill_window_native_handle, GWL_EXSTYLE);
            let overlay_extended_style =
                (current_extended_style | extended_style_bits_to_add) & !extended_style_bits_to_remove;
            SetWindowLongPtrW(pill_window_native_handle, GWL_EXSTYLE, overlay_extended_style);
        }
        Ok(())
    }

    fn show_pill_window_without_activating(pill_window: &WebviewWindow) -> Result<(), String> {
        let pill_window_native_handle = pill_window.hwnd().map_err(|error| error.to_string())?;
        // SAFETY: same live, same-thread window handle as above. The return value only
        // says whether the window was visible before, so there is no error to handle.
        unsafe {
            let _ = ShowWindow(pill_window_native_handle, SW_SHOWNOACTIVATE);
        }
        Ok(())
    }
}
