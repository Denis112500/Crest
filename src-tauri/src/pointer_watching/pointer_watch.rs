//! The trait each OS implements: report where the mouse pointer is whenever it moves or a
//! button goes down, anywhere on screen, without taking any input away from other apps, and
//! run one timer for the handler (the pointer sends nothing while it stands still).

use std::time::Duration;

/// One report, in physical screen pixels (the same pixels as the window positions).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerSample {
    pub screen_x: i32,
    pub screen_y: i32,
    /// A mouse button was pressed in this report (a click, or the start of a drag).
    pub has_button_gone_down: bool,
}

/// What the handler wants done with its one timer after a report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestTimerChange {
    LeaveAsIs,
    /// Start it, or start it over if it's running.
    Restart(Duration),
    Stop,
}

/// Called on the watcher's own thread, so it must be quick.
pub trait PointerSampleHandler: Send {
    fn handle_pointer_sample(&mut self, pointer_sample: PointerSample) -> RestTimerChange;
    /// The timer ran out; the sample is where the pointer is now.
    fn handle_rest_timer_elapsed(&mut self, pointer_sample: PointerSample);
}

/// Lets the pill notice the pointer resting on it while its window takes no clicks. Each
/// operating system gets its own implementation in its own folder.
pub trait PointerWatch {
    /// Starts watching on a background thread that sleeps until the mouse reports something
    /// or the timer runs out (no polling). The pointer and the clicks still go where they
    /// always go.
    fn start_watching_pointer(pointer_sample_handler: Box<dyn PointerSampleHandler>) -> Result<(), String>;
}
