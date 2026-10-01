// Tauri makes the window transparent with DWM "blur behind", where anything the classic
// Win32 painting code draws into the window shows up white. Because the pill has a custom
// window region, Windows doesn't draw a modern frame, and the default handling of a few
// messages repaints the classic title bar instead: a white "Crest" bar with a close button
// across the top of the window, visible in the transparent margin and corners around the
// pill (seen on a click or a focus change). A subclass is a hook that receives the
// window's messages before Tauri does; this one stops exactly those repaints.

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows::Win32::UI::WindowsAndMessaging::{WM_NCACTIVATE, WM_NCPAINT};

/// Any number works; it only tells this subclass apart from others on the same window.
const CLASSIC_FRAME_PAINTING_BLOCKER_SUBCLASS_ID: usize = 1;
/// Undocumented messages that themed windows use to draw the title bar and frame. They are
/// missing from the `windows` crate; their numbers are the ones Windows itself uses.
const WM_NCUAHDRAWCAPTION: u32 = 0x00AE;
const WM_NCUAHDRAWFRAME: u32 = 0x00AF;
/// Documented value for `WM_NCACTIVATE`: "don't repaint the frame for this change".
const SKIP_FRAME_REPAINT_ON_ACTIVATION_CHANGE: isize = -1;

pub fn block_classic_frame_painting(pill_window_native_handle: HWND) -> Result<(), String> {
    // SAFETY: called on the thread that created the window (Tauri's setup), as subclassing
    // requires; the subclass procedure below lives for the whole program.
    let was_subclassed = unsafe {
        SetWindowSubclass(
            pill_window_native_handle,
            Some(skip_classic_frame_painting),
            CLASSIC_FRAME_PAINTING_BLOCKER_SUBCLASS_ID,
            0,
        )
    };
    if was_subclassed.as_bool() {
        Ok(())
    } else {
        Err("Windows refused to hook the pill window's frame painting".to_string())
    }
}

unsafe extern "system" fn skip_classic_frame_painting(
    window: HWND,
    message: u32,
    message_word_parameter: WPARAM,
    message_long_parameter: LPARAM,
    _subclass_id: usize,
    _subclass_reference_data: usize,
) -> LRESULT {
    match message {
        // The pill has no frame area at all, so there is nothing to paint.
        WM_NCPAINT | WM_NCUAHDRAWCAPTION | WM_NCUAHDRAWFRAME => LRESULT(0),
        // Still passed on, because Tauri tracks focus with it; only the repaint is skipped.
        WM_NCACTIVATE => DefSubclassProc(
            window,
            message,
            message_word_parameter,
            LPARAM(SKIP_FRAME_REPAINT_ON_ACTIVATION_CHANGE),
        ),
        _ => DefSubclassProc(window, message, message_word_parameter, message_long_parameter),
    }
}
