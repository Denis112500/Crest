//! One Claude Code session as the tracker keeps it. A folder of its own (split after CC.2, when
//! the file reached 325 lines) so its parts share the session's fields while the rest of the
//! source can't touch them.

mod tracked_session_hook_events;
mod tracked_session_row;
mod tracked_session_state;
mod tracked_session_user_actions;

pub use tracked_session_state::TrackedSession;
