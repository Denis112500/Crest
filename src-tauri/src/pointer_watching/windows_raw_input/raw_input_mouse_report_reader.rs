//! Reads one raw mouse report: whether a button went down in it, and where the pointer is now
//! (the report itself only carries the movement, not the position).

use std::ffi::c_void;

use windows::Win32::Foundation::POINT;
use windows::Win32::UI::Input::{GetRawInputData, HRAWINPUT, RAWINPUT, RAWINPUTHEADER, RID_INPUT, RIM_TYPEMOUSE};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, RI_MOUSE_BUTTON_4_DOWN, RI_MOUSE_BUTTON_5_DOWN, RI_MOUSE_LEFT_BUTTON_DOWN, RI_MOUSE_MIDDLE_BUTTON_DOWN,
    RI_MOUSE_RIGHT_BUTTON_DOWN,
};

use crate::pointer_watching::pointer_watch::PointerSample;

const ANY_BUTTON_DOWN_FLAGS: u32 = RI_MOUSE_LEFT_BUTTON_DOWN
    | RI_MOUSE_RIGHT_BUTTON_DOWN
    | RI_MOUSE_MIDDLE_BUTTON_DOWN
    | RI_MOUSE_BUTTON_4_DOWN
    | RI_MOUSE_BUTTON_5_DOWN;

/// `None` for a report that isn't from a mouse or can't be read.
pub fn read_button_gone_down(raw_input_handle: HRAWINPUT) -> Option<bool> {
    let mut raw_input = RAWINPUT::default();
    let mut raw_input_size = size_of::<RAWINPUT>() as u32;
    // SAFETY: the buffer is a whole RAWINPUT and its size is passed along; a mouse report
    // always fits. The handle is the one Windows just sent with WM_INPUT.
    unsafe {
        let copied_size = GetRawInputData(
            raw_input_handle,
            RID_INPUT,
            Some(&mut raw_input as *mut RAWINPUT as *mut c_void),
            &mut raw_input_size,
            size_of::<RAWINPUTHEADER>() as u32,
        );
        if copied_size == u32::MAX || raw_input.header.dwType != RIM_TYPEMOUSE.0 {
            return None;
        }
        let button_flags = u32::from(raw_input.data.mouse.Anonymous.Anonymous.usButtonFlags);
        Some(button_flags & ANY_BUTTON_DOWN_FLAGS != 0)
    }
}

pub fn read_pointer_sample(has_button_gone_down: bool) -> Option<PointerSample> {
    let mut pointer_position = POINT::default();
    // SAFETY: only writes the position into the local structure.
    unsafe { GetCursorPos(&mut pointer_position) }.ok()?;
    Some(PointerSample { screen_x: pointer_position.x, screen_y: pointer_position.y, has_button_gone_down })
}
