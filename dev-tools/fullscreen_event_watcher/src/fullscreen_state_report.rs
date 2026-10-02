// One line describing "how fullscreen is the screen right now": the front window, its monitor,
// whether it covers that monitor, and what the shell's own notification state says.

use windows::Win32::Foundation::RECT;
use windows::Win32::Graphics::Gdi::{
    GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITORINFOEXW, MONITOR_DEFAULTTONEAREST,
};
use windows::Win32::UI::Shell::{
    SHQueryUserNotificationState, QUNS_ACCEPTS_NOTIFICATIONS, QUNS_APP, QUNS_BUSY, QUNS_NOT_PRESENT,
    QUNS_PRESENTATION_MODE, QUNS_QUIET_TIME, QUNS_RUNNING_D3D_FULL_SCREEN,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetForegroundWindow, GetWindowRect, GetWindowTextW, MONITORINFOF_PRIMARY,
};

const MAX_REPORTED_TITLE_CHARACTERS: usize = 50;
const WINDOW_TEXT_BUFFER_LENGTH: usize = 256;

pub fn describe_fullscreen_state() -> String {
    format!("quns={:<24} {}", describe_user_notification_state(), describe_front_window())
}

fn describe_user_notification_state() -> String {
    // SAFETY: no arguments; the shell only reports a value.
    match unsafe { SHQueryUserNotificationState() } {
        Ok(QUNS_NOT_PRESENT) => "not_present".to_string(),
        Ok(QUNS_BUSY) => "busy".to_string(),
        Ok(QUNS_RUNNING_D3D_FULL_SCREEN) => "d3d_exclusive_fullscreen".to_string(),
        Ok(QUNS_PRESENTATION_MODE) => "presentation_mode".to_string(),
        Ok(QUNS_ACCEPTS_NOTIFICATIONS) => "free".to_string(),
        Ok(QUNS_QUIET_TIME) => "quiet_time".to_string(),
        Ok(QUNS_APP) => "store_app".to_string(),
        Ok(unknown_state) => format!("unknown_{}", unknown_state.0),
        Err(query_error) => format!("error({query_error})"),
    }
}

fn describe_front_window() -> String {
    // SAFETY: every call only reads properties of a window handle that Windows just gave us;
    // if the window closed meanwhile, the calls fail harmlessly and report empty values.
    unsafe {
        let front_window = GetForegroundWindow();
        if front_window.is_invalid() {
            return "front=<none>".to_string();
        }
        let mut class_name_buffer = [0u16; WINDOW_TEXT_BUFFER_LENGTH];
        let class_name_length = GetClassNameW(front_window, &mut class_name_buffer).max(0) as usize;
        let front_window_class_name = String::from_utf16_lossy(&class_name_buffer[..class_name_length]);
        let mut title_buffer = [0u16; WINDOW_TEXT_BUFFER_LENGTH];
        let title_length = GetWindowTextW(front_window, &mut title_buffer).max(0) as usize;
        let front_window_title: String = String::from_utf16_lossy(&title_buffer[..title_length])
            .chars()
            .take(MAX_REPORTED_TITLE_CHARACTERS)
            .collect();

        let mut front_window_rectangle = RECT::default();
        let _ = GetWindowRect(front_window, &mut front_window_rectangle);
        let front_window_monitor = MonitorFromWindow(front_window, MONITOR_DEFAULTTONEAREST);
        let mut monitor_description = MONITORINFOEXW::default();
        monitor_description.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
        let _ = GetMonitorInfoW(
            front_window_monitor,
            &mut monitor_description as *mut MONITORINFOEXW as *mut MONITORINFO,
        );
        let monitor_rectangle = monitor_description.monitorInfo.rcMonitor;
        let monitor_device_name_length =
            monitor_description.szDevice.iter().position(|&character| character == 0).unwrap_or(0);
        let monitor_device_name = String::from_utf16_lossy(&monitor_description.szDevice[..monitor_device_name_length]);
        let is_primary_monitor = monitor_description.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0;
        let covers_whole_monitor = front_window_rectangle.left <= monitor_rectangle.left
            && front_window_rectangle.top <= monitor_rectangle.top
            && front_window_rectangle.right >= monitor_rectangle.right
            && front_window_rectangle.bottom >= monitor_rectangle.bottom;

        format!(
            "covers_monitor={covers_whole_monitor:<5} monitor={monitor_device_name}{} rect={} class='{front_window_class_name}' title='{front_window_title}'",
            if is_primary_monitor { "(primary)" } else { "" },
            describe_rectangle(&front_window_rectangle),
        )
    }
}

fn describe_rectangle(rectangle: &RECT) -> String {
    format!(
        "{},{} {}x{}",
        rectangle.left,
        rectangle.top,
        rectangle.right - rectangle.left,
        rectangle.bottom - rectangle.top
    )
}
