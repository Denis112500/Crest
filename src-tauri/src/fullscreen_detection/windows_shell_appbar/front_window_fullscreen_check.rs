use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST};
use windows::Win32::UI::WindowsAndMessaging::{GetClassNameW, GetForegroundWindow, GetWindowRect};

/// Window classes of the desktop itself. The desktop covers the whole monitor, so without
/// this, clicking an empty spot of it could count as a fullscreen app.
const DESKTOP_WINDOW_CLASS_NAMES: [&str; 2] = ["Progman", "WorkerW"];

/// Windows limits window class names to 256 characters.
const WINDOW_CLASS_NAME_BUFFER_LENGTH: usize = 256;

/// The shell only says that *some* fullscreen app opened, not where. This checks that the
/// front window covers the monitor the pill is on, so a fullscreen video on another
/// monitor leaves the pill alone.
pub fn is_fullscreen_app_in_front_of_pill(pill_window: HWND) -> bool {
    // SAFETY: every call only reads properties of window and monitor handles Windows just
    // returned; if the window closed in between, the calls fail and the answer is "no".
    unsafe {
        let front_window = GetForegroundWindow();
        if front_window.is_invalid() || is_desktop_window(front_window) {
            return false;
        }
        let front_window_monitor = MonitorFromWindow(front_window, MONITOR_DEFAULTTONEAREST);
        if front_window_monitor != MonitorFromWindow(pill_window, MONITOR_DEFAULTTONEAREST) {
            return false;
        }
        let mut front_window_monitor_description =
            MONITORINFO { cbSize: size_of::<MONITORINFO>() as u32, ..Default::default() };
        let mut front_window_rectangle = RECT::default();
        if !GetMonitorInfoW(front_window_monitor, &mut front_window_monitor_description).as_bool()
            || GetWindowRect(front_window, &mut front_window_rectangle).is_err()
        {
            return false;
        }
        let monitor_rectangle = front_window_monitor_description.rcMonitor;
        front_window_rectangle.left <= monitor_rectangle.left
            && front_window_rectangle.top <= monitor_rectangle.top
            && front_window_rectangle.right >= monitor_rectangle.right
            && front_window_rectangle.bottom >= monitor_rectangle.bottom
    }
}

fn is_desktop_window(window_to_check: HWND) -> bool {
    let mut class_name_buffer = [0u16; WINDOW_CLASS_NAME_BUFFER_LENGTH];
    // SAFETY: the buffer outlives the call, and Windows writes at most its length.
    let class_name_length = unsafe { GetClassNameW(window_to_check, &mut class_name_buffer) }.max(0) as usize;
    let window_class_name = String::from_utf16_lossy(&class_name_buffer[..class_name_length]);
    DESKTOP_WINDOW_CLASS_NAMES.contains(&window_class_name.as_str())
}
