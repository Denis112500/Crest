//! The core: collects what every activity source reports and decides what the pill shows
//! and when it's on screen. It never looks inside an activity's content.

mod activity_arbiter;
mod activity_publisher;
mod activity_source;
mod activity_source_registry;
mod activity_update;
mod pill_activity_arrangement;
mod pill_visibility_controller;
mod pill_visibility_policy;

// Public (not re-exported) because `tauri::generate_handler!` needs the defining path.
pub mod activity_action_command;
pub mod focus_activity_command;
pub mod pill_arrangement_command;
pub mod pill_visibility_command;

pub use activity_arbiter::{ActivityArbiter, SharedActivityArbiter};
pub use activity_publisher::ActivityPublisher;
pub use activity_source::ActivitySource;
pub use activity_source_registry::ActivitySourceRegistry;
pub use activity_update::{ActivityPresence, ActivityUpdate};
pub use pill_visibility_controller::PillVisibilityController;
