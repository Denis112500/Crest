//! Win32 for the pill window: tool-window styles (no taskbar, no Alt+Tab), showing without
//! focus, hiding, the window region that decides which part takes the mouse, and letting every
//! click through while the pill is closed.

// Rule for the pill window on Windows: never call Tauri's `show()`, `hide()` or any
// setter that changes a window flag (always-on-top, ignore-cursor, resizable, ...).
// Tauri's window layer (tao 0.37) rewrites the whole extended style from its own
// flags on every such change, which would erase WS_EX_TOOLWINDOW, and its `show()`
// uses SW_SHOW, which activates the window and steals focus (tao PR #1358, unmerged).

use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow};
use windows::Win32::Foundation::COLORREF;
use windows::Win32::Graphics::Gdi::{CreateRectRgn, DeleteObject, SetWindowRgn};
use windows::Win32::UI::WindowsAndMessaging::{
    GetWindowLongPtrW, SetLayeredWindowAttributes, SetWindowLongPtrW, SetWindowPos, ShowWindow, GWL_EXSTYLE,
    GWL_STYLE, LWA_ALPHA, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SW_HIDE,
    SW_SHOWNOACTIVATE, WS_CAPTION, WS_EX_APPWINDOW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TRANSPARENT, WS_SYSMENU,
};

use crate::pill_window::pill_window_platform::{DisplayChangeListener, PillWindowPlatform};
use crate::pill_window::windows_native::classic_frame_painting_blocker::block_classic_frame_painting;
use crate::pill_window::windows_native::display_change_forwarder::forward_display_changes;
use crate::pill_window::windows_native::webview_memory_usage_target::adjust_pill_webview_memory_target;

pub struct WindowsPillWindowPlatform;

impl PillWindowPlatform for WindowsPillWindowPlatform {
    fn prepare_pill_window_as_overlay(pill_window: &WebviewWindow) -> Result<(), String> {
        let pill_window_native_handle = pill_window.hwnd().map_err(|error| error.to_string())?;
        // WS_EX_TOOLWINDOW keeps the window out of both the taskbar and Alt+Tab, which
        // Tauri's `skipTaskbar` doesn't reliably do (tauri#10422). WS_EX_APPWINDOW would
        // force a taskbar button back, so it's removed. WS_EX_NOACTIVATE is already set
        // by `focusable: false`; it's repeated so this function alone guarantees the result.
        // WS_EX_LAYERED stays on for good: Windows lets clicks through a window only when it is
        // layered and WS_EX_TRANSPARENT (toggled in `set_pill_window_interactive_area`), and
        // switching layering itself on and off would rebuild the window's drawing surface.
        let extended_style_bits_to_add = (WS_EX_TOOLWINDOW.0 | WS_EX_NOACTIVATE.0 | WS_EX_LAYERED.0) as isize;
        let extended_style_bits_to_remove = WS_EX_APPWINDOW.0 as isize;
        // Tauri hides the title bar of an undecorated window by giving its frame zero size,
        // but leaves the caption styles set. Removing them leaves Windows less frame to
        // paint; the repaints it still attempts are blocked by `block_classic_frame_painting`.
        let window_style_bits_to_remove = (WS_CAPTION.0 | WS_SYSMENU.0) as isize;
        // SAFETY: the handle belongs to a live window owned by this process, and Tauri
        // runs setup and synchronous commands on the thread that created the window.
        unsafe {
            let current_extended_style = GetWindowLongPtrW(pill_window_native_handle, GWL_EXSTYLE);
            let overlay_extended_style =
                (current_extended_style | extended_style_bits_to_add) & !extended_style_bits_to_remove;
            SetWindowLongPtrW(pill_window_native_handle, GWL_EXSTYLE, overlay_extended_style);
            // A layered window stays invisible until it's told how to blend; fully opaque leaves
            // the page's own transparency in charge.
            SetLayeredWindowAttributes(pill_window_native_handle, COLORREF(0), u8::MAX, LWA_ALPHA)
                .map_err(|error| error.to_string())?;
            let current_window_style = GetWindowLongPtrW(pill_window_native_handle, GWL_STYLE);
            SetWindowLongPtrW(
                pill_window_native_handle,
                GWL_STYLE,
                current_window_style & !window_style_bits_to_remove,
            );
            // Windows caches the frame; it only re-reads the new styles after FRAMECHANGED.
            SetWindowPos(
                pill_window_native_handle,
                None,
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            )
            .map_err(|error| error.to_string())?;
        }
        block_classic_frame_painting(pill_window_native_handle)?;
        // The window starts hidden, and the low target is otherwise only set when it hides
        // again; without this, a Crest started with nothing playing (e.g. at login) would keep
        // the normal target until the first song, about 78 MB instead of 20 (measured).
        adjust_pill_webview_memory_target(pill_window, true)
    }

    fn watch_display_changes(
        pill_window: &WebviewWindow,
        display_change_listener: DisplayChangeListener,
    ) -> Result<(), String> {
        let pill_window_native_handle = pill_window.hwnd().map_err(|error| error.to_string())?;
        forward_display_changes(pill_window_native_handle, display_change_listener)
    }

    fn show_pill_window_without_activating(pill_window: &WebviewWindow) -> Result<(), String> {
        let pill_window_native_handle = pill_window.hwnd().map_err(|error| error.to_string())?;
        adjust_pill_webview_memory_target(pill_window, false)?;
        // SAFETY: same live, same-thread window handle as above. The return value only
        // says whether the window was visible before, so there is no error to handle.
        unsafe {
            let _ = ShowWindow(pill_window_native_handle, SW_SHOWNOACTIVATE);
        }
        Ok(())
    }

    fn hide_pill_window(pill_window: &WebviewWindow) -> Result<(), String> {
        let pill_window_native_handle = pill_window.hwnd().map_err(|error| error.to_string())?;
        // SAFETY: same live, same-thread window handle as above; the return value only
        // says whether the window was visible before.
        unsafe {
            let _ = ShowWindow(pill_window_native_handle, SW_HIDE);
        }
        adjust_pill_webview_memory_target(pill_window, true)
    }

    fn set_pill_window_interactive_area(
        pill_window: &WebviewWindow,
        area_top_left: PhysicalPosition<i32>,
        area_size: PhysicalSize<u32>,
        is_click_through: bool,
    ) -> Result<(), String> {
        let pill_window_native_handle = pill_window.hwnd().map_err(|error| error.to_string())?;
        // A rectangle, not a rounded shape: region edges are hard pixel cuts, so a rounded
        // region would give jagged corners; CSS draws the smooth rounded pill inside it.
        // SAFETY: the region handle is created here and either handed to Windows (which
        // then owns and frees it) or deleted by us if Windows refuses it.
        unsafe {
            let interactive_region = CreateRectRgn(
                area_top_left.x,
                area_top_left.y,
                area_top_left.x + area_size.width as i32,
                area_top_left.y + area_size.height as i32,
            );
            if interactive_region.is_invalid() {
                return Err("Windows could not create the pill's interactive region".to_string());
            }
            if SetWindowRgn(pill_window_native_handle, Some(interactive_region), true) == 0 {
                let _ = DeleteObject(interactive_region.into());
                return Err("Windows refused the pill's interactive region".to_string());
            }
            let current_extended_style = GetWindowLongPtrW(pill_window_native_handle, GWL_EXSTYLE);
            let click_through_bit = WS_EX_TRANSPARENT.0 as isize;
            let next_extended_style = if is_click_through {
                current_extended_style | click_through_bit
            } else {
                current_extended_style & !click_through_bit
            };
            if next_extended_style != current_extended_style {
                SetWindowLongPtrW(pill_window_native_handle, GWL_EXSTYLE, next_extended_style);
            }
        }
        Ok(())
    }
}
