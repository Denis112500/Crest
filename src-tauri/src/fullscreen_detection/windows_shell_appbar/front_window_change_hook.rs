//! While a fullscreen app is reported, listens for "the front window changed". The shell's own
//! "fullscreen closed" never reaches a watcher that started during the fullscreen period
//! (measured 2026-10-06: Crest started while Valorant was in front, alt-tab out → no "closed"
//! until the next round trip), so without this the pill could stay hidden after the game.

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK};
use windows::Win32::UI::WindowsAndMessaging::{EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT};

use crate::fullscreen_detection::windows_shell_appbar::appbar_watcher_window_procedure::recheck_after_front_window_change;

/// Removed again when dropped, so the hook only exists while a fullscreen app is reported.
pub struct FrontWindowChangeHook {
    event_hook: HWINEVENTHOOK,
}

impl FrontWindowChangeHook {
    /// Must run on the watcher thread: Windows delivers the events to the message loop of the
    /// thread that set the hook.
    pub fn start() -> Option<Self> {
        // Out of context: Windows queues each event to this thread's message loop instead of
        // loading Crest's code into other processes, so nothing ever touches the game (safe
        // next to anti-cheat software).
        // SAFETY: the callback is a plain function that lives as long as the program.
        let event_hook = unsafe {
            SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                None,
                Some(handle_front_window_change_event),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            )
        };
        if event_hook.is_invalid() {
            eprintln!("Crest: could not listen for front window changes; the pill returns after the next game round trip");
            return None;
        }
        Some(Self { event_hook })
    }
}

impl Drop for FrontWindowChangeHook {
    fn drop(&mut self) {
        // SAFETY: the hook was set by this thread and is removed exactly once.
        let _ = unsafe { UnhookWinEvent(self.event_hook) };
    }
}

unsafe extern "system" fn handle_front_window_change_event(
    _event_hook: HWINEVENTHOOK,
    _event_kind: u32,
    _event_window: HWND,
    _event_object_id: i32,
    _event_child_id: i32,
    _event_thread_id: u32,
    _event_time_milliseconds: u32,
) {
    recheck_after_front_window_change();
}
