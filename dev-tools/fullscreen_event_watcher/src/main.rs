// Dev tool, not part of Crest: logs what Windows reports about fullscreen apps, to pick how
// Crest hides the pill during games. It registers as an appbar (so the shell sends it
// ABN_FULLSCREENAPP, the signal the taskbar uses to hide itself) and listens for changes of
// the front window. Every event prints the front window, its monitor, whether it covers that
// monitor, and what SHQueryUserNotificationState says. Read-only: it never touches other
// windows or processes.
//
//   cd dev-tools/fullscreen_event_watcher
//   cargo run --release -- <watch seconds> <log file>
mod fullscreen_state_report;

use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Mutex, OnceLock, PoisonError};
use std::time::Instant;

use windows::core::{w, BOOL};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Console::SetConsoleCtrlHandler;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows::Win32::UI::HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2};
use windows::Win32::UI::Shell::{
    SHAppBarMessage, ABM_NEW, ABM_REMOVE, ABN_FULLSCREENAPP, ABN_POSCHANGED, ABN_STATECHANGE, ABN_WINDOWARRANGE,
    APPBARDATA,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, PostQuitMessage,
    PostThreadMessageW, RegisterClassW, RegisterWindowMessageW, SetTimer, TranslateMessage,
    EVENT_SYSTEM_FOREGROUND, MSG, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS, WM_APP, WM_QUIT, WM_TIMER,
    WNDCLASSW, WS_EX_TOOLWINDOW, WS_POPUP,
};

use fullscreen_state_report::describe_fullscreen_state;

const DEFAULT_WATCH_SECONDS: u32 = 600;
const MILLISECONDS_PER_SECOND: u32 = 1000;
const APPBAR_CALLBACK_MESSAGE: u32 = WM_APP + 1;
const WATCH_END_TIMER_ID: usize = 1;

static WATCH_START_TIME: OnceLock<Instant> = OnceLock::new();
static WATCH_LOG_FILE: Mutex<Option<File>> = Mutex::new(None);
static TASKBAR_CREATED_MESSAGE: AtomicU32 = AtomicU32::new(0);
static MESSAGE_LOOP_THREAD_ID: AtomicU32 = AtomicU32::new(0);

fn write_watch_log_line(event_description: &str) {
    let seconds_since_start = WATCH_START_TIME.get().map_or(0.0, |start_time| start_time.elapsed().as_secs_f64());
    let watch_log_line = format!("{seconds_since_start:8.2}s  {event_description}");
    println!("{watch_log_line}");
    if let Some(watch_log_file) = WATCH_LOG_FILE.lock().unwrap_or_else(PoisonError::into_inner).as_mut() {
        let _ = writeln!(watch_log_file, "{watch_log_line}");
    }
}

fn log_event_with_fullscreen_state(event_name: &str) {
    write_watch_log_line(&format!("{event_name:<24} {}", describe_fullscreen_state()));
}

fn send_appbar_message(appbar_message: u32, watcher_window: HWND) -> bool {
    let mut appbar_description = APPBARDATA {
        cbSize: size_of::<APPBARDATA>() as u32,
        hWnd: watcher_window,
        uCallbackMessage: APPBAR_CALLBACK_MESSAGE,
        ..Default::default()
    };
    // SAFETY: the struct lives for the whole call and describes our own window.
    unsafe { SHAppBarMessage(appbar_message, &mut appbar_description) != 0 }
}

unsafe extern "system" fn watcher_window_procedure(
    watcher_window: HWND,
    window_message: u32,
    message_wparam: WPARAM,
    message_lparam: LPARAM,
) -> LRESULT {
    if window_message == APPBAR_CALLBACK_MESSAGE {
        let appbar_notification_name = match message_wparam.0 as u32 {
            ABN_FULLSCREENAPP if message_lparam.0 != 0 => "ABN_FULLSCREENAPP open",
            ABN_FULLSCREENAPP => "ABN_FULLSCREENAPP close",
            ABN_STATECHANGE => "ABN_STATECHANGE",
            ABN_POSCHANGED => "ABN_POSCHANGED",
            ABN_WINDOWARRANGE => "ABN_WINDOWARRANGE",
            _ => "ABN_<unknown>",
        };
        log_event_with_fullscreen_state(appbar_notification_name);
        return LRESULT(0);
    }
    // Explorer forgets every appbar when it restarts and then broadcasts "TaskbarCreated".
    if window_message != 0 && window_message == TASKBAR_CREATED_MESSAGE.load(Ordering::Relaxed) {
        let registered_again = send_appbar_message(ABM_NEW, watcher_window);
        write_watch_log_line(&format!("Explorer restarted; appbar registered again: {registered_again}"));
        return LRESULT(0);
    }
    if window_message == WM_TIMER {
        PostQuitMessage(0);
        return LRESULT(0);
    }
    DefWindowProcW(watcher_window, window_message, message_wparam, message_lparam)
}

unsafe extern "system" fn front_window_changed(
    _event_hook: HWINEVENTHOOK,
    _event_kind: u32,
    _event_window: HWND,
    _event_object_id: i32,
    _event_child_id: i32,
    _event_thread_id: u32,
    _event_time_milliseconds: u32,
) {
    log_event_with_fullscreen_state("front window changed");
}

/// Ctrl+C ends the message loop instead of killing the process, so the appbar is removed.
unsafe extern "system" fn stop_watching_on_ctrl_c(_console_control_kind: u32) -> BOOL {
    let _ = PostThreadMessageW(MESSAGE_LOOP_THREAD_ID.load(Ordering::Relaxed), WM_QUIT, WPARAM(0), LPARAM(0));
    true.into()
}

fn main() -> windows::core::Result<()> {
    let command_line_arguments: Vec<String> = std::env::args().collect();
    let watch_seconds = command_line_arguments
        .get(1)
        .and_then(|watch_seconds_argument| watch_seconds_argument.parse().ok())
        .unwrap_or(DEFAULT_WATCH_SECONDS);
    if let Some(watch_log_path) = command_line_arguments.get(2) {
        match File::create(watch_log_path) {
            Ok(watch_log_file) => *WATCH_LOG_FILE.lock().unwrap_or_else(PoisonError::into_inner) = Some(watch_log_file),
            Err(create_error) => eprintln!("could not create {watch_log_path}: {create_error}"),
        }
    }
    WATCH_START_TIME.get_or_init(Instant::now);

    // SAFETY: plain Win32 setup on this thread; every handle created here is used and
    // released on this same thread.
    unsafe {
        // Without this, a scaled monitor reports scaled coordinates and "covers the monitor"
        // would be computed wrongly. Crest (through Tauri) is per-monitor aware as well.
        SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)?;
        MESSAGE_LOOP_THREAD_ID.store(GetCurrentThreadId(), Ordering::Relaxed);
        TASKBAR_CREATED_MESSAGE.store(RegisterWindowMessageW(w!("TaskbarCreated")), Ordering::Relaxed);

        let this_program = GetModuleHandleW(None)?;
        let watcher_window_class_name = w!("CrestFullscreenEventWatcher");
        let watcher_window_class = WNDCLASSW {
            lpfnWndProc: Some(watcher_window_procedure),
            hInstance: this_program.into(),
            lpszClassName: watcher_window_class_name,
            ..Default::default()
        };
        if RegisterClassW(&watcher_window_class) == 0 {
            return Err(windows::core::Error::from_thread());
        }
        // A hidden top-level window: appbar notifications need a window to arrive at, and
        // message-only windows don't receive the TaskbarCreated broadcast.
        let watcher_window = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            watcher_window_class_name,
            w!("Crest fullscreen event watcher"),
            WS_POPUP,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(this_program.into()),
            None,
        )?;
        let is_appbar_registered = send_appbar_message(ABM_NEW, watcher_window);
        write_watch_log_line(&format!("appbar registered: {is_appbar_registered}"));
        // Out of context: Windows queues each event to this thread's message loop instead of
        // loading our code into other processes, so nothing ever touches the game.
        let front_window_hook = SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_FOREGROUND,
            None,
            Some(front_window_changed),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        );
        SetConsoleCtrlHandler(Some(stop_watching_on_ctrl_c), true)?;
        SetTimer(Some(watcher_window), WATCH_END_TIMER_ID, watch_seconds * MILLISECONDS_PER_SECOND, None);
        write_watch_log_line(&format!("watching for {watch_seconds} s; Ctrl+C stops early"));
        log_event_with_fullscreen_state("start");

        let mut queued_message = MSG::default();
        while GetMessageW(&mut queued_message, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&queued_message);
            DispatchMessageW(&queued_message);
        }

        let _ = UnhookWinEvent(front_window_hook);
        send_appbar_message(ABM_REMOVE, watcher_window);
        let _ = DestroyWindow(watcher_window);
    }
    write_watch_log_line("stopped");
    Ok(())
}
