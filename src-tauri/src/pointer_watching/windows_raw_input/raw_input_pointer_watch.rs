//! Windows: raw mouse input for an invisible message-only window. With RIDEV_INPUTSINK the
//! window gets every mouse report even while another app is in front, and only reads it: raw
//! input isn't part of the chain that delivers the mouse to apps (a low-level mouse hook is,
//! and a slow hook delays every mouse movement on the PC). The handler's timer is a window
//! timer on the same thread.

use std::cell::RefCell;
use std::ffi::c_void;
use std::thread;

use windows::core::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Input::{RegisterRawInputDevices, HRAWINPUT, RAWINPUTDEVICE, RIDEV_INPUTSINK};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, KillTimer, RegisterClassW, SetTimer, HWND_MESSAGE,
    MSG, WINDOW_EX_STYLE, WINDOW_STYLE, WM_INPUT, WM_TIMER, WNDCLASSW,
};

use crate::pointer_watching::pointer_watch::{PointerSampleHandler, PointerWatch, RestTimerChange};
use crate::pointer_watching::windows_raw_input::raw_input_mouse_report_reader::{read_button_gone_down, read_pointer_sample};

const POINTER_WATCHER_THREAD_NAME: &str = "crest-pointer-watcher";
/// The HID usage page and usage that mean "a mouse" (HID Usage Tables, Generic Desktop page).
const GENERIC_DESKTOP_USAGE_PAGE: u16 = 0x01;
const MOUSE_USAGE: u16 = 0x02;
const REST_TIMER_IDENTIFIER: usize = 1;

struct PointerWatcherThreadState {
    watcher_window: HWND,
    pointer_sample_handler: Box<dyn PointerSampleHandler>,
}

thread_local! {
    /// Windows calls the window procedure as a plain function on the watcher thread; this is
    /// how it reaches the handler.
    static POINTER_WATCHER_THREAD_STATE: RefCell<Option<PointerWatcherThreadState>> = const { RefCell::new(None) };
}

pub struct RawInputPointerWatch;

impl PointerWatch for RawInputPointerWatch {
    fn start_watching_pointer(pointer_sample_handler: Box<dyn PointerSampleHandler>) -> Result<(), String> {
        thread::Builder::new()
            .name(POINTER_WATCHER_THREAD_NAME.to_string())
            .spawn(move || {
                if let Err(watch_error) = run_pointer_watcher_thread(pointer_sample_handler) {
                    eprintln!("Crest: the pointer can't be watched: {watch_error}");
                }
            })
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

fn run_pointer_watcher_thread(pointer_sample_handler: Box<dyn PointerSampleHandler>) -> windows::core::Result<()> {
    // SAFETY: the watcher window is created and used only on this thread, which lives as long
    // as the app; the structures passed to Windows outlive every call that uses them.
    unsafe {
        let crest_program = GetModuleHandleW(None)?;
        let watcher_window_class_name = w!("CrestPointerWatcher");
        let watcher_window_class = WNDCLASSW {
            lpfnWndProc: Some(raw_input_window_procedure),
            hInstance: crest_program.into(),
            lpszClassName: watcher_window_class_name,
            ..Default::default()
        };
        if RegisterClassW(&watcher_window_class) == 0 {
            return Err(windows::core::Error::from_thread());
        }
        // Message-only: never shown, not in any window list, only receives messages.
        let watcher_window = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            watcher_window_class_name,
            None,
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(crest_program.into()),
            None,
        )?;
        POINTER_WATCHER_THREAD_STATE.set(Some(PointerWatcherThreadState { watcher_window, pointer_sample_handler }));
        let mouse_registration = RAWINPUTDEVICE {
            usUsagePage: GENERIC_DESKTOP_USAGE_PAGE,
            usUsage: MOUSE_USAGE,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: watcher_window,
        };
        RegisterRawInputDevices(&[mouse_registration], size_of::<RAWINPUTDEVICE>() as u32)?;

        // GetMessageW sleeps until the mouse reports something or the timer runs out: no CPU
        // while the mouse is still.
        let mut queued_message = MSG::default();
        while GetMessageW(&mut queued_message, None, 0, 0).as_bool() {
            DispatchMessageW(&queued_message);
        }
    }
    Ok(())
}

unsafe extern "system" fn raw_input_window_procedure(
    watcher_window: HWND,
    window_message: u32,
    message_wparam: WPARAM,
    message_lparam: LPARAM,
) -> LRESULT {
    if window_message == WM_INPUT {
        handle_raw_input(HRAWINPUT(message_lparam.0 as *mut c_void));
    } else if window_message == WM_TIMER && message_wparam.0 == REST_TIMER_IDENTIFIER {
        // One-shot: Windows timers repeat until killed.
        let _ = KillTimer(Some(watcher_window), REST_TIMER_IDENTIFIER);
        if let Some(pointer_sample) = read_pointer_sample(false) {
            with_pointer_watcher_thread_state(|thread_state| {
                thread_state.pointer_sample_handler.handle_rest_timer_elapsed(pointer_sample);
            });
        }
        return LRESULT(0);
    }
    // Also for WM_INPUT: Windows frees the report's data only through DefWindowProc.
    DefWindowProcW(watcher_window, window_message, message_wparam, message_lparam)
}

fn handle_raw_input(raw_input_handle: HRAWINPUT) {
    let Some(has_button_gone_down) = read_button_gone_down(raw_input_handle) else {
        return;
    };
    let Some(pointer_sample) = read_pointer_sample(has_button_gone_down) else {
        return;
    };
    with_pointer_watcher_thread_state(|thread_state| {
        let rest_timer_change = thread_state.pointer_sample_handler.handle_pointer_sample(pointer_sample);
        // SAFETY: the timer belongs to this thread's own window.
        unsafe {
            match rest_timer_change {
                RestTimerChange::LeaveAsIs => {}
                // A new timer with the same identifier replaces the running one.
                RestTimerChange::Restart(rest_delay) => {
                    SetTimer(Some(thread_state.watcher_window), REST_TIMER_IDENTIFIER, rest_delay.as_millis() as u32, None);
                }
                RestTimerChange::Stop => {
                    let _ = KillTimer(Some(thread_state.watcher_window), REST_TIMER_IDENTIFIER);
                }
            }
        }
    });
}

fn with_pointer_watcher_thread_state(state_action: impl FnOnce(&mut PointerWatcherThreadState)) {
    POINTER_WATCHER_THREAD_STATE.with_borrow_mut(|thread_state| {
        if let Some(thread_state) = thread_state.as_mut() {
            state_action(thread_state);
        }
    });
}
